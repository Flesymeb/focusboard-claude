use crate::auth::PublicUser;
use crate::store::{new_id, now_rfc3339, CommandError, CommandResult, Store};
use serde::Serialize;

/// One state-changing action on a task or project. Rows are append-only:
/// history is never rewritten, so the per-task list reads exactly what
/// happened, in order.
#[derive(Debug, Serialize, Clone)]
pub struct ActivityEvent {
    pub id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub event_type: String,
    pub summary: String,
    pub created_at: String,
}

pub fn record(
    store: &Store,
    user_id: &str,
    entity_id: &str,
    entity_type: &str,
    event_type: &str,
    summary: &str,
) -> CommandResult<()> {
    store
        .conn
        .execute(
            "INSERT INTO activity_events(id, user_id, entity_id, entity_type, event_type, summary, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                new_id("act"),
                user_id,
                entity_id,
                entity_type,
                event_type,
                summary,
                now_rfc3339()
            ],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not record activity: {e}")))?;
    Ok(())
}

/// Concise per-task history: newest last so the UI reads like a timeline.
pub fn list_for_task(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
) -> CommandResult<Vec<ActivityEvent>> {
    let mut stmt = store
        .conn
        .prepare(
            "SELECT id, entity_id, entity_type, event_type, summary, created_at
             FROM activity_events
             WHERE user_id = ?1 AND entity_id = ?2 AND entity_type = 'task'
             ORDER BY created_at, id",
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not read activity: {e}")))?;
    let rows = stmt
        .query_map(rusqlite::params![user.id, task_id], |r| {
            Ok(ActivityEvent {
                id: r.get(0)?,
                entity_id: r.get(1)?,
                entity_type: r.get(2)?,
                event_type: r.get(3)?,
                summary: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(|e| CommandError::new("storage_error", format!("Could not read activity: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CommandError::new("storage_error", format!("Could not read activity: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("focusboard-act-{name}-{}", new_id("t")));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn user() -> PublicUser {
        PublicUser {
            id: "u1".to_string(),
            email: "ada@example.com".to_string(),
            display_name: "Ada".to_string(),
            timezone: "UTC".to_string(),
            status: "active".to_string(),
        }
    }

    fn seeded_store(dir: &std::path::Path) -> Store {
        let store = Store::open(&dir.join("focusboard.sqlite3")).unwrap();
        store
            .conn
            .execute(
                "INSERT INTO users(id, email, password_hash, display_name, status, created_at)
                 VALUES('u1', 'ada@example.com', 'hash', 'Ada', 'active', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
        store
            .conn
            .execute(
                "INSERT INTO users(id, email, password_hash, display_name, status, created_at)
                 VALUES('u2', 'grace@example.com', 'hash', 'Grace', 'active', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
        store
    }

    #[test]
    fn events_append_in_order_and_scope_to_task() {
        let dir = temp_dir("append");
        let store = seeded_store(&dir);
        let u = user();
        record(&store, "u1", "t1", "task", "created", "Created").unwrap();
        record(&store, "u1", "t1", "task", "completed", "Marked complete").unwrap();
        record(&store, "u1", "t1", "task", "reopened", "Reopened").unwrap();
        // Another user's events and other entities never leak in.
        record(&store, "u2", "t1", "task", "created", "Someone else").unwrap();
        record(&store, "u1", "prj_1", "project", "archived", "Archived").unwrap();

        let events = list_for_task(&store, &u, "t1").unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event_type, "created");
        assert_eq!(events[2].event_type, "reopened");
        assert!(events.iter().all(|e| e.entity_id == "t1"));
        assert!(events.iter().all(|e| e.entity_type == "task"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn events_survive_reopen() {
        let dir = temp_dir("reopen");
        let db = dir.join("focusboard.sqlite3");
        {
            let store = Store::open(&db).unwrap();
            store
                .conn
                .execute(
                    "INSERT INTO users(id, email, password_hash, display_name, status, created_at)
                     VALUES('u1', 'ada@example.com', 'hash', 'Ada', 'active', '2026-01-01T00:00:00Z')",
                    [],
                )
                .unwrap();
            record(&store, "u1", "t1", "task", "created", "Created").unwrap();
        }
        let reopened = Store::open(&db).unwrap();
        let events = list_for_task(&reopened, &user(), "t1").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Created");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
