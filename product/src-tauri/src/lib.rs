mod auth;
mod mail;
mod store;
mod tasks;

use mail::MailSink;
use serde::Serialize;
use std::path::PathBuf;
use store::{CommandError, CommandResult, Store};
use tauri::Manager;

pub struct AppState {
    store: std::sync::Mutex<Store>,
    sink: MailSink,
    verify_base: String,
}

/// The verification link points back into the app's own verify route so the
/// single-use token is consumed by the desktop client, not a browser.
const VERIFY_BASE: &str = "focusboard://auth";

fn map_result<T, F>(state: &AppState, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&Store) -> CommandResult<T>,
{
    let guard = state
        .store
        .lock()
        .map_err(|_| CommandError::new("internal_error", "Internal state is unavailable."))?;
    f(&guard)
}

fn require_then<T, F>(state: &AppState, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&Store, &auth::PublicUser) -> CommandResult<T>,
{
    map_result(state, |s| {
        let user = auth::require_user(s)?;
        f(s, &user)
    })
}

#[derive(Serialize)]
struct RegisterOutcome {
    status: String,
    email: String,
}

#[tauri::command]
fn register(
    state: tauri::State<AppState>,
    email: String,
    password: String,
    display_name: String,
) -> Result<RegisterOutcome, CommandError> {
    let status = map_result(&state, |s| {
        auth::register(s, &state.sink, &state.verify_base, &email, &password, &display_name)
    })?;
    Ok(RegisterOutcome {
        status,
        email: email.trim().to_string(),
    })
}

#[tauri::command]
fn verify_email(state: tauri::State<AppState>, token: String) -> Result<auth::PublicUser, CommandError> {
    map_result(&state, |s| auth::verify_email_token(s, &token))
}

#[tauri::command]
fn sign_in(
    state: tauri::State<AppState>,
    email: String,
    password: String,
) -> Result<auth::PublicUser, CommandError> {
    map_result(&state, |s| auth::sign_in(s, &email, &password))
}

#[tauri::command]
fn sign_out(state: tauri::State<AppState>) -> Result<(), CommandError> {
    map_result(&state, auth::sign_out)
}

