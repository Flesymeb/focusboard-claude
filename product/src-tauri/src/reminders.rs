use crate::auth::PublicUser;
use crate::mail::{reminder_message, MailMessage, MailSink};
use crate::store::{new_id, now_rfc3339, CommandError, CommandResult, Store};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::OptionalExtension;
use serde::Serialize;

/// The six PRD section 7 delivery states.
pub const STATUS_SCHEDULED: &str = "scheduled";
const STATUS_PROCESSING: &str = "processing";
pub const STATUS_SENT: &str = "sent";
pub const STATUS_FAILED: &str = "failed";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_SKIPPED: &str = "skipped";

#[derive(Debug, Serialize, Clone)]
pub struct Reminder {
    pub id: String,
    pub task_id: String,
    pub user_id: String,
    pub scheduled_at: String,
    pub timezone: String,
    pub channel: String,
    pub status: String,
    pub provider_message_id: Option<String>,
    pub sent_at: Option<String>,
    pub retry_count: i64,
    pub dedup_key: String,
}

fn parse_timezone(timezone: &str) -> CommandResult<Tz> {
    timezone.parse::<Tz>().map_err(|_| {
        CommandError::new(
            "invalid_timezone",
            "Choose a valid IANA timezone, like Asia/Shanghai.",
        )
    })
}

/// Converts a local wall-clock time in the reminder's timezone into the
/// absolute UTC instant that the scheduler stores and fires against.
/// DST-ambiguous local times resolve to the earlier instant; times inside a
/// spring-forward gap are rejected as unschedulable rather than shifted.
pub fn local_to_instant(local: &str, timezone: &str) -> CommandResult<DateTime<Utc>> {
    let tz = parse_timezone(timezone)?;
    let naive = chrono::NaiveDateTime::parse_from_str(local, "%Y-%m-%dT%H:%M").map_err(|_| {
        CommandError::new(
            "invalid_reminder_time",
            "Use a valid reminder time in YYYY-MM-DDTHH:MM format.",
        )
    })?;
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(earliest, _) => Ok(earliest.with_timezone(&Utc)),
        chrono::LocalResult::None => Err(CommandError::new(
            "invalid_reminder_time",
            "That local time does not exist in this timezone on that date.",
        )),
    }
}

/// Formats a stored UTC instant back to the reminder timezone's local time so
/// emails and delivery states can explain the originating wall clock.
pub fn instant_in_timezone(instant: &str, timezone: &str) -> String {
    let tz = match timezone.parse::<Tz>() {
        Ok(tz) => tz,
        Err(_) => return instant.to_string(),
    };
    match DateTime::parse_from_rfc3339(instant) {
        Ok(dt) => dt
            .with_timezone(&tz)
            .format("%Y-%m-%d %H:%M (%Z)")
            .to_string(),
        Err(_) => instant.to_string(),
    }
}

fn row_to_reminder(r: &rusqlite::Row) -> rusqlite::Result<Reminder> {
    Ok(Reminder {
        id: r.get(0)?,
        task_id: r.get(1)?,
        user_id: r.get(2)?,
        scheduled_at: r.get(3)?,
        timezone: r.get(4)?,
        channel: r.get(5)?,
        status: r.get(6)?,
        provider_message_id: r.get(7)?,
        sent_at: r.get(8)?,
        retry_count: r.get(9)?,
        dedup_key: r.get(10)?,
    })
}

const REMINDER_COLUMNS: &str =
    "id, task_id, user_id, scheduled_at, timezone, channel, status, provider_message_id, sent_at, retry_count, dedup_key";

