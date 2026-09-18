use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Local development mail sink. Every outgoing message is written as one JSON
/// file plus an append-only `outbox.jsonl` so the host replay can consume the
/// verification message by script. No provider credentials are involved and
/// message bodies contain no secrets beyond the single-use action link.
#[derive(Debug, Serialize, Clone, serde::Deserialize)]
pub struct MailMessage {
    pub id: String,
    pub to: String,
    pub subject: String,
    pub body_text: String,
    pub action_url: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct MailSink {
    dir: PathBuf,
}

impl MailSink {
    pub fn new(base_dir: &std::path::Path) -> Self {
        MailSink {
            dir: base_dir.join("mail-sink"),
        }
    }

    /// Snapshot of every stored message. Callers must reduce the result to
    /// booleans or outcomes before anything crosses IPC — message bodies and
    /// action links carry the single-use tokens.
    pub fn read_messages(&self) -> Vec<MailMessage> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut messages = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(message) = serde_json::from_str::<MailMessage>(&raw) {
                    messages.push(message);
                }
            }
        }
        messages
    }

    pub fn deliver(&self, message: &MailMessage) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| format!("mail sink unavailable: {e}"))?;
        let file_name = format!("{}.json", message.id);
        let json = serde_json::to_string_pretty(message).map_err(|e| e.to_string())?;
        fs::write(self.dir.join(file_name), &json)
            .map_err(|e| format!("mail sink write failed: {e}"))?;
        let mut line = serde_json::to_string(message).map_err(|e| e.to_string())?;
        line.push('\n');
        let mut out = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("outbox.jsonl"))
            .map_err(|e| format!("mail sink unavailable: {e}"))?;
        out.write_all(line.as_bytes())
            .map_err(|e| format!("mail sink append failed: {e}"))?;
        Ok(())
    }
}

pub fn verification_message(to: &str, token: &str, verify_base: &str) -> MailMessage {
    let action_url = format!("{verify_base}/verify?token={token}");
    MailMessage {
        id: crate::store::new_id("mail"),
        to: to.to_string(),
        subject: "Verify your Focusboard email".to_string(),
        body_text: format!(
            "Welcome to Focusboard.\n\nConfirm this email address to activate your account:\n{action_url}\n\nThe link is single-use and expires in 24 hours. If you did not create an account, you can ignore this message."
        ),
        action_url,
        created_at: crate::store::now_rfc3339(),
    }
}

/// Reminder email: product identity, task title, due context, and a deep link
/// that opens the task in the signed-in app. The link carries only the task
/// id — never a token, credential, or private content beyond the task title.
pub fn reminder_message(
    to: &str,
    task_id: &str,
    task_title: &str,
    due_date: Option<&str>,
    local_due: &str,
) -> MailMessage {
    let action_url = format!("focusboard://task?id={task_id}");
    let due_context = match due_date {
        Some(date) => format!("Task due {date}. Reminder set for {local_due}."),
        None => format!("Reminder set for {local_due}."),
    };
    MailMessage {
        id: crate::store::new_id("mail"),
        to: to.to_string(),
        subject: format!("Focusboard reminder: {task_title}"),
        body_text: format!(
            "Focusboard reminder\n\nTask: {task_title}\n{due_context}\n\nOpen the task in Focusboard:\n{action_url}\n\nManage reminder preferences in Focusboard Settings."
        ),
        action_url,
        created_at: crate::store::now_rfc3339(),
    }
}
