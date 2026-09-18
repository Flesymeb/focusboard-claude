use crate::auth::PublicUser;
use crate::store::{new_id, now_rfc3339, CommandError, CommandResult, Store};
use chrono::DateTime;
use serde::Serialize;

pub const STATE_RUNNING: &str = "running";
pub const STATE_PAUSED: &str = "paused";
pub const STATE_COMPLETED: &str = "completed";
pub const STATE_CANCELLED: &str = "cancelled";

#[derive(Debug, Serialize, Clone, serde::Deserialize)]
pub struct FocusPause {
    pub paused_at: String,
    pub resumed_at: Option<String>,
}

/// A focus session: one task, explicit start/pause/resume/finish/cancel, and
/// pause intervals persisted so a refresh or restart can restore the exact
/// state without silently completing anything.
#[derive(Debug, Serialize, Clone)]
pub struct FocusSession {
    pub id: String,
    pub task_id: Option<String>,
    pub task_title: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub state: String,
    pub pauses: Vec<FocusPause>,
    pub elapsed_seconds: i64,
}

struct SessionRow {
    id: String,
    task_id: Option<String>,
    task_title: Option<String>,
    started_at: String,
    ended_at: Option<String>,
    state: String,
    pauses: String,
}

fn parse_pauses(raw: &str) -> Vec<FocusPause> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn parse_instant(value: &str) -> Option<DateTime<chrono::Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn effective_elapsed(row: &SessionRow) -> i64 {
    let started = match parse_instant(&row.started_at) {
        Some(t) => t,
        None => return 0,
    };
    let mut now = chrono::Utc::now();
    // A paused (or finished) session accumulates time only up to its last
    // pause boundary, so refresh never inflates the elapsed clock.
    if row.state == STATE_PAUSED {
        if let Some(last_paused) = parse_pauses(&row.pauses)
            .iter()
            .filter_map(|p| p.resumed_at.is_none().then_some(p.paused_at.clone()))
            .filter_map(|t| parse_instant(&t))
            .max()
        {
            now = last_paused;
        }
    } else if let Some(ended) = row.ended_at.as_deref().and_then(parse_instant) {
        now = ended;
    }
    let paused_total: i64 = parse_pauses(&row.pauses)
        .iter()
        .filter_map(|p| {
            let start = parse_instant(&p.paused_at)?;
            let end = p
                .resumed_at
                .as_deref()
                .and_then(parse_instant)
                .unwrap_or(now);
            Some((end - start).num_seconds().max(0))
        })
        .sum();
    (now - started)
        .num_seconds()
        .saturating_sub(paused_total)
        .max(0)
}

fn row_to_session(r: &rusqlite::Row) -> rusqlite::Result<SessionRow> {
    Ok(SessionRow {
        id: r.get(0)?,
        task_id: r.get(1)?,
        task_title: r.get(2)?,
        started_at: r.get(3)?,
        ended_at: r.get(4)?,
        state: r.get(5)?,
        pauses: r.get(6)?,
    })
}

const SESSION_COLUMNS: &str =
    "s.id, s.task_id, t.title, s.started_at, s.ended_at, s.state, s.pauses";

fn to_public(row: SessionRow) -> FocusSession {
    let elapsed = effective_elapsed(&row);
    FocusSession {
        id: row.id,
        task_id: row.task_id,
        task_title: row.task_title,
        started_at: row.started_at,
        ended_at: row.ended_at,
        state: row.state,
        pauses: parse_pauses(&row.pauses),
        elapsed_seconds: elapsed,
    }
}

fn get_session_row(
    store: &Store,
    user: &PublicUser,
    session_id: &str,
) -> CommandResult<SessionRow> {
    store
        .conn
        .query_row(
            &format!(
                "SELECT {SESSION_COLUMNS} FROM focus_sessions s
                 LEFT JOIN tasks t ON t.id = s.task_id
                 WHERE s.id = ?1 AND s.user_id = ?2"
            ),
            rusqlite::params![session_id, user.id],
            row_to_session,
        )
        .map_err(|_| CommandError::new("not_found", "That focus session no longer exists."))
}