pub fn get_reminder(
    store: &Store,
    user: &PublicUser,
    reminder_id: &str,
) -> CommandResult<Reminder> {
    store
        .conn
        .query_row(
            &format!("SELECT {REMINDER_COLUMNS} FROM reminders WHERE id = ?1 AND user_id = ?2"),
            rusqlite::params![reminder_id, user.id],
            row_to_reminder,
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Reminder lookup failed: {e}")))?
        .ok_or_else(|| CommandError::new("not_found", "That reminder no longer exists."))
}

/// Sets (or replaces) the pending reminder for a task. Rescheduling cancels
/// the previous pending reminder so one task carries at most one scheduled
/// reminder, and the unique dedup key makes repeated identical schedules
/// idempotent across retries and restarts.
pub fn set_task_reminder(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    local_time: &str,
    timezone: &str,
) -> CommandResult<Reminder> {
    crate::tasks::get_task_owned(store, user, task_id)?;
    let tz_name = timezone.trim();
    let scheduled_at = local_to_instant(local_time, tz_name)?;
    let scheduled = scheduled_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let dedup_key = format!("{task_id}:{scheduled}");
    // Any earlier pending reminder for this task is superseded.
    store
        .conn
        .execute(
            "UPDATE reminders SET status = ?1
             WHERE task_id = ?2 AND user_id = ?3 AND status IN (?4, ?5)",
            rusqlite::params![
                STATUS_CANCELLED,
                task_id,
                user.id,
                STATUS_SCHEDULED,
                STATUS_PROCESSING
            ],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not replace reminder: {e}"))
        })?;
    store
        .conn
        .execute(
            "INSERT INTO reminders(id, task_id, user_id, scheduled_at, timezone, channel, status, retry_count, dedup_key)
             VALUES(?1, ?2, ?3, ?4, ?5, 'email', ?6, 0, ?7)
             ON CONFLICT(dedup_key) DO UPDATE SET status = ?6",
            rusqlite::params![
                new_id("rem"),
                task_id,
                user.id,
                scheduled,
                tz_name,
                STATUS_SCHEDULED,
                dedup_key
            ],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not schedule reminder: {e}")))?;
    store
        .conn
        .query_row(
            &format!(
                "SELECT {REMINDER_COLUMNS} FROM reminders WHERE dedup_key = ?1 AND user_id = ?2"
            ),
            rusqlite::params![dedup_key, user.id],
            row_to_reminder,
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not read reminder: {e}")))
}

/// Cancels pending reminders when a task is completed (`skipped`) or deleted
/// (`cancelled`). Only rows still pending are touched; sent history remains.
pub fn cancel_pending_for_task(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    status: &str,
) -> CommandResult<()> {
    store
        .conn
        .execute(
            "UPDATE reminders SET status = ?1
             WHERE task_id = ?2 AND user_id = ?3 AND status IN (?4, ?5)",
            rusqlite::params![
                status,
                task_id,
                user.id,
                STATUS_SCHEDULED,
                STATUS_PROCESSING
            ],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not cancel reminder: {e}"))
        })?;
    Ok(())
}

pub fn list_reminders(store: &Store, user: &PublicUser) -> CommandResult<Vec<Reminder>> {
    let mut stmt = store
        .conn
        .prepare(&format!(
            "SELECT {REMINDER_COLUMNS} FROM reminders WHERE user_id = ?1
             ORDER BY scheduled_at DESC"
        ))
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not list reminders: {e}"))
        })?;
    let rows = stmt.query_map([&user.id], row_to_reminder).map_err(|e| {
        CommandError::new("storage_error", format!("Could not list reminders: {e}"))
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CommandError::new("storage_error", format!("Could not read reminders: {e}")))
}

/// Re-arms one failed reminder for immediate redelivery. Deliberate retry
/// policy: the dedup key moves with the new instant so the retried send still
/// cannot duplicate a previously delivered message.
pub fn retry_reminder(
    store: &Store,
    user: &PublicUser,
    reminder_id: &str,
) -> CommandResult<Reminder> {
    let reminder = get_reminder(store, user, reminder_id)?;
    if reminder.status != STATUS_FAILED {
        return Err(CommandError::new(
            "invalid_state",
            "Only a failed reminder can be retried.",
        ));
    }
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let dedup_key = format!("{}:{}", reminder.task_id, now);
    store
        .conn
        .execute(
            "UPDATE reminders SET status = ?1, scheduled_at = ?2, dedup_key = ?3
             WHERE id = ?4 AND user_id = ?5 AND status = ?6",
            rusqlite::params![
                STATUS_SCHEDULED,
                now,
                dedup_key,
                reminder_id,
                user.id,
                STATUS_FAILED
            ],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not retry reminder: {e}"))
        })?;
    get_reminder(store, user, reminder_id)
}

