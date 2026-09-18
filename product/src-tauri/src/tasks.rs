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

#[derive(Debug, Serialize, Clone)]
pub struct Subtask {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub done: bool,
    pub position: i64,
}

const PRIORITIES: [&str; 3] = ["low", "normal", "high"];

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
            CommandError::new("invalid_due_date", "Use a valid date in YYYY-MM-DD format.")
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
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not count projects: {e}"))
        })?;
    let id = new_id("prj");
    let now = now_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO projects(id, user_id, name, accent, sort_order, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![id, user.id, name, next_accent(count as usize), count, now],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not create project: {e}"))
        })?;
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
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not rename project: {e}"))
        })?;
    if n == 0 {
        return Err(CommandError::new(
            "not_found",
            "That project no longer exists.",
        ));
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
    list_projects_in(store, user, false)
}

/// Archived projects keep their tasks and history; only visibility changes.
pub fn list_archived_projects(store: &Store, user: &PublicUser) -> CommandResult<Vec<Project>> {
    list_projects_in(store, user, true)
}

fn list_projects_in(
    store: &Store,
    user: &PublicUser,
    archived: bool,
) -> CommandResult<Vec<Project>> {
    let mut stmt = store
        .conn
        .prepare(
            "SELECT id, name, description, accent, archived, sort_order, created_at
             FROM projects WHERE user_id = ?1 AND archived = ?2 ORDER BY sort_order, created_at",
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not list projects: {e}")))?;
    let rows = stmt
        .query_map(rusqlite::params![user.id, archived as i64], |r| {
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

/// Hides (or restores) a project. Tasks and their history are untouched, so a
/// restore returns the project exactly as it was.
pub fn set_project_archived(
    store: &Store,
    user: &PublicUser,
    project_id: &str,
    archived: bool,
) -> CommandResult<Project> {
    let n = store
        .conn
        .execute(
            "UPDATE projects SET archived = ?1, updated_at = ?2
             WHERE id = ?3 AND user_id = ?4",
            rusqlite::params![archived as i64, now_rfc3339(), project_id, user.id],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not update project: {e}"))
        })?;
    if n == 0 {
        return Err(CommandError::new(
            "not_found",
            "That project no longer exists.",
        ));
    }
    let project = get_project(store, user, project_id)?;
    crate::activity::record(
        store,
        &user.id,
        project_id,
        "project",
        if archived { "archived" } else { "restored" },
        if archived {
            format!("Archived project {}", project.name)
        } else {
            format!("Restored project {}", project.name)
        }
        .as_str(),
    )?;
    Ok(project)
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
        return Err(CommandError::new(
            "not_found",
            "That project no longer exists.",
        ));
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
    crate::activity::record(store, &user.id, &id, "task", "created", "Created task")?;
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
    let before = get_task(store, user, task_id)?;
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
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not assign task: {e}"))
            })?;
        if before.project_id.as_deref() != value {
            match value {
                Some(id) => {
                    let name: String = store
                        .conn
                        .query_row("SELECT name FROM projects WHERE id = ?1", [id], |r| {
                            r.get(0)
                        })
                        .unwrap_or_else(|_| id.to_string());
                    crate::activity::record(
                        store,
                        &user.id,
                        task_id,
                        "task",
                        "assigned",
                        &format!("Moved to project {name}"),
                    )?;
                }
                None => {
                    crate::activity::record(
                        store,
                        &user.id,
                        task_id,
                        "task",
                        "assigned",
                        "Removed from project",
                    )?;
                }
            }
        }
    }
    if let Some(d) = due {
        store
            .conn
            .execute(
                "UPDATE tasks SET due_date = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
                rusqlite::params![d, now, task_id, user.id],
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not schedule task: {e}"))
            })?;
        if before.due_date != d {
            let summary = match &d {
                Some(date) => format!("Due date set to {date}"),
                None => "Due date removed".to_string(),
            };
            crate::activity::record(store, &user.id, task_id, "task", "due", &summary)?;
        }
    }
    if let Some(s) = status {
        if s != "open" && s != "in_progress" && s != "completed" {
            return Err(CommandError::new("invalid_status", "Unknown task status."));
        }
        let completed_at = if s == "completed" {
            Some(now.clone())
        } else {
            None
        };
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
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not update task: {e}"))
            })?;
        if before.status != s {
            let (event_type, summary) = match s {
                "completed" => ("completed", "Marked complete"),
                "in_progress" => ("started", "Started progress"),
                _ => ("reopened", "Reopened"),
            };
            crate::activity::record(store, &user.id, task_id, "task", event_type, summary)?;
        }
        if s == "completed" {
            // PRD section 7: completing a task skips its future reminders.
            crate::reminders::cancel_pending_for_task(store, user, task_id, "skipped")?;
        }
    }
    get_task(store, user, task_id)
}

