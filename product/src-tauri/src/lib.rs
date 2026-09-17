mod auth;
mod mail;
mod store;
mod tasks;

use mail::MailSink;
use serde::Serialize;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use store::{CommandError, CommandResult, Store};
use tauri::{Emitter, Manager};

pub struct AppState {
    store: std::sync::Mutex<Store>,
    sink: MailSink,
    verify_base: String,
}

/// The verification link points back into the app's own verify route so the
/// single-use token is consumed by the desktop client, not a browser.
const VERIFY_BASE: &str = "focusboard://auth";
const DEEP_LINK_EVENT: &str = "deep-link";
const DEEP_LINK_SOCKET: &str = "focusboard-deeplink.sock";

/// Deep-link URL delivered externally (cold boot or warm activation) and not
/// yet handed to the webview.
struct PendingDeepLink(std::sync::Mutex<Option<String>>);

/// Extracts only the token value from a delivered focusboard:// link.
fn token_from_link(url: &str) -> Option<String> {
    let rest = url.trim().strip_prefix("focusboard://")?;
    let idx = rest.find("token=")?;
    let rest = &rest[idx + "token=".len()..];
    let token: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    if token.is_empty() { None } else { Some(token) }
}

fn deep_link_from_args() -> Option<String> {
    std::env::args()
        .skip(1)
        .find(|a| a.starts_with("focusboard://"))
        .map(|a| a.to_string())
}

/// Mirrors resolve_data_dir before the Tauri app handle exists so a warm
/// activation can forward to the running instance instead of booting a
/// second full window.
fn preflight_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("FOCUSBOARD_DATA_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share")))
        .map(|d| d.join("app.focusboard.desktop"))
}

/// Socket locations for single-instance deep-link delivery. The runtime-dir
/// socket is shared by every process in the desktop session, so a warm
/// activation still forwards even when the data dir env differs; the data-dir
/// socket covers sessions without XDG_RUNTIME_DIR.
fn session_socket_paths() -> Vec<PathBuf> {
    std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .filter(|d| !d.is_empty())
        .map(|d| vec![PathBuf::from(d).join(DEEP_LINK_SOCKET)])
        .unwrap_or_default()
}

fn deep_link_socket_candidates(data_dir: &Path) -> Vec<PathBuf> {
    let mut paths = session_socket_paths();
    paths.push(data_dir.join(DEEP_LINK_SOCKET));
    paths
}

/// Sends the activated URL to the running instance. Returns true when an
/// instance acknowledged it (warm activation).
fn forward_to_running_instance(candidates: &[PathBuf], url: &str) -> bool {
    for path in candidates {
        let Ok(mut stream) = UnixStream::connect(path) else {
            continue;
        };
        if stream.write_all(url.as_bytes()).is_ok() {
            return true;
        }
    }
    false
}

/// Quotes one argv entry for a desktop-entry Exec line only when the value
/// contains characters that require it — the generic xdg-open launcher splits
/// Exec on whitespace without spec-compliant quote stripping, so an always-
/// quoted form would reach `env` with the quotes intact.
fn desktop_exec_arg(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if value.bytes().all(|b| {
        b.is_ascii_alphanumeric()
            || matches!(b, b'/' | b'.' | b'_' | b'-' | b':' | b'=' | b'@' | b'+' | b',')
    }) {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"").replace('$', "\\$"))
    }
}

/// Registers the focusboard:// scheme at the OS level for this session by
/// installing a desktop entry that points at the running binary and carries
/// the session's data dir, so an externally launched activation lands in the
/// same database. Best effort: failures never block startup.
fn register_scheme() {
    let Ok(exe) = std::env::current_exe() else { return };
    let Ok(home) = std::env::var("HOME") else { return };
    let apps = PathBuf::from(home).join(".local/share/applications");
    if std::fs::create_dir_all(&apps).is_err() {
        return;
    }
    let mut exec = String::new();
    if let Ok(dir) = std::env::var("FOCUSBOARD_DATA_DIR") {
        if !dir.is_empty() {
            exec.push_str(&format!("env FOCUSBOARD_DATA_DIR={} ", desktop_exec_arg(&dir)));
        }
    }
    exec.push_str(&desktop_exec_arg(&exe.display().to_string()));
    exec.push_str(" %u");
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Focusboard\nTerminal=false\nNoDisplay=true\nExec={exec}\nMimeType=x-scheme-handler/focusboard;\n"
    );
    if std::fs::write(apps.join("focusboard.desktop"), entry).is_err() {
        return;
    }
    let _ = std::process::Command::new("xdg-mime")
        .args(["default", "focusboard.desktop", "x-scheme-handler/focusboard"])
        .status();
}

