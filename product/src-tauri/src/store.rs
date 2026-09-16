use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;

/// Error surfaced across the IPC boundary. `code` is stable and machine-readable;
/// `message` is user-facing and must never contain secrets, tokens, or passwords.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;

pub struct Store {
    pub conn: Connection,
}

const SCHEMA_VERSION: i64 = 1;

impl Store {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let store = Store { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), rusqlite::Error> {
        let version: i64 = self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version >= SCHEMA_VERSION {
            return Ok(());
        }
        self.conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE COLLATE NOCASE,
                password_hash TEXT NOT NULL,
                display_name TEXT NOT NULL DEFAULT '',
                timezone TEXT NOT NULL DEFAULT 'UTC',
                locale TEXT NOT NULL DEFAULT 'en',
                status TEXT NOT NULL DEFAULT 'unverified',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                revoked INTEGER NOT NULL DEFAULT 0,
                device TEXT NOT NULL DEFAULT 'desktop',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS email_tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                purpose TEXT NOT NULL,
                token_hash TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                consumed_at TEXT
            );
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                accent TEXT NOT NULL DEFAULT 'amber',
                archived INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
                title TEXT NOT NULL,
                notes TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'open',
                priority TEXT NOT NULL DEFAULT 'normal',
                due_date TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                completed_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_user ON tasks(user_id, status);
            CREATE INDEX IF NOT EXISTS idx_projects_user ON projects(user_id, archived);
            CREATE TABLE IF NOT EXISTS auth_attempts (
                email TEXT NOT NULL COLLATE NOCASE,
                attempted_at TEXT NOT NULL,
                success INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_attempts ON auth_attempts(email, attempted_at);
            CREATE TABLE IF NOT EXISTS app_kv (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            COMMIT;",
        )?;
        self.conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        Ok(())
    }

    pub fn kv_get(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM app_kv WHERE key = ?1",
                [key],
                |r| r.get(0),
            )
            .ok()
    }

    pub fn kv_set(&self, key: &str, value: &str) -> Result<(), CommandError> {
        self.conn
            .execute(
                "INSERT INTO app_kv(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [key, value],
            )
            .map_err(|e| CommandError::new("storage_error", format!("Failed to persist state: {e}")))?;
        Ok(())
    }

    pub fn kv_delete(&self, key: &str) -> Result<(), CommandError> {
        self.conn
            .execute("DELETE FROM app_kv WHERE key = ?1", [key])
            .map_err(|e| CommandError::new("storage_error", format!("Failed to clear state: {e}")))?;
        Ok(())
    }
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn new_id(prefix: &str) -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("{prefix}_{hex}")
}
