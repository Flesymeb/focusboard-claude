use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Local development mail sink. Every outgoing message is written as one JSON
/// file plus an append-only `outbox.jsonl` so the host replay can consume the
/// verification message by script. No provider credentials are involved and
/// message bodies contain no secrets beyond the single-use action link.
#[derive(Debug, Serialize, Clone)]
pub struct MailMessage {
    pub id: String,
    pub to: String,
    pub subject: String,
    pub body_text: String,
    pub action_url: String,
    pub created_at: String,
}

pub struct MailSink {
    dir: PathBuf,
}

impl MailSink {
    pub fn new(base_dir: &std::path::Path) -> Self {
        MailSink {
            dir: base_dir.join("mail-sink"),
        }
    }

    pub fn deliver(&self, message: &MailMessage) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| format!("mail sink unavailable: {e}"))?;
        let file_name = format!("{}.json", message.id);
        let json = serde_json::to_string_pretty(message).map_err(|e| e.to_string())?;
        fs::write(self.dir.join(file_name), &json).map_err(|e| format!("mail sink write failed: {e}"))?;
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
