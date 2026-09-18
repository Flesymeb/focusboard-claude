use crate::auth::PublicUser;
use crate::store::{new_id, now_rfc3339, CommandError, CommandResult, Store};
use rusqlite::OptionalExtension;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub accent: String,
    pub archived: bool,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Task {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub notes: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

const ACCENTS: [&str; 6] = ["amber", "olive", "clay", "slate", "moss", "plum"];

fn validate_project_name(name: &str) -> Result<String, CommandError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new("empty_name", "Give the project a name."));
    }
    if trimmed.len() > 80 {
        return Err(CommandError::new(
            "name_too_long",
            "Keep project names under 80 characters.",
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_due_date(due: &str) -> Result<String, CommandError> {
    chrono::NaiveDate::parse_from_str(due, "%Y-%m-%d")
        .map(|_| due.to_string())
        .map_err(|_| {
            CommandError::new(
                "invalid_due_date",
                "Use a valid date in YYYY-MM-DD format.",
            )
        })
}

fn next_accent(existing: usize) -> &'static str {
    ACCENTS[existing % ACCENTS.len()]
}

pub fn create_project(store: &Store, user: &PublicUser, name: &str) -> CommandResult<Project> {
    let name = validate_project_name(name)?;
    let count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE user_id = ?1",
            [&user.id],
            |r| r.get(0),
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not count projects: {e}")))?;
    let id = new_id("prj");
    let now = now_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO projects(id, user_id, name, accent, sort_order, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![id, user.id, name, next_accent(count as usize), count, now],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not create project: {e}")))?;
    Ok(Project {
        id,
        name,
        description: String::new(),
        accent: next_accent(count as usize).to_string(),
        archived: false,
        sort_order: count,
        created_at: now.clone(),
    })
}

pub fn rename_project(
    store: &Store,
    user: &PublicUser,
    project_id: &str,
    name: &str,
) -> CommandResult<Project> {
    let name = validate_project_name(name)?;
    let n = store
        .conn
        .execute(
            "UPDATE projects SET name = ?1, updated_at = ?2
             WHERE id = ?3 AND user_id = ?4 AND archived = 0",
            rusqlite::params![name, now_rfc3339(), project_id, user.id],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not rename project: {e}")))?;
    if n == 0 {
        return Err(CommandError::new("not_found", "That project no longer exists."));
    }
    get_project(store, user, project_id)
}

fn get_project(store: &Store, user: &PublicUser, project_id: &str) -> CommandResult<Project> {
    store
        .conn
        .query_row(
            "SELECT id, name, description, accent, archived, sort_order, created_at
             FROM projects WHERE id = ?1 AND user_id = ?2",
            rusqlite::params![project_id, user.id],
            |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    description: r.get(2)?,
                    accent: r.get(3)?,
                    archived: r.get::<_, i64>(4)? != 0,
                    sort_order: r.get(5)?,
                    created_at: r.get(6)?,
                })
            },
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Project lookup failed: {e}")))?
        .ok_or_else(|| CommandError::new("not_found", "That project no longer exists."))
}

pub fn list_projects(store: &Store, user: &PublicUser) -> CommandResult<Vec<Project>> {
    let mut stmt = store
        .conn
        .prepare(
            "SELECT id, name, description, accent, archived, sort_order, created_at
             FROM projects WHERE user_id = ?1 AND archived = 0 ORDER BY sort_order, created_at",
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not list projects: {e}")))?;
    let rows = stmt
        .query_map([&user.id], |r| {
            Ok(Project {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                accent: r.get(3)?,
                archived: r.get::<_, i64>(4)? != 0,
                sort_order: r.get(5)?,
                created_at: r.get(6)?,
            })
        })
        .map_err(|e| CommandError::new("storage_error", format!("Could not list projects: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CommandError::new("storage_error", format!("Could not read projects: {e}")))
}

fn validate_task_title(title: &str) -> Result<String, CommandError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new(
            "empty_title",
            "A task needs a title before it can be added.",
        ));
    }
    if trimmed.len() > 200 {
        return Err(CommandError::new(
            "title_too_long",
            "Keep task titles under 200 characters.",
        ));
    }
    Ok(trimmed.to_string())
}

fn assert_project_owned(store: &Store, user: &PublicUser, project_id: &str) -> CommandResult<()> {
    let owned: Option<i64> = store
        .conn
        .query_row(
            "SELECT 1 FROM projects WHERE id = ?1 AND user_id = ?2",
            rusqlite::params![project_id, user.id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Project lookup failed: {e}")))?;
    if owned.is_none() {
        return Err(CommandError::new("not_found", "That project no longer exists."));
    }
    Ok(())
}

pub fn create_task(
    store: &Store,
    user: &PublicUser,
    title: &str,
    project_id: Option<&str>,
    due_date: Option<&str>,
) -> CommandResult<Task> {
    let title = validate_task_title(title)?;
    if let Some(pid) = project_id {
        assert_project_owned(store, user, pid)?;
    }
    let due = due_date.map(validate_due_date).transpose()?;
    let position: i64 = store
        .conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM tasks WHERE user_id = ?1",
            [&user.id],
            |r| r.get(0),
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not order task: {e}")))?;
    let id = new_id("tsk");
    let now = now_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO tasks(id, user_id, project_id, title, status, due_date, position, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, 'open', ?5, ?6, ?7, ?7)",
            rusqlite::params![id, user.id, project_id, title, due, position, now],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not create task: {e}")))?;
    get_task(store, user, &id)
}

fn get_task(store: &Store, user: &PublicUser, task_id: &str) -> CommandResult<Task> {
    get_task_owned(store, user, task_id)
}

/// Ownership-checked task lookup shared with the reminder and focus modules.
pub fn get_task_owned(store: &Store, user: &PublicUser, task_id: &str) -> CommandResult<Task> {
    store
        .conn
        .query_row(
            "SELECT id, project_id, title, notes, status, priority, due_date, completed_at, created_at, updated_at
             FROM tasks WHERE id = ?1 AND user_id = ?2",
            rusqlite::params![task_id, user.id],
            |r| {
                Ok(Task {
                    id: r.get(0)?,
                    project_id: r.get(1)?,
                    title: r.get(2)?,
                    notes: r.get(3)?,
                    status: r.get(4)?,
                    priority: r.get(5)?,
                    due_date: r.get(6)?,
                    completed_at: r.get(7)?,
                    created_at: r.get(8)?,
                    updated_at: r.get(9)?,
                })
            },
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Task lookup failed: {e}")))?
        .ok_or_else(|| CommandError::new("not_found", "That task no longer exists."))
}

pub fn list_tasks(store: &Store, user: &PublicUser) -> CommandResult<Vec<Task>> {
    let mut stmt = store
        .conn
        .prepare(
            "SELECT id, project_id, title, notes, status, priority, due_date, completed_at, created_at, updated_at
             FROM tasks WHERE user_id = ?1
             ORDER BY status = 'completed', position, created_at",
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not list tasks: {e}")))?;
    let rows = stmt
        .query_map([&user.id], |r| {
            Ok(Task {
                id: r.get(0)?,
                project_id: r.get(1)?,
                title: r.get(2)?,
                notes: r.get(3)?,
                status: r.get(4)?,
                priority: r.get(5)?,
                due_date: r.get(6)?,
                completed_at: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
            })
        })
        .map_err(|e| CommandError::new("storage_error", format!("Could not list tasks: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CommandError::new("storage_error", format!("Could not read tasks: {e}")))
}

pub fn update_task(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    project_id: Option<Option<&str>>,
    due_date: Option<Option<&str>>,
    status: Option<&str>,
) -> CommandResult<Task> {
    // Ownership check first: never leak other users' rows through UPDATE.
    get_task(store, user, task_id)?;
    if let Some(pid) = project_id
        .as_ref()
        .and_then(|p| p.as_deref())
        .filter(|p| !p.is_empty())
    {
        assert_project_owned(store, user, pid)?;
    }
    let due = match due_date {
        Some(Some(d)) => Some(Some(validate_due_date(d)?)),
        Some(None) => Some(None),
        None => None,
    };
    let now = now_rfc3339();
    if let Some(pid) = project_id {
        let value: Option<&str> = if pid.unwrap_or_default().is_empty() {
            None
        } else {
            pid
        };
        store
            .conn
            .execute(
                "UPDATE tasks SET project_id = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
                rusqlite::params![value, now, task_id, user.id],
            )
            .map_err(|e| CommandError::new("storage_error", format!("Could not assign task: {e}")))?;
    }
    if let Some(d) = due {
        store
            .conn
            .execute(
                "UPDATE tasks SET due_date = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
                rusqlite::params![d, now, task_id, user.id],
            )
            .map_err(|e| CommandError::new("storage_error", format!("Could not schedule task: {e}")))?;
    }
    if let Some(s) = status {
        if s != "open" && s != "completed" {
            return Err(CommandError::new("invalid_status", "Unknown task status."));
        }
        let completed_at = if s == "completed" { Some(now.clone()) } else { None };
        // Re-completing an already completed task keeps the original
        // completion timestamp: completion is recorded exactly once.
        store
            .conn
            .execute(
                "UPDATE tasks SET status = ?1,
                    completed_at = CASE WHEN status = ?1 THEN completed_at ELSE ?2 END,
                    updated_at = ?3
                 WHERE id = ?4 AND user_id = ?5",
                rusqlite::params![s, completed_at, now, task_id, user.id],
            )
            .map_err(|e| CommandError::new("storage_error", format!("Could not update task: {e}")))?;
        if s == "completed" {
            // PRD section 7: completing a task skips its future reminders.
            crate::reminders::cancel_pending_for_task(store, user, task_id, "skipped")?;
        }
    }
    get_task(store, user, task_id)
}
