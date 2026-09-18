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

const SCHEMA_VERSION: i64 = 3;

const SCHEMA_V1: &str = "
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
    );";

/// v2 adds the reminder and focus-session domain plus user notification
/// preference. Existing databases upgrade in place; reminders and sessions
/// are restart-safe because every row lives in SQLite.
const SCHEMA_V2: &str = "
    ALTER TABLE users ADD COLUMN notifications_enabled INTEGER NOT NULL DEFAULT 1;
    CREATE TABLE reminders (
        id TEXT PRIMARY KEY,
        task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        scheduled_at TEXT NOT NULL,
        timezone TEXT NOT NULL,
        channel TEXT NOT NULL DEFAULT 'email',
        status TEXT NOT NULL DEFAULT 'scheduled',
        provider_message_id TEXT,
        sent_at TEXT,
        retry_count INTEGER NOT NULL DEFAULT 0,
        dedup_key TEXT NOT NULL UNIQUE
    );
    CREATE INDEX IF NOT EXISTS idx_reminders_due ON reminders(status, scheduled_at);
    CREATE INDEX IF NOT EXISTS idx_reminders_task ON reminders(task_id);
    CREATE TABLE focus_sessions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
        started_at TEXT NOT NULL,
        pauses TEXT NOT NULL DEFAULT '[]',
        ended_at TEXT,
        state TEXT NOT NULL DEFAULT 'running',
        updated_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_focus_user ON focus_sessions(user_id, state);
    CREATE TABLE activity_events (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        entity_id TEXT NOT NULL,
        entity_type TEXT NOT NULL,
        event_type TEXT NOT NULL,
        summary TEXT NOT NULL,
        created_at TEXT NOT NULL
    );";

/// v3 adds the per-email auth-request log used to rate-limit password-reset
/// and verification-email endpoints. It records email shape and request kind
/// only — never tokens, credentials, or whether an address is registered.
const SCHEMA_V3: &str = "
    CREATE TABLE auth_requests (
        email TEXT NOT NULL COLLATE NOCASE,
        kind TEXT NOT NULL,
        requested_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_auth_requests
        ON auth_requests(email, kind, requested_at);";

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
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < 1 {
            self.conn
                .execute_batch(&format!("BEGIN; {SCHEMA_V1} COMMIT;"))?;
        }
        if version < 2 {
            self.conn
                .execute_batch(&format!("BEGIN; {SCHEMA_V2} COMMIT;"))?;
        }
        if version < 3 {
            self.conn
                .execute_batch(&format!("BEGIN; {SCHEMA_V3} COMMIT;"))?;
        }
        if version < SCHEMA_VERSION {
            self.conn
                .pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        Ok(())
    }

    pub fn kv_get(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM app_kv WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .ok()
    }

    pub fn kv_set(&self, key: &str, value: &str) -> Result<(), CommandError> {
        self.conn
            .execute(
                "INSERT INTO app_kv(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [key, value],
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Failed to persist state: {e}"))
            })?;
        Ok(())
    }

    pub fn kv_delete(&self, key: &str) -> Result<(), CommandError> {
        self.conn
            .execute("DELETE FROM app_kv WHERE key = ?1", [key])
            .map_err(|e| {
                CommandError::new("storage_error", format!("Failed to clear state: {e}"))
            })?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("focusboard-store-{name}-{}", new_id("t")));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn user_version(conn: &Connection) -> i64 {
        conn.query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn fresh_database_opens_at_current_schema() {
        let dir = temp_dir("fresh");
        let store = Store::open(&dir.join("focusboard.sqlite3")).unwrap();
        assert_eq!(user_version(&store.conn), SCHEMA_VERSION);
        for table in ["reminders", "focus_sessions", "activity_events"] {
            let n: i64 = store
                .conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 0, "{table} must exist and start empty");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn existing_v1_database_upgrades_in_place_without_data_loss() {
        let dir = temp_dir("upgrade-v1");
        let db = dir.join("focusboard.sqlite3");
        {
            // Build a genuine v1 database with pre-existing user data.
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(SCHEMA_V1).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
            conn.execute(
                "INSERT INTO users(id, email, password_hash, display_name, timezone, locale, status, created_at)
                 VALUES('u1', 'ada@example.com', 'hash', 'Ada', 'UTC', 'en', 'active', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO tasks(id, user_id, title, status, position, created_at, updated_at)
                 VALUES('t1', 'u1', 'Prepare first demo', 'open', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
        }
        let store = Store::open(&db).unwrap();
        assert_eq!(user_version(&store.conn), SCHEMA_VERSION);

        // The v2 domain tables exist and accept rows for the migrated data.
        store
            .conn
            .execute(
                "INSERT INTO reminders(id, task_id, user_id, scheduled_at, timezone, dedup_key)
                 VALUES('r1', 't1', 'u1', '2026-09-18T00:00:00Z', 'UTC', 't1:2026-09-18T00:00:00Z')",
                [],
            )
            .unwrap();
        store
            .conn
            .execute(
                "INSERT INTO focus_sessions(id, user_id, started_at, state, updated_at)
                 VALUES('f1', 'u1', '2026-01-01T00:00:00Z', 'running', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();

        // Legacy rows survive with sensible defaults for new columns.
        let title: String = store
            .conn
            .query_row("SELECT title FROM tasks WHERE id = 't1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Prepare first demo");
        let notify: i64 = store
            .conn
            .query_row(
                "SELECT notifications_enabled FROM users WHERE id = 'u1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(notify, 1);
        drop(store);

        // Reopening an already-migrated database is a no-op, not a re-run.
        let reopened = Store::open(&db).unwrap();
        assert_eq!(user_version(&reopened.conn), SCHEMA_VERSION);
        let reminders: i64 = reopened
            .conn
            .query_row("SELECT COUNT(*) FROM reminders", [], |r| r.get(0))
            .unwrap();
        assert_eq!(reminders, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