#[tauri::command]
fn session_status(state: tauri::State<AppState>) -> Result<Option<auth::PublicUser>, CommandError> {
    match map_result(&state, auth::current_user) {
        Ok(user) => Ok(Some(user)),
        Err(CommandError { code, .. })
            if code == "unauthenticated" || code == "session_expired" =>
        {
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn request_verification_email(
    state: tauri::State<AppState>,
    email: String,
) -> Result<(), CommandError> {
    map_result(&state, |s| {
        auth::request_verification_email(s, &state.sink, &state.verify_base, &email)
    })
}

#[tauri::command]
fn create_project(state: tauri::State<AppState>, name: String) -> Result<tasks::Project, CommandError> {
    require_then(&state, |s, u| tasks::create_project(s, u, &name))
}

#[tauri::command]
fn rename_project(
    state: tauri::State<AppState>,
    project_id: String,
    name: String,
) -> Result<tasks::Project, CommandError> {
    require_then(&state, |s, u| tasks::rename_project(s, u, &project_id, &name))
}

#[tauri::command]
fn list_projects(state: tauri::State<AppState>) -> Result<Vec<tasks::Project>, CommandError> {
    require_then(&state, |s, u| tasks::list_projects(s, u))
}

#[tauri::command]
fn create_task(
    state: tauri::State<AppState>,
    title: String,
    project_id: Option<String>,
    due_date: Option<String>,
) -> Result<tasks::Task, CommandError> {
    require_then(&state, |s, u| {
        tasks::create_task(s, u, &title, project_id.as_deref(), due_date.as_deref())
    })
}

#[tauri::command]
fn list_tasks(state: tauri::State<AppState>) -> Result<Vec<tasks::Task>, CommandError> {
    require_then(&state, |s, u| tasks::list_tasks(s, u))
}

#[tauri::command]
fn assign_task(
    state: tauri::State<AppState>,
    task_id: String,
    project_id: Option<String>,
) -> Result<tasks::Task, CommandError> {
    require_then(&state, |s, u| {
        tasks::update_task(s, u, &task_id, Some(project_id.as_deref()), None, None)
    })
}

#[tauri::command]
fn set_task_due(
    state: tauri::State<AppState>,
    task_id: String,
    due_date: Option<String>,
) -> Result<tasks::Task, CommandError> {
    require_then(&state, |s, u| {
        tasks::update_task(s, u, &task_id, None, Some(due_date.as_deref()), None)
    })
}

#[tauri::command]
fn set_task_status(
    state: tauri::State<AppState>,
    task_id: String,
    status: String,
) -> Result<tasks::Task, CommandError> {
    require_then(&state, |s, u| {
        tasks::update_task(s, u, &task_id, None, None, Some(&status))
    })
}

fn resolve_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, CommandError> {
    if let Ok(dir) = std::env::var("FOCUSBOARD_DATA_DIR") {
        if !dir.is_empty() {
            let path = PathBuf::from(dir);
            std::fs::create_dir_all(&path)
                .map_err(|e| CommandError::new("storage_error", format!("Data directory unavailable: {e}")))?;
            return Ok(path);
        }
    }
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| CommandError::new("storage_error", "Data directory could not be resolved."))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| CommandError::new("storage_error", format!("Data directory unavailable: {e}")))?;
    Ok(dir)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = resolve_data_dir(app.handle()).map_err(|e| e.message)?;
            let db_path = data_dir.join("focusboard.sqlite3");
            let store = Store::open(&db_path).map_err(|e| format!("Could not open database: {e}"))?;
            let sink = MailSink::new(&data_dir);
            app.manage(AppState {
                store: std::sync::Mutex::new(store),
                sink,
                verify_base: VERIFY_BASE.to_string(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            register,
            verify_email,
            sign_in,
            sign_out,
            session_status,
            request_verification_email,
            create_project,
            rename_project,
            list_projects,
            create_task,
            list_tasks,
            assign_task,
            set_task_due,
            set_task_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focusboard");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "focusboard-test-{name}-{}",
            store::new_id("t")
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn open_store(dir: &PathBuf) -> Store {
        Store::open(&dir.join("focusboard.sqlite3")).unwrap()
    }

    fn read_sink_token(dir: &PathBuf, to: &str) -> String {
        let sink_dir = dir.join("mail-sink");
        let mut last: Option<String> = None;
        for entry in std::fs::read_dir(&sink_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = std::fs::read_to_string(&path).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
                if parsed["to"].as_str() == Some(to) {
                    last = Some(raw);
                }
            }
        }
        let raw = last.expect("mail sink must contain the verification message");
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let url = parsed["action_url"].as_str().unwrap().to_string();
        let token = url.rsplit('=').next().unwrap().to_string();
        assert!(!token.is_empty());
        assert!(!raw.contains("password"), "mail must not contain passwords");
        token
    }

    #[test]
    fn register_verify_signin_quickadd_restart_persists() {
        let dir = temp_dir("full-path");
        let sink = MailSink::new(&dir);
        let verify_base = "focusboard://auth";

        // Pass 1: register -> verify -> sign in -> capture data.
        let user = {
            let store = open_store(&dir);
            let outcome =
                auth::register(&store, &sink, verify_base, "ada@example.com", "tulip-garnet-9", "Ada")
                    .unwrap();
            assert_eq!(outcome, "verification_sent");
            let token = read_sink_token(&dir, "ada@example.com");

            // Sign-in before verification must be rejected with the explicit
            // unverified state.
            let err = auth::sign_in(&store, "ada@example.com", "tulip-garnet-9").unwrap_err();
            assert_eq!(err.code, "unverified");

            let verified = auth::verify_email_token(&store, &token).unwrap();
            assert_eq!(verified.email, "ada@example.com");

            // Tokens are single-use.
            let err = auth::verify_email_token(&store, &token).unwrap_err();
            assert_eq!(err.code, "token_already_used");

            let signed_in = auth::sign_in(&store, "ada@example.com", "tulip-garnet-9").unwrap();
            assert_eq!(signed_in.status, "active");

            let gate = auth::require_user(&store).unwrap();
            assert_eq!(gate.id, signed_in.id);

            let project = tasks::create_project(&store, &gate, "Launch plan").unwrap();
            assert_eq!(project.name, "Launch plan");

            let task = tasks::create_task(
                &store,
                &gate,
                "Prepare first demo",
                Some(&project.id),
                Some("2026-09-16"),
            )
            .unwrap();
            let inbox_only = tasks::create_task(&store, &gate, "Undated idea", None, None).unwrap();

            // Empty titles are rejected server-side.
            let err = tasks::create_task(&store, &gate, "   ", None, None).unwrap_err();
            assert_eq!(err.code, "empty_title");

            let all = tasks::list_tasks(&store, &gate).unwrap();
            assert_eq!(all.len(), 2);
            assert!(all.iter().any(|t| t.id == task.id && t.due_date == Some("2026-09-16".into())));
            assert!(all.iter().any(|t| t.id == inbox_only.id && t.due_date.is_none()));
            signed_in
        }; // Store dropped: simulates a full application restart.

        // Pass 2: reopen — session and data must survive.
        let store = open_store(&dir);
        let restored = auth::current_user(&store).unwrap();
        assert_eq!(restored.email, user.email);
        let projects = tasks::list_projects(&store, &restored).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Launch plan");
        let all = tasks::list_tasks(&store, &restored).unwrap();
        assert_eq!(all.len(), 2);

        // User isolation: a second account must see none of Ada's rows.
        auth::register(&store, &sink, verify_base, "grace@example.com", "cobalt-lantern-7", "Grace")
            .unwrap();
        let grace_token = read_sink_token(&dir, "grace@example.com");
        auth::verify_email_token(&store, &grace_token).unwrap();
        let grace = auth::sign_in(&store, "grace@example.com", "cobalt-lantern-7").unwrap();
        let grace_projects = tasks::list_projects(&store, &grace).unwrap();
        assert!(grace_projects.is_empty(), "other users must not see Ada's projects");
        let grace_tasks = tasks::list_tasks(&store, &grace).unwrap();
        assert!(grace_tasks.is_empty(), "other users must not see Ada's tasks");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_password_is_generic_and_rate_limited() {
        let dir = temp_dir("rate-limit");
        let sink = MailSink::new(&dir);
        let store = open_store(&dir);
        auth::register(&store, &sink, "focusboard://auth", "lin@example.com", "quiet-harbor-3", "Lin")
            .unwrap();
        let token = read_sink_token(&dir, "lin@example.com");
        auth::verify_email_token(&store, &token).unwrap();

        for _ in 0..5 {
            let err = auth::sign_in(&store, "lin@example.com", "wrong-pass-1").unwrap_err();
            assert_eq!(err.code, "invalid_credentials");
            assert!(err.message.contains("email or password"));
        }
        let err = auth::sign_in(&store, "lin@example.com", "wrong-pass-1").unwrap_err();
        assert_eq!(err.code, "attempt_limit");

        // Even the correct password is locked out during the window.
        let err = auth::sign_in(&store, "lin@example.com", "quiet-harbor-3").unwrap_err();
        assert_eq!(err.code, "attempt_limit");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn private_operations_reject_without_session() {
        let dir = temp_dir("no-session");
        let store = open_store(&dir);
        let err = auth::require_user(&store).unwrap_err();
        assert_eq!(err.code, "unauthenticated");
        let err = auth::current_user(&store).unwrap_err();
        assert_eq!(err.code, "unauthenticated");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