/// Sets a task's priority from the UI. The column already persists across
/// restarts; this adds the write path and its history entry.
pub fn set_task_priority(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    priority: &str,
) -> CommandResult<Task> {
    let priority = priority.trim();
    if !PRIORITIES.contains(&priority) {
        return Err(CommandError::new(
            "invalid_priority",
            "Priority must be low, normal, or high.",
        ));
    }
    let before = get_task(store, user, task_id)?;
    if before.priority == priority {
        return Ok(before);
    }
    store
        .conn
        .execute(
            "UPDATE tasks SET priority = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
            rusqlite::params![priority, now_rfc3339(), task_id, user.id],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not set priority: {e}")))?;
    crate::activity::record(
        store,
        &user.id,
        task_id,
        "task",
        "priority",
        &format!("Priority set to {priority}"),
    )?;
    get_task(store, user, task_id)
}

/// Filtered task query backing the Inbox. Every parameter is optional; the
/// query is always parameterized so a search term can never alter its shape.
pub fn query_tasks(
    store: &Store,
    user: &PublicUser,
    search: Option<&str>,
    priority: Option<&str>,
    due: Option<&str>,
    status: Option<&str>,
    today: Option<&str>,
) -> CommandResult<Vec<Task>> {
    if let Some(p) = priority {
        if !PRIORITIES.contains(&p) {
            return Err(CommandError::new(
                "invalid_priority",
                "Priority must be low, normal, or high.",
            ));
        }
    }
    if let Some(s) = status {
        if s != "open" && s != "completed" && s != "in_progress" {
            return Err(CommandError::new("invalid_status", "Unknown task status."));
        }
    }
    let mut clauses: Vec<String> = vec!["user_id = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(user.id.clone())];
    let trimmed = search.map(str::trim).filter(|s| !s.is_empty());
    if let Some(term) = trimmed {
        params.push(Box::new(format!("%{}%", escape_like(term))));
        clauses.push(format!("title LIKE ?{} ESCAPE '\\'", params.len()));
    }
    if let Some(p) = priority {
        params.push(Box::new(p.to_string()));
        clauses.push(format!("priority = ?{}", params.len()));
    }
    if let Some(s) = status {
        // "Open" means not finished yet, so in-progress work stays visible.
        match s {
            "open" => clauses.push("status != 'completed'".to_string()),
            "completed" => clauses.push("status = 'completed'".to_string()),
            _ => {
                params.push(Box::new(s.to_string()));
                clauses.push(format!("status = ?{}", params.len()));
            }
        }
    }
    if let Some(d) = due {
        match d {
            "any" => clauses.push("due_date IS NOT NULL".to_string()),
            "none" => clauses.push("due_date IS NULL".to_string()),
            "today" | "overdue" => {
                let day = today.ok_or_else(|| {
                    CommandError::new(
                        "invalid_due_filter",
                        "A due-date filter needs today's date.",
                    )
                })?;
                validate_due_date(day)?;
                params.push(Box::new(day.to_string()));
                let op = if d == "today" { "=" } else { "<" };
                clauses.push(format!("due_date {op} ?{}", params.len()));
            }
            _ => {
                return Err(CommandError::new(
                    "invalid_due_filter",
                    "Unknown due-date filter.",
                ))
            }
        }
    }
    let sql = format!(
        "SELECT id, project_id, title, notes, status, priority, due_date, completed_at, created_at, updated_at
         FROM tasks WHERE {}
         ORDER BY status = 'completed', position, created_at",
        clauses.join(" AND ")
    );
    let mut stmt = store
        .conn
        .prepare(&sql)
        .map_err(|e| CommandError::new("storage_error", format!("Could not query tasks: {e}")))?;
    let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt
        .query_map(refs.as_slice(), |r| {
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
        .map_err(|e| CommandError::new("storage_error", format!("Could not query tasks: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CommandError::new("storage_error", format!("Could not query tasks: {e}")))
}

/// Escapes LIKE metacharacters so a search term is matched literally.
fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn validate_subtask_title(title: &str) -> Result<String, CommandError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new(
            "empty_title",
            "A subtask needs a title before it can be added.",
        ));
    }
    if trimmed.len() > 200 {
        return Err(CommandError::new(
            "title_too_long",
            "Keep subtask titles under 200 characters.",
        ));
    }
    Ok(trimmed.to_string())
}

fn get_subtask(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    subtask_id: &str,
) -> CommandResult<Subtask> {
    store
        .conn
        .query_row(
            "SELECT id, task_id, title, done, position
             FROM subtasks WHERE id = ?1 AND task_id = ?2 AND user_id = ?3",
            rusqlite::params![subtask_id, task_id, user.id],
            |r| {
                Ok(Subtask {
                    id: r.get(0)?,
                    task_id: r.get(1)?,
                    title: r.get(2)?,
                    done: r.get::<_, i64>(3)? != 0,
                    position: r.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Subtask lookup failed: {e}")))?
        .ok_or_else(|| CommandError::new("not_found", "That subtask no longer exists."))
}

pub fn list_subtasks(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
) -> CommandResult<Vec<Subtask>> {
    get_task(store, user, task_id)?;
    let mut stmt = store
        .conn
        .prepare(
            "SELECT id, task_id, title, done, position
             FROM subtasks WHERE task_id = ?1 AND user_id = ?2
             ORDER BY position, created_at",
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not list subtasks: {e}")))?;
    let rows = stmt
        .query_map(rusqlite::params![task_id, user.id], |r| {
            Ok(Subtask {
                id: r.get(0)?,
                task_id: r.get(1)?,
                title: r.get(2)?,
                done: r.get::<_, i64>(3)? != 0,
                position: r.get(4)?,
            })
        })
        .map_err(|e| CommandError::new("storage_error", format!("Could not list subtasks: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| CommandError::new("storage_error", format!("Could not list subtasks: {e}")))
}

pub fn add_subtask(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    title: &str,
) -> CommandResult<Subtask> {
    get_task(store, user, task_id)?;
    let title = validate_subtask_title(title)?;
    let position: i64 = store
        .conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM subtasks WHERE task_id = ?1",
            [task_id],
            |r| r.get(0),
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not order subtask: {e}")))?;
    let id = new_id("sub");
    let now = now_rfc3339();
    store
        .conn
        .execute(
            "INSERT INTO subtasks(id, task_id, user_id, title, done, position, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, 0, ?5, ?6, ?6)",
            rusqlite::params![id, task_id, user.id, title, position, now],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not add subtask: {e}")))?;
    crate::activity::record(
        store,
        &user.id,
        task_id,
        "task",
        "subtask_added",
        &format!("Added subtask \"{title}\""),
    )?;
    get_subtask(store, user, task_id, &id)
}

pub fn set_subtask_done(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    subtask_id: &str,
    done: bool,
) -> CommandResult<Subtask> {
    let before = get_subtask(store, user, task_id, subtask_id)?;
    if before.done == done {
        return Ok(before);
    }
    store
        .conn
        .execute(
            "UPDATE subtasks SET done = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
            rusqlite::params![done as i64, now_rfc3339(), subtask_id, user.id],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not update subtask: {e}"))
        })?;
    let summary = if done {
        format!("Completed subtask \"{}\"", before.title)
    } else {
        format!("Reopened subtask \"{}\"", before.title)
    };
    crate::activity::record(
        store,
        &user.id,
        task_id,
        "task",
        "subtask_toggled",
        &summary,
    )?;
    get_subtask(store, user, task_id, subtask_id)
}

pub fn remove_subtask(
    store: &Store,
    user: &PublicUser,
    task_id: &str,
    subtask_id: &str,
) -> CommandResult<()> {
    let subtask = get_subtask(store, user, task_id, subtask_id)?;
    store
        .conn
        .execute(
            "DELETE FROM subtasks WHERE id = ?1 AND user_id = ?2",
            rusqlite::params![subtask_id, user.id],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not remove subtask: {e}"))
        })?;
    crate::activity::record(
        store,
        &user.id,
        task_id,
        "task",
        "subtask_removed",
        &format!("Removed subtask \"{}\"", subtask.title),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::new_id;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("focusboard-tasks-{name}-{}", new_id("t")));
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

    fn seeded_store(name: &str) -> Store {
        let dir = temp_dir(name);
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
    }

    fn history(store: &Store, task_id: &str) -> Vec<String> {
        crate::activity::list_for_task(store, &user(), task_id)
            .unwrap()
            .into_iter()
            .map(|e| e.event_type)
            .collect()
    }

    #[test]
    fn board_status_transitions_record_history_exactly_once() {
        let store = seeded_store("board");
        let u = user();
        let task = create_task(&store, &u, "Write the report", None, None).unwrap();

        let started = update_task(&store, &u, &task.id, None, None, Some("in_progress")).unwrap();
        assert_eq!(started.status, "in_progress");
        assert!(started.completed_at.is_none());

        let done = update_task(&store, &u, &task.id, None, None, Some("completed")).unwrap();
        assert_eq!(done.status, "completed");
        let completed_at = done.completed_at.clone().unwrap();

        // Re-completing is idempotent: the original completion time stands.
        let again = update_task(&store, &u, &task.id, None, None, Some("completed")).unwrap();
        assert_eq!(again.completed_at.as_deref(), Some(completed_at.as_str()));

        let reopened = update_task(&store, &u, &task.id, None, None, Some("open")).unwrap();
        assert_eq!(reopened.status, "open");
        assert!(reopened.completed_at.is_none());

        assert_eq!(
            history(&store, &task.id),
            vec!["created", "started", "completed", "reopened"]
        );
        assert!(update_task(&store, &u, &task.id, None, None, Some("archived")).is_err());
    }

    #[test]
    fn no_op_updates_do_not_append_history() {
        let store = seeded_store("noop");
        let u = user();
        let task = create_task(&store, &u, "Stable task", None, None).unwrap();
        update_task(&store, &u, &task.id, None, None, Some("open")).unwrap();
        update_task(&store, &u, &task.id, Some(Some("")), None, None).unwrap();
        assert_eq!(history(&store, &task.id), vec!["created"]);
    }

    #[test]
    fn priority_is_settable_and_recorded() {
        let store = seeded_store("priority");
        let u = user();
        let task = create_task(&store, &u, "Prioritized", None, None).unwrap();
        assert_eq!(task.priority, "normal");
        let high = set_task_priority(&store, &u, &task.id, "high").unwrap();
        assert_eq!(high.priority, "high");
        assert_eq!(
            set_task_priority(&store, &u, &task.id, "high")
                .unwrap()
                .priority,
            "high"
        );
        assert_eq!(history(&store, &task.id), vec!["created", "priority"]);
        assert!(set_task_priority(&store, &u, &task.id, "urgent").is_err());
    }

    #[test]
    fn subtasks_add_toggle_remove_and_persist_with_history() {
        let store = seeded_store("subtasks");
        let u = user();
        let task = create_task(&store, &u, "Parent task", None, None).unwrap();

        let first = add_subtask(&store, &u, &task.id, "Draft outline").unwrap();
        let second = add_subtask(&store, &u, &task.id, "Collect data").unwrap();
        assert_eq!(first.position, 0);
        assert_eq!(second.position, 1);

        let toggled = set_subtask_done(&store, &u, &task.id, &first.id, true).unwrap();
        assert!(toggled.done);
        assert_eq!(
            set_subtask_done(&store, &u, &task.id, &first.id, true)
                .unwrap()
                .done,
            true
        );

        remove_subtask(&store, &u, &task.id, &second.id).unwrap();
        let remaining = list_subtasks(&store, &u, &task.id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "Draft outline");

        assert_eq!(
            history(&store, &task.id),
            vec![
                "created",
                "subtask_added",
                "subtask_added",
                "subtask_toggled",
                "subtask_removed"
            ]
        );
        assert!(add_subtask(&store, &u, &task.id, "   ").is_err());
    }

    #[test]
    fn subtasks_are_scoped_to_their_task_and_owner() {
        let store = seeded_store("scoped");
        let u = user();
        let other = PublicUser {
            id: "u2".to_string(),
            email: "grace@example.com".to_string(),
            display_name: "Grace".to_string(),
            timezone: "UTC".to_string(),
            status: "active".to_string(),
        };
        store
            .conn
            .execute(
                "INSERT INTO users(id, email, password_hash, display_name, status, created_at)
                 VALUES('u2', 'grace@example.com', 'hash', 'Grace', 'active', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
        let task = create_task(&store, &u, "Owned task", None, None).unwrap();
        let sub = add_subtask(&store, &u, &task.id, "Private step").unwrap();
        assert!(list_subtasks(&store, &other, &task.id).is_err());
        assert!(set_subtask_done(&store, &other, &task.id, &sub.id, true).is_err());
        assert!(remove_subtask(&store, &other, &task.id, &sub.id).is_err());
        assert_eq!(list_subtasks(&store, &u, &task.id).unwrap().len(), 1);
    }

    #[test]
    fn archive_hides_and_restore_returns_a_project_with_tasks_intact() {
        let store = seeded_store("archive");
        let u = user();
        let project = create_project(&store, &u, "Website").unwrap();
        let task = create_task(&store, &u, "Design header", Some(&project.id), None).unwrap();

        set_project_archived(&store, &u, &project.id, true).unwrap();
        assert!(list_projects(&store, &u).unwrap().is_empty());
        let archived = list_archived_projects(&store, &u).unwrap();
        assert_eq!(archived.len(), 1);
        assert!(archived[0].archived);
        // Task rows are untouched while hidden.
        assert_eq!(list_tasks(&store, &u).unwrap().len(), 1);

        set_project_archived(&store, &u, &project.id, false).unwrap();
        let restored = list_projects(&store, &u).unwrap();
        assert_eq!(restored.len(), 1);
        assert!(!restored[0].archived);
        assert_eq!(restored[0].name, "Website");
        let tasks = list_tasks(&store, &u).unwrap();
        assert_eq!(tasks[0].id, task.id);
        assert_eq!(tasks[0].project_id.as_deref(), Some(project.id.as_str()));
        // Renaming an archived project stays blocked.
        set_project_archived(&store, &u, &project.id, true).unwrap();
        assert!(rename_project(&store, &u, &project.id, "Renamed").is_err());
        let _ = std::fs::remove_dir_all(temp_dir("archive"));
    }

    #[test]
    fn inbox_query_filters_narrow_by_search_priority_due_and_completion() {
        let store = seeded_store("query");
        let u = user();
        create_task(&store, &u, "Write proposal", None, Some("2026-09-19")).unwrap();
        create_task(&store, &u, "Review budget", None, Some("2026-09-01")).unwrap();
        let errand = create_task(&store, &u, "Buy printer paper", None, None).unwrap();
        set_task_priority(&store, &u, &errand.id, "high").unwrap();
        let done = create_task(&store, &u, "Archive old files", None, None).unwrap();
        update_task(&store, &u, &done.id, None, None, Some("completed")).unwrap();

        // Search narrows by title substring.
        let hits = query_tasks(&store, &u, Some("budget"), None, None, None, None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Review budget");

        // Priority filter returns only high-priority rows.
        let hits = query_tasks(&store, &u, None, Some("high"), None, None, None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, errand.id);

        // Due filters: overdue vs today vs no due date.
        let overdue = query_tasks(
            &store,
            &u,
            None,
            None,
            Some("overdue"),
            None,
            Some("2026-09-19"),
        )
        .unwrap();
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].title, "Review budget");
        let today = query_tasks(
            &store,
            &u,
            None,
            None,
            Some("today"),
            None,
            Some("2026-09-19"),
        )
        .unwrap();
        assert_eq!(today.len(), 1);
        assert_eq!(today[0].title, "Write proposal");
        let no_due = query_tasks(&store, &u, None, None, Some("none"), None, None).unwrap();
        assert_eq!(no_due.len(), 2);

        // Completion filter narrows to finished work.
        let completed = query_tasks(&store, &u, None, None, None, Some("completed"), None).unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, done.id);

        // "Open" includes in-progress work; only finished tasks are excluded.
        let started = create_task(&store, &u, "Board move in flight", None, None).unwrap();
        update_task(&store, &u, &started.id, None, None, Some("in_progress")).unwrap();
        let open = query_tasks(&store, &u, None, None, None, Some("open"), None).unwrap();
        assert!(open.iter().any(|t| t.id == started.id));
        assert!(!open.iter().any(|t| t.id == done.id));

        // Filters compose.
        let both = query_tasks(
            &store,
            &u,
            Some("budget"),
            None,
            Some("overdue"),
            Some("open"),
            Some("2026-09-19"),
        )
        .unwrap();
        assert_eq!(both.len(), 1);

        // LIKE metacharacters match literally instead of wildcarding.
        create_task(&store, &u, "Fix 100%_bugs", None, None).unwrap();
        let literal = query_tasks(&store, &u, Some("100%_bugs"), None, None, None, None).unwrap();
        assert_eq!(literal.len(), 1);

        assert!(query_tasks(&store, &u, None, Some("urgent"), None, None, None).is_err());
        assert!(query_tasks(&store, &u, None, None, Some("yesterday"), None, None).is_err());
        assert!(query_tasks(&store, &u, None, None, Some("today"), None, None).is_err());
    }

    #[test]
    fn due_and_assign_changes_are_recorded_in_history() {
        let store = seeded_store("due-assign");
        let u = user();
        let project = create_project(&store, &u, "Launch").unwrap();
        let task = create_task(&store, &u, "Prepare", None, None).unwrap();
        update_task(&store, &u, &task.id, None, Some(Some("2026-10-01")), None).unwrap();
        update_task(&store, &u, &task.id, Some(Some(&project.id)), None, None).unwrap();
        update_task(&store, &u, &task.id, Some(Some("")), None, None).unwrap();
        let events = crate::activity::list_for_task(&store, &u, &task.id).unwrap();
        let summaries: Vec<String> = events.iter().map(|e| e.summary.clone()).collect();
        assert!(summaries.contains(&"Due date set to 2026-10-01".to_string()));
        assert!(summaries.contains(&"Moved to project Launch".to_string()));
        assert!(summaries.contains(&"Removed from project".to_string()));
    }
}