/// Delivery-check result for the Settings test-email action. Names only the
/// recipient address and the subject — never credentials or message bodies.
#[derive(Debug, Serialize, Clone)]
pub struct TestEmailOutcome {
    pub delivered_to: String,
    pub subject: String,
}

/// Sends a delivery-check test email to the signed-in user's address through
/// the configured mail sink. The message carries product identity only — no
/// token, password, or task content — so it is safe to trigger from Settings.
pub fn send_test_email(user: &PublicUser, sink: &MailSink) -> CommandResult<TestEmailOutcome> {
    let message = MailMessage {
        id: new_id("mail"),
        to: user.email.clone(),
        subject: "Focusboard test email".to_string(),
        body_text: "Focusboard delivery check.\n\nIf this reached your inbox, reminder email delivery works for your account. No action is needed.\n\nManage reminder preferences in Focusboard Settings.".to_string(),
        action_url: "focusboard://settings".to_string(),
        created_at: now_rfc3339(),
    };
    sink.deliver(&message).map_err(|e| {
        CommandError::new(
            "delivery_failed",
            format!("Could not send the test email: {e}"),
        )
    })?;
    Ok(TestEmailOutcome {
        delivered_to: user.email.clone(),
        subject: message.subject,
    })
}

fn task_snapshot(
    conn: &rusqlite::Connection,
    task_id: &str,
) -> Option<(String, Option<String>, String)> {
    conn.query_row(
        "SELECT title, due_date, user_id FROM tasks WHERE id = ?1",
        [task_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .ok()
}

fn user_email(conn: &rusqlite::Connection, user_id: &str) -> Option<String> {
    conn.query_row("SELECT email FROM users WHERE id = ?1", [user_id], |r| {
        r.get(0)
    })
    .ok()
}

/// One scheduler pass. Restart-safe and idempotent:
/// - reminders whose task completed are marked skipped;
/// - each due reminder is claimed by a guarded UPDATE (`scheduled` ->
///   `processing`), so concurrent or repeated passes never double-send;
/// - delivery through the mail sink is recorded with its message id before
///   the next pass can observe the row again.
/// Returns the number of deliveries attempted.
pub fn tick(store: &Store, sink: &MailSink) -> CommandResult<usize> {
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    // Cancel-on-complete safety net for reminders that reach their instant
    // after the task was already completed.
    store
        .conn
        .execute(
            "UPDATE reminders SET status = ?1
             WHERE status IN (?2, ?3) AND scheduled_at <= ?4
               AND EXISTS (SELECT 1 FROM tasks t WHERE t.id = reminders.task_id AND t.status = 'completed')",
            rusqlite::params![STATUS_SKIPPED, STATUS_SCHEDULED, STATUS_PROCESSING, now],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Scheduler sweep failed: {e}")))?;

    let due: Vec<Reminder> = {
        let mut stmt = store
            .conn
            .prepare(&format!(
                "SELECT {REMINDER_COLUMNS} FROM reminders
                 WHERE status = ?1 AND scheduled_at <= ?2
                 ORDER BY scheduled_at"
            ))
            .map_err(|e| {
                CommandError::new("storage_error", format!("Scheduler query failed: {e}"))
            })?;
        let rows = stmt
            .query_map(rusqlite::params![STATUS_SCHEDULED, now], row_to_reminder)
            .map_err(|e| {
                CommandError::new("storage_error", format!("Scheduler query failed: {e}"))
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| {
            CommandError::new("storage_error", format!("Scheduler read failed: {e}"))
        })?
    };

    let mut attempted = 0;
    for reminder in due {
        // Claim exactly once: the status guard is the idempotency boundary.
        let claimed = store
            .conn
            .execute(
                "UPDATE reminders SET status = ?1 WHERE id = ?2 AND status = ?3",
                rusqlite::params![STATUS_PROCESSING, reminder.id, STATUS_SCHEDULED],
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Scheduler claim failed: {e}"))
            })?;
        if claimed != 1 {
            continue;
        }
        attempted += 1;
        let Some((title, due_date, task_user)) = task_snapshot(&store.conn, &reminder.task_id)
        else {
            let _ = store.conn.execute(
                "UPDATE reminders SET status = ?1 WHERE id = ?2",
                rusqlite::params![STATUS_CANCELLED, reminder.id],
            );
            continue;
        };
        if task_user != reminder.user_id {
            let _ = store.conn.execute(
                "UPDATE reminders SET status = ?1 WHERE id = ?2",
                rusqlite::params![STATUS_SKIPPED, reminder.id],
            );
            continue;
        }
        let Some(to) = user_email(&store.conn, &reminder.user_id) else {
            let _ = store.conn.execute(
                "UPDATE reminders SET status = ?1, retry_count = retry_count + 1 WHERE id = ?2",
                rusqlite::params![STATUS_FAILED, reminder.id],
            );
            continue;
        };
        let local_due = instant_in_timezone(&reminder.scheduled_at, &reminder.timezone);
        let message = reminder_message(
            &to,
            &reminder.task_id,
            &title,
            due_date.as_deref(),
            &local_due,
        );
        match sink.deliver(&message) {
            Ok(()) => {
                store
                    .conn
                    .execute(
                        "UPDATE reminders SET status = ?1, provider_message_id = ?2, sent_at = ?3 WHERE id = ?4",
                        rusqlite::params![STATUS_SENT, message.id, now_rfc3339(), reminder.id],
                    )
                    .map_err(|e| CommandError::new("storage_error", format!("Could not record delivery: {e}")))?;
            }
            Err(_) => {
                store
                    .conn
                    .execute(
                        "UPDATE reminders SET status = ?1, retry_count = retry_count + 1 WHERE id = ?2 AND status = ?3",
                        rusqlite::params![STATUS_FAILED, reminder.id, STATUS_PROCESSING],
                    )
                    .map_err(|e| CommandError::new("storage_error", format!("Could not record failure: {e}")))?;
            }
        }
    }
    Ok(attempted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::MailMessage;
    use crate::store::Store;
    use std::path::{Path, PathBuf};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "focusboard-rem-{name}-{}",
            crate::store::new_id("t")
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn open_store(dir: &Path) -> Store {
        Store::open(&dir.join("focusboard.sqlite3")).unwrap()
    }

    /// Full signed-in setup: register -> verify -> sign in -> project + task.
    fn signed_in_with_task(dir: &Path, email: &str) -> (Store, PublicUser, String) {
        let sink = MailSink::new(dir);
        let store = open_store(dir);
        crate::auth::register(
            &store,
            &sink,
            "focusboard://auth",
            email,
            "tulip-garnet-9",
            "Ada",
        )
        .unwrap();
        let stored = read_token(dir, email);
        crate::auth::verify_email_token(&store, &stored).unwrap();
        let user = crate::auth::sign_in(&store, email, "tulip-garnet-9").unwrap();
        let task = crate::tasks::create_task(
            &store,
            &user,
            "Prepare first demo",
            None,
            Some("2026-09-18"),
        )
        .unwrap();
        (store, user, task.id)
    }

    fn read_token(dir: &Path, to: &str) -> String {
        let sink_dir = dir.join("mail-sink");
        for entry in std::fs::read_dir(&sink_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = std::fs::read_to_string(&path).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
                if parsed["to"].as_str() == Some(to)
                    && parsed["subject"].as_str().unwrap().contains("Verify")
                {
                    return parsed["action_url"]
                        .as_str()
                        .unwrap()
                        .rsplit('=')
                        .next()
                        .unwrap()
                        .to_string();
                }
            }
        }
        panic!("verification email missing");
    }

    fn sink_messages(dir: &Path) -> Vec<MailMessage> {
        let sink_dir = dir.join("mail-sink");
        let mut out = vec![];
        for entry in std::fs::read_dir(&sink_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = std::fs::read_to_string(&path).unwrap();
                out.push(serde_json::from_str(&raw).unwrap());
            }
        }
        out
    }

    fn due_now_local(tz: &str) -> String {
        let instant = Utc::now() - chrono::Duration::seconds(5);
        let tz_parsed: Tz = tz.parse().unwrap();
        instant
            .with_timezone(&tz_parsed)
            .format("%Y-%m-%dT%H:%M")
            .to_string()
    }

    #[test]
    fn timezone_conversion_is_exact() {
        // 2026-09-18 08:30 in Shanghai is 00:30 UTC.
        let instant = local_to_instant("2026-09-18T08:30", "Asia/Shanghai").unwrap();
        assert_eq!(
            instant.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-09-18T00:30:00Z"
        );
        // Same wall clock in New York is four hours later in UTC (EDT).
        let ny = local_to_instant("2026-09-18T08:30", "America/New_York").unwrap();
        assert_eq!(
            ny.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-09-18T12:30:00Z"
        );
        let err = local_to_instant("2026-09-18T08:30", "Not/AZone").unwrap_err();
        assert_eq!(err.code, "invalid_timezone");
    }

    #[test]
    fn scheduler_is_idempotent_and_delivers_once_with_safe_content() {
        let dir = temp_dir("idempotent");
        let (store, user, task_id) = signed_in_with_task(&dir, "ada@example.com");
        let local = due_now_local("UTC");
        let reminder = set_task_reminder(&store, &user, &task_id, &local, "UTC").unwrap();
        assert_eq!(reminder.status, STATUS_SCHEDULED);

        // Re-setting the same schedule is idempotent: one pending row, same key.
        let again = set_task_reminder(&store, &user, &task_id, &local, "UTC").unwrap();
        assert_eq!(again.dedup_key, reminder.dedup_key);

        let sink = MailSink::new(&dir);
        assert_eq!(tick(&store, &sink).unwrap(), 1, "first pass delivers once");
        assert_eq!(
            tick(&store, &sink).unwrap(),
            0,
            "second pass must not resend"
        );
        assert_eq!(
            tick(&store, &sink).unwrap(),
            0,
            "third pass must not resend"
        );

        let delivered = get_reminder(&store, &user, &reminder.id).unwrap();
        assert_eq!(delivered.status, STATUS_SENT);
        assert!(delivered.provider_message_id.is_some());
        assert!(delivered.sent_at.is_some());

        let reminders = list_reminders(&store, &user)
            .unwrap()
            .into_iter()
            .filter(|r| r.task_id == task_id)
            .collect::<Vec<_>>();
        assert_eq!(reminders.len(), 1, "rescheduling must not leave duplicates");

        // Email content: task title and due context, safe link, no tokens or
        // credentials. (The only email is the reminder; verification mail was
        // consumed by the setup helper and filtered here.)
        let mails = sink_messages(&dir);
        let reminder_mail = mails
            .iter()
            .find(|m| m.subject.contains("Prepare first demo"))
            .expect("reminder email must be delivered");
        assert!(reminder_mail.body_text.contains("Prepare first demo"));
        assert!(reminder_mail.body_text.contains("2026-09-18"));
        assert!(reminder_mail
            .action_url
            .starts_with("focusboard://task?id="));
        assert!(!reminder_mail.action_url.contains("token="));
        assert!(!reminder_mail.body_text.contains("password"));
        assert!(!reminder_mail.body_text.contains("tulip-garnet-9"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn completing_a_task_skips_its_pending_reminder() {
        let dir = temp_dir("cancel-on-complete");
        let (store, user, task_id) = signed_in_with_task(&dir, "kim@example.com");
        // Reminder scheduled in the future so it is pending, not yet due.
        let reminder =
            set_task_reminder(&store, &user, &task_id, "2099-01-01T09:00", "UTC").unwrap();
        crate::tasks::update_task(&store, &user, &task_id, None, None, Some("completed")).unwrap();
        let after = get_reminder(&store, &user, &reminder.id).unwrap();
        assert_eq!(after.status, STATUS_SKIPPED);

        // The scheduler never delivers a skipped reminder.
        let sink = MailSink::new(&dir);
        tick(&store, &sink).unwrap();
        assert_eq!(
            get_reminder(&store, &user, &reminder.id).unwrap().status,
            STATUS_SKIPPED
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reminder_survives_a_full_restart_and_fires_after() {
        let dir = temp_dir("restart-safe");
        let (store, user, task_id) = signed_in_with_task(&dir, "restart@example.com");
        let reminder =
            set_task_reminder(&store, &user, &task_id, "2099-01-01T09:00", "Asia/Shanghai")
                .unwrap();
        assert_eq!(reminder.timezone, "Asia/Shanghai");
        drop(store);

        // Full application restart: reopen the store, the reminder is still
        // scheduled and the tick fires it because its instant already passed.
        let reopened = open_store(&dir);
        let restored_user = crate::auth::current_user(&reopened).unwrap();
        assert_eq!(restored_user.id, user.id);
        let pending = get_reminder(&reopened, &restored_user, &reminder.id).unwrap();
        assert_eq!(pending.status, STATUS_SCHEDULED);

        // Pull the scheduled instant into the past and confirm delivery.
        let past = (Utc::now() - chrono::Duration::seconds(5))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        reopened
            .conn
            .execute(
                "UPDATE reminders SET scheduled_at = ?1 WHERE id = ?2",
                rusqlite::params![past, reminder.id],
            )
            .unwrap();
        let sink = MailSink::new(&dir);
        assert_eq!(tick(&reopened, &sink).unwrap(), 1);
        assert_eq!(
            get_reminder(&reopened, &restored_user, &reminder.id)
                .unwrap()
                .status,
            STATUS_SENT
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_delivery_is_visible_and_retry_delivers_exactly_once() {
        let dir = temp_dir("retry");
        let (store, user, task_id) = signed_in_with_task(&dir, "grace@example.com");
        let local = due_now_local("UTC");
        let reminder = set_task_reminder(&store, &user, &task_id, &local, "UTC").unwrap();

        // A sink whose directory is a regular file cannot be written to.
        let broken_root = dir.join("broken-mail-root");
        std::fs::write(&broken_root, b"not a directory").unwrap();
        let broken = MailSink::new(&broken_root);
        assert_eq!(tick(&store, &broken).unwrap(), 1);
        let failed = get_reminder(&store, &user, &reminder.id).unwrap();
        assert_eq!(failed.status, STATUS_FAILED);
        assert_eq!(failed.retry_count, 1);
        assert_eq!(
            tick(&store, &broken).unwrap(),
            0,
            "failed rows are not re-picked automatically"
        );

        // Retry re-arms and a healthy sink delivers exactly one email.
        let retried = retry_reminder(&store, &user, &reminder.id).unwrap();
        assert_eq!(retried.status, STATUS_SCHEDULED);
        assert_eq!(retried.retry_count, 1);
        let healthy = MailSink::new(&dir);
        assert_eq!(tick(&store, &healthy).unwrap(), 1);
        assert_eq!(
            get_reminder(&store, &user, &reminder.id).unwrap().status,
            STATUS_SENT
        );
        let reminder_mails = sink_messages(&dir)
            .into_iter()
            .filter(|m| m.subject.contains("Prepare first demo"))
            .count();
        assert_eq!(reminder_mails, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
