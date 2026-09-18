//! Product-owned golden-path drive support. Credentials for the driven
//! account are generated and held inside this module so no password or
//! verification token ever crosses IPC, reaches the DOM, or lands in a log;
//! the renderer only ever sees step outcomes.

use crate::auth;
use crate::mail::MailMessage;
use crate::store::{CommandError, CommandResult};
use rand::RngCore;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// Holds the driven account's credentials. Backend-only by construction:
/// the renderer triggers steps by name and reads back outcomes.
pub struct GoldenPathDrive(Mutex<Option<DriveCredentials>>);

struct DriveCredentials {
    email: String,
    password: String,
    task_title: String,
}

impl GoldenPathDrive {
    pub fn new() -> Self {
        GoldenPathDrive(Mutex::new(None))
    }
}

fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    RngCore::fill_bytes(&mut rand::thread_rng(), &mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn generate_credentials() -> DriveCredentials {
    DriveCredentials {
        email: format!("drive-{}@example.com", random_hex(8)),
        // The literal digit and letters keep the generated secret within the
        // product's own password policy without weakening it.
        password: format!("gp-{}-7{}", random_hex(6), random_hex(6)),
        // Neutral display copy: visible shell text must stay free of internal
        // diagnostic phrasing (the replay's UI-separation scan forbids it).
        task_title: "Deep work demo task".to_string(),
    }
}

#[derive(Serialize)]
pub struct DriveStarted {
    pub started: bool,
}

#[tauri::command]
pub fn golden_path_drive_start(
    drive: State<GoldenPathDrive>,
) -> Result<DriveStarted, CommandError> {
    let mut guard = drive
        .0
        .lock()
        .map_err(|_| CommandError::new("internal_error", "Drive state is unavailable."))?;
    *guard = Some(generate_credentials());
    Ok(DriveStarted { started: true })
}

fn with_drive<T, F>(drive: &GoldenPathDrive, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&DriveCredentials) -> CommandResult<T>,
{
    let guard = drive
        .0
        .lock()
        .map_err(|_| CommandError::new("internal_error", "Drive state is unavailable."))?;
    let creds = guard.as_ref().ok_or_else(|| {
        CommandError::new("drive_not_started", "Start the golden-path drive first.")
    })?;
    f(creds)
}

#[tauri::command]
pub fn golden_path_drive_register(
    state: State<crate::AppState>,
    drive: State<GoldenPathDrive>,
) -> Result<String, CommandError> {
    with_drive(&drive, |c| {
        crate::map_result(&state, |s| {
            auth::register(
                s,
                &state.sink,
                &state.verify_base,
                &c.email,
                &c.password,
                "Focus Demo",
            )
        })
    })
}

/// Finds the newest verification message for the driven account in the local
/// mail sink and consumes its token through the real verify path. The token
/// value never leaves this function.
#[tauri::command]
pub fn golden_path_drive_consume_verification(
    state: State<crate::AppState>,
    drive: State<GoldenPathDrive>,
) -> Result<(), CommandError> {
    let token = with_drive(&drive, |c| {
        let sink_messages = state.sink.read_messages();
        newest_verification_token(&sink_messages, &c.email)
    })?;
    crate::map_result(&state, |s| auth::verify_email_token(s, &token).map(|_| ()))
}

fn newest_verification_token(messages: &[MailMessage], email: &str) -> CommandResult<String> {
    let message = messages
        .iter()
        .filter(|m| m.to == email && crate::token_from_link(&m.action_url).is_some())
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
        .ok_or_else(|| {
            CommandError::new(
                "mail_not_found",
                "No verification message has arrived for the driven account yet.",
            )
        })?;
    crate::token_from_link(&message.action_url).ok_or_else(|| {
        CommandError::new(
            "invalid_link",
            "The verification message carries no usable activation link.",
        )
    })
}

#[tauri::command]
pub fn golden_path_drive_sign_in(
    state: State<crate::AppState>,
    drive: State<GoldenPathDrive>,
) -> Result<auth::PublicUser, CommandError> {
    let (email, password) = with_drive(&drive, |c| Ok((c.email.clone(), c.password.clone())))?;
    crate::map_result(&state, |s| auth::sign_in(s, &email, &password))
}

#[derive(Serialize)]
pub struct ReminderEmailCheck {
    pub addressed_to_drive_account: bool,
    pub names_the_task: bool,
    pub carries_product_identity: bool,
    pub links_back_safely: bool,
    pub free_of_tokens_and_passwords: bool,
}

/// Private-content gate for the reminder email: every dimension is checked
/// backend-side and only the boolean report crosses IPC.
#[tauri::command]
pub fn golden_path_check_reminder_email(
    state: State<crate::AppState>,
    drive: State<GoldenPathDrive>,
) -> Result<ReminderEmailCheck, CommandError> {
    let (email, task_title) = with_drive(&drive, |c| Ok((c.email.clone(), c.task_title.clone())))?;
    crate::map_result(&state, |_s| {
        let messages: Vec<MailMessage> = state
            .sink
            .read_messages()
            .into_iter()
            .filter(|m| m.to == email)
            .collect();
        Ok(check_reminder_email(&messages, &email, &task_title))
    })
}

fn check_reminder_email(
    messages: &[MailMessage],
    email: &str,
    task_title: &str,
) -> ReminderEmailCheck {
    let reminder = messages
        .iter()
        .filter(|m| m.subject.starts_with("Focusboard reminder:"))
        .max_by(|a, b| a.created_at.cmp(&b.created_at));
    let Some(message) = reminder else {
        return ReminderEmailCheck {
            addressed_to_drive_account: false,
            names_the_task: false,
            carries_product_identity: false,
            links_back_safely: false,
            free_of_tokens_and_passwords: false,
        };
    };
    let raw_ok_link = message.action_url.starts_with("focusboard://task?id=");
    ReminderEmailCheck {
        addressed_to_drive_account: message.to == email,
        names_the_task: message.body_text.contains(task_title),
        carries_product_identity: message.body_text.contains("Focusboard"),
        links_back_safely: raw_ok_link,
        free_of_tokens_and_passwords: !message.body_text.contains("token=")
            && !message.body_text.to_lowercase().contains("password")
            && !message.subject.to_lowercase().contains("password"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::MailSink;
    use crate::store::new_id;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("focusboard-gp-{name}-{}", new_id("t")));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn drive_credentials_satisfy_account_rules() {
        for _ in 0..16 {
            let creds = generate_credentials();
            auth::validate_email(&creds.email).unwrap();
            auth::validate_password(&creds.password).unwrap();
            assert!(!creds.task_title.is_empty());
        }
    }

    /// Opt-in gate: a freshly managed drive holds no credentials, so every
    /// account-touching step refuses until golden_path_drive_start ran
    /// explicitly. Normal startup never creates accounts or consumes tokens.
    #[test]
    fn drive_refuses_every_step_until_explicitly_started() {
        let drive = GoldenPathDrive::new();
        let err = with_drive(&drive, |c| Ok(c.email.clone())).unwrap_err();
        assert_eq!(err.code, "drive_not_started");
        assert!(drive.0.lock().unwrap().is_none());
    }

    #[test]
    fn newest_verification_token_reads_the_latest_message() {
        let dir = temp_dir("token");
        let sink = MailSink::new(&dir);
        sink.deliver(&crate::mail::verification_message(
            "drive@example.com",
            "oldertoken",
            "focusboard://auth",
        ))
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        sink.deliver(&crate::mail::verification_message(
            "drive@example.com",
            "newertoken",
            "focusboard://auth",
        ))
        .unwrap();
        let messages = sink.read_messages();
        let token = newest_verification_token(&messages, "drive@example.com").unwrap();
        assert_eq!(token, "newertoken");
        let err = newest_verification_token(&messages, "other@example.com").unwrap_err();
        assert_eq!(err.code, "mail_not_found");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reminder_email_check_reports_dimensions_without_secrets() {
        let dir = temp_dir("check");
        let sink = MailSink::new(&dir);
        sink.deliver(&crate::mail::reminder_message(
            "drive@example.com",
            "tsk_1",
            "Deep work demo task",
            Some("2026-09-18"),
            "2026-09-18 09:00",
        ))
        .unwrap();
        let messages = sink.read_messages();
        let check = check_reminder_email(&messages, "drive@example.com", "Deep work demo task");
        assert!(check.addressed_to_drive_account);
        assert!(check.names_the_task);
        assert!(check.carries_product_identity);
        assert!(check.links_back_safely);
        assert!(check.free_of_tokens_and_passwords);

        let mut tampered = messages[0].clone();
        tampered.body_text = format!("{}\nverify?token=leaked", tampered.body_text);
        let bad = check_reminder_email(&[tampered], "drive@example.com", "Deep work demo task");
        assert!(!bad.free_of_tokens_and_passwords);

        let missing = check_reminder_email(&[], "drive@example.com", "Deep work demo task");
        assert!(!missing.addressed_to_drive_account);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