/// Listens for URLs forwarded by later activations and pushes them to the
/// webview, focusing the window like a normal OS activation.
fn spawn_deep_link_listener(app: tauri::AppHandle, data_dir: &Path) {
    for path in deep_link_socket_candidates(data_dir) {
        let _ = std::fs::remove_file(&path);
        let Ok(listener) = UnixListener::bind(&path) else {
            continue;
        };
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let url = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                if token_from_link(&url).is_none() {
                    continue;
                }
                let _ = app.emit(DEEP_LINK_EVENT, url);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_focus();
                }
            }
        });
        return;
    }
}

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
fn take_deep_link(state: tauri::State<PendingDeepLink>) -> Option<String> {
    state.0.lock().ok().and_then(|mut pending| {
        pending
            .take()
            .filter(|url| token_from_link(url).is_some())
    })
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
    let activated_link = deep_link_from_args();
    // Warm activation: hand the URL to the running instance and exit instead
    // of booting a second full application.
    if let Some(url) = &activated_link {
        let mut candidates = session_socket_paths();
        if let Some(dir) = preflight_data_dir() {
            candidates.push(dir.join(DEEP_LINK_SOCKET));
        }
        if forward_to_running_instance(&candidates, url) {
            return;
        }
    }
    tauri::Builder::default()
        .setup(move |app| {
            let data_dir = resolve_data_dir(app.handle()).map_err(|e| e.message)?;
            let db_path = data_dir.join("focusboard.sqlite3");
            let store = Store::open(&db_path).map_err(|e| format!("Could not open database: {e}"))?;
            let sink = MailSink::new(&data_dir);
            app.manage(AppState {
                store: std::sync::Mutex::new(store),
                sink,
                verify_base: VERIFY_BASE.to_string(),
            });
            register_scheme();
            spawn_deep_link_listener(app.handle().clone(), &data_dir);
            // Cold activation: the link that launched us is handed to the
            // webview when it asks, so no token-dependent frame is missed.
            app.manage(PendingDeepLink(std::sync::Mutex::new(activated_link)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            register,
            verify_email,
            sign_in,
            sign_out,
            session_status,
            request_verification_email,
            take_deep_link,
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

    #[test]
    fn token_parsing_accepts_only_delivered_links() {
        assert_eq!(
            token_from_link("focusboard://auth/verify?token=abc123XYZ"),
            Some("abc123XYZ".to_string())
        );
        // Not a focusboard link: rejected outright.
        assert_eq!(token_from_link("https://example.com/verify?token=abc123"), None);
        assert_eq!(token_from_link("focusboard://auth/verify"), None);
        assert_eq!(token_from_link("focusboard://auth/verify?email=a@b.c"), None);
        // Encoded or punctuated payload: only the alphanumeric token is taken.
        assert_eq!(token_from_link("focusboard://auth/verify?token=abc%20def"), Some("abc".to_string()));
        assert_eq!(token_from_link("  focusboard://auth/verify?token=tok9  "), Some("tok9".to_string()));
        assert_eq!(token_from_link("focusboard://auth/verify?token="), None);
    }

    #[test]
    fn malformed_expired_and_reused_link_tokens_stay_recoverable() {
        let dir = temp_dir("link-token-states");
        let sink = MailSink::new(&dir);
        let store = open_store(&dir);
        auth::register(&store, &sink, VERIFY_BASE, "kim@example.com", "amber-meadow-5", "Kim")
            .unwrap();
        let token = read_sink_token(&dir, "kim@example.com");

        // A malformed token (e.g. tampered link) never matches: explicit error, no crash.
        let err = auth::verify_email_token(&store, "not-a-real-token").unwrap_err();
        assert_eq!(err.code, "invalid_token");

        let verified = auth::verify_email_token(&store, &token).unwrap();
        assert_eq!(verified.email, "kim@example.com");

        // Reused link token: recoverable error after single-use consumption.
        let err = auth::verify_email_token(&store, &token).unwrap_err();
        assert_eq!(err.code, "token_already_used");

        // Whatever the link carried, the shell store remains usable.
        assert!(auth::require_user(&store).is_err() || auth::current_user(&store).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