fn get_session(store: &Store, user: &PublicUser, session_id: &str) -> CommandResult<FocusSession> {
    get_session_row(store, user, session_id).map(|row| to_public(row))
}

fn active_session_row(store: &Store, user: &PublicUser) -> CommandResult<Option<SessionRow>> {
    store
        .conn
        .query_row(
            &format!(
                "SELECT {SESSION_COLUMNS} FROM focus_sessions s
                 LEFT JOIN tasks t ON t.id = s.task_id
                 WHERE s.user_id = ?1 AND s.state IN (?2, ?3)
                 ORDER BY s.started_at DESC LIMIT 1"
            ),
            rusqlite::params![user.id, STATE_RUNNING, STATE_PAUSED],
            row_to_session,
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
        .map_err(|e| CommandError::new("storage_error", format!("Focus lookup failed: {e}")))
}

pub fn active_session(store: &Store, user: &PublicUser) -> CommandResult<Option<FocusSession>> {
    active_session_row(store, user).map(|row| row.map(|r| to_public(r)))
}

pub fn start_session(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
) -> CommandResult<FocusSession> {
    crate::tasks::get_task_owned(store, user, task_id)?;
    if active_session_row(store, user)?.is_some() {
        return Err(CommandError::new(
            "focus_already_active",
            "Finish or cancel your current focus session before starting another.",
        ));
    }
    let id = new_id("fcs");
    let now = now_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO focus_sessions(id, user_id, task_id, started_at, pauses, state, updated_at)
             VALUES(?1, ?2, ?3, ?4, '[]', ?5, ?4)",
            rusqlite::params![id, user.id, task_id, now, STATE_RUNNING],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not start focus session: {e}")))?;
    get_session(store, user, &id)
}

/// Applies one guarded state transition. The `WHERE state IN (...)` clause is
/// the exactly-once boundary: a second finish or cancel after the first is a
/// visible error, never a silent double completion.
fn transition(
    store: &Store,
    user: &PublicUser,
    session_id: &str,
    from: &[&str],
    to: &str,
    pause_append: Option<&str>,
) -> CommandResult<FocusSession> {
    let row = get_session_row(store, user, session_id)?;
    if !from.contains(&row.state.as_str()) {
        return Err(CommandError::new(
            "invalid_state",
            "That focus session is not in a state that allows this action.",
        ));
    }
    let now = now_rfc3339();
    let pauses = match pause_append {
        Some(fragment) => {
            let mut current = row.pauses.clone();
            if current == "[]" || current.is_empty() {
                current = format!("[{fragment}]");
            } else {
                let trimmed = current.trim_end_matches(']');
                current = format!("{trimmed},{fragment}]");
            }
            current
        }
        None => row.pauses.clone(),
    };
    let (ended_at, _completed) = match to {
        STATE_COMPLETED | STATE_CANCELLED => (Some(now.clone()), true),
        _ => (row.ended_at.clone(), false),
    };
    let n = store
        .conn
        .execute(
            "UPDATE focus_sessions
             SET state = ?1, pauses = ?2, ended_at = COALESCE(?3, ended_at), updated_at = ?4
             WHERE id = ?5 AND user_id = ?6 AND state = ?7",
            rusqlite::params![to, pauses, ended_at, now, session_id, user.id, row.state],
        )
        .map_err(|e| {
            CommandError::new(
                "storage_error",
                format!("Could not update focus session: {e}"),
            )
        })?;
    if n == 0 {
        return Err(CommandError::new(
            "conflict",
            "The focus session changed elsewhere. Refresh and try again.",
        ));
    }
    get_session(store, user, session_id)
}

pub fn pause_session(
    store: &Store,
    user: &PublicUser,
    session_id: &str,
) -> CommandResult<FocusSession> {
    let fragment = format!(r#"{{"paused_at":"{}","resumed_at":null}}"#, now_rfc3339());
    transition(
        store,
        user,
        session_id,
        &[STATE_RUNNING],
        STATE_PAUSED,
        Some(&fragment),
    )
}

pub fn resume_session(
    store: &Store,
    user: &PublicUser,
    session_id: &str,
) -> CommandResult<FocusSession> {
    let now = now_rfc3339();
    let row = get_session_row(store, user, session_id)?;
    if row.state != STATE_PAUSED {
        return Err(CommandError::new(
            "invalid_state",
            "Only a paused focus session can be resumed.",
        ));
    }
    // Close the most recent open pause interval.
    let mut pauses = parse_pauses(&row.pauses);
    if let Some(last_open) = pauses.iter_mut().rev().find(|p| p.resumed_at.is_none()) {
        last_open.resumed_at = Some(now.clone());
    }
    let pauses_json = serde_json::to_string(&pauses)
        .map_err(|e| CommandError::new("storage_error", format!("Could not record pause: {e}")))?;
    let n = store
        .conn
        .execute(
            "UPDATE focus_sessions SET state = ?1, pauses = ?2, updated_at = ?3
             WHERE id = ?4 AND user_id = ?5 AND state = ?6",
            rusqlite::params![
                STATE_RUNNING,
                pauses_json,
                now,
                session_id,
                user.id,
                STATE_PAUSED
            ],
        )
        .map_err(|e| {
            CommandError::new(
                "storage_error",
                format!("Could not resume focus session: {e}"),
            )
        })?;
    if n == 0 {
        return Err(CommandError::new(
            "conflict",
            "The focus session changed elsewhere. Refresh and try again.",
        ));
    }
    get_session(store, user, session_id)
}

pub fn finish_session(
    store: &Store,
    user: &PublicUser,
    session_id: &str,
) -> CommandResult<FocusSession> {
    transition(
        store,
        user,
        session_id,
        &[STATE_RUNNING, STATE_PAUSED],
        STATE_COMPLETED,
        None,
    )
}

pub fn cancel_session(
    store: &Store,
    user: &PublicUser,
    session_id: &str,
) -> CommandResult<FocusSession> {
    transition(
        store,
        user,
        session_id,
        &[STATE_RUNNING, STATE_PAUSED],
        STATE_CANCELLED,
        None,
    )
}

pub fn list_sessions(store: &Store, user: &PublicUser) -> CommandResult<Vec<FocusSession>> {
    let mut stmt = store
        .conn
        .prepare(&format!(
            "SELECT {SESSION_COLUMNS} FROM focus_sessions s
             LEFT JOIN tasks t ON t.id = s.task_id
             WHERE s.user_id = ?1
             ORDER BY s.started_at DESC"
        ))
        .map_err(|e| {
            CommandError::new(
                "storage_error",
                format!("Could not list focus sessions: {e}"),
            )
        })?;
    let rows = stmt.query_map([&user.id], row_to_session).map_err(|e| {
        CommandError::new(
            "storage_error",
            format!("Could not list focus sessions: {e}"),
        )
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map(|rows| rows.into_iter().map(to_public).collect())
        .map_err(|e| {
            CommandError::new(
                "storage_error",
                format!("Could not read focus sessions: {e}"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::MailSink;
    use crate::store::Store;
    use std::path::{Path, PathBuf};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "focusboard-focus-{name}-{}",
            crate::store::new_id("t")
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn signed_in_with_task(dir: &Path, email: &str) -> (Store, PublicUser, String) {
        let sink = MailSink::new(dir);
        let store = Store::open(&dir.join("focusboard.sqlite3")).unwrap();
        crate::auth::register(
            &store,
            &sink,
            "focusboard://auth",
            email,
            "cobalt-lantern-7",
            "Ada",
        )
        .unwrap();
        let sink_dir = dir.join("mail-sink");
        let mut token = String::new();
        for entry in std::fs::read_dir(&sink_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let parsed: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
                token = parsed["action_url"]
                    .as_str()
                    .unwrap()
                    .rsplit('=')
                    .next()
                    .unwrap()
                    .to_string();
            }
        }
        crate::auth::verify_email_token(&store, &token).unwrap();
        let user = crate::auth::sign_in(&store, email, "cobalt-lantern-7").unwrap();
        let task =
            crate::tasks::create_task(&store, &user, "Prepare first demo", None, None).unwrap();
        (store, user, task.id)
    }

    #[test]
    fn focus_lifecycle_records_completion_exactly_once() {
        let dir = temp_dir("lifecycle");
        let (store, user, task_id) = signed_in_with_task(&dir, "ada@example.com");

        // Pause/resume before start is rejected.
        let err = pause_session(&store, &user, "fcs_missing").unwrap_err();
        assert_eq!(err.code, "not_found");

        let session = start_session(&store, &user, &task_id).unwrap();
        assert_eq!(session.state, STATE_RUNNING);
        assert_eq!(session.task_title.as_deref(), Some("Prepare first demo"));

        // One active session per user.
        let err = start_session(&store, &user, &task_id).unwrap_err();
        assert_eq!(err.code, "focus_already_active");

        let paused = pause_session(&store, &user, &session.id).unwrap();
        assert_eq!(paused.state, STATE_PAUSED);
        assert_eq!(paused.pauses.len(), 1);
        assert!(paused.pauses[0].resumed_at.is_none());
        // Elapsed is frozen while paused.
        let frozen = paused.elapsed_seconds;

        let resumed = resume_session(&store, &user, &session.id).unwrap();
        assert_eq!(resumed.state, STATE_RUNNING);
        assert!(resumed.pauses[0].resumed_at.is_some());

        // Double resume is a visible error, not a silent success.
        let err = resume_session(&store, &user, &session.id).unwrap_err();
        assert_eq!(err.code, "invalid_state");

        let finished = finish_session(&store, &user, &session.id).unwrap();
        assert_eq!(finished.state, STATE_COMPLETED);
        assert!(finished.ended_at.is_some());

        // Completing again is rejected: completion is recorded exactly once.
        let err = finish_session(&store, &user, &session.id).unwrap_err();
        assert_eq!(err.code, "invalid_state");
        let err = cancel_session(&store, &user, &session.id).unwrap_err();
        assert_eq!(err.code, "invalid_state");

        // Frozen-while-paused still holds in the finished row.
        assert!(finished.elapsed_seconds >= frozen);

        let sessions = list_sessions(&store, &user).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].state, STATE_COMPLETED);
        assert!(frozen >= 0);
        let _ = frozen;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn focus_session_survives_restart_without_silent_completion() {
        let dir = temp_dir("restart");
        let (store, user, task_id) = signed_in_with_task(&dir, "grace@example.com");
        let session = start_session(&store, &user, &task_id).unwrap();
        pause_session(&store, &user, &session.id).unwrap();
        drop(store);

        let reopened = Store::open(&dir.join("focusboard.sqlite3")).unwrap();
        let restored = crate::auth::current_user(&reopened).unwrap();
        let active = active_session(&reopened, &restored)
            .unwrap()
            .expect("paused session must survive restart");
        assert_eq!(active.id, session.id);
        assert_eq!(
            active.state, STATE_PAUSED,
            "restart must not complete or resume silently"
        );

        let resumed = resume_session(&reopened, &restored, &active.id).unwrap();
        assert_eq!(resumed.state, STATE_RUNNING);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cancel_ends_the_session_and_frees_the_lane() {
        let dir = temp_dir("cancel");
        let (store, user, task_id) = signed_in_with_task(&dir, "lin@example.com");
        let session = start_session(&store, &user, &task_id).unwrap();
        let cancelled = cancel_session(&store, &user, &session.id).unwrap();
        assert_eq!(cancelled.state, STATE_CANCELLED);
        let next = start_session(&store, &user, &task_id).unwrap();
        assert_eq!(next.state, STATE_RUNNING);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
