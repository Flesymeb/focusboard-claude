mod activity;
mod auth;
mod focus;
mod golden_path;
mod mail;
mod reminders;
mod store;
mod tasks;

use mail::MailSink;
use serde::Serialize;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use store::{CommandError, CommandResult, Store};
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

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
const SQLITE_FILE: &str = "focusboard.sqlite3";

/// Deep-link URL delivered externally (cold boot or warm activation) and not
/// yet handed to the webview.
struct PendingDeepLink(std::sync::Mutex<Option<String>>);

impl PendingDeepLink {
    fn peek(&self) -> bool {
        self.0.lock().map(|guard| guard.is_some()).unwrap_or(false)
    }
}

/// Out-of-process observable record of focusboard:// activation: how many
/// cold deliveries were handed to the webview, how many warm activations the
/// listener forwarded, and the outcome of the most recent verification. The
/// link URLs and token values themselves are never recorded here.
///
/// Counter persistence is strictly opt-in: only when the host evidence lane
/// sets FOCUSBOARD_EVIDENCE_COUNTERS=1 does every update mirror to a
/// machine-readable JSON file next to the store so an external collector can
/// read activations off zero. Normal startup never writes this file.
struct DeepLinkLedger {
    state: std::sync::Mutex<DeepLinkLedgerState>,
    mirror_path: Option<PathBuf>,
}

#[derive(Default)]
struct DeepLinkLedgerState {
    cold_delivered: u32,
    warm_forwarded: u32,
    last_verify_outcome: Option<String>,
    last_reset_outcome: Option<String>,
}

#[derive(Serialize)]
struct DeepLinkStatus {
    pending_now: bool,
    cold_delivered: u32,
    warm_forwarded: u32,
    last_verify_outcome: Option<String>,
    last_reset_outcome: Option<String>,
}

impl DeepLinkLedger {
    fn new(mirror_path: Option<PathBuf>) -> Self {
        DeepLinkLedger {
            state: std::sync::Mutex::new(DeepLinkLedgerState::default()),
            mirror_path,
        }
    }

    /// Mirrors the current counters to the opt-in evidence file. Counters and
    /// outcome codes only — never link URLs or token material.
    fn persist(&self, state: &DeepLinkLedgerState) {
        let Some(path) = &self.mirror_path else {
            return;
        };
        let json = serde_json::json!({
            "cold_delivered": state.cold_delivered,
            "warm_forwarded": state.warm_forwarded,
            "last_verify_outcome": state.last_verify_outcome,
            "last_reset_outcome": state.last_reset_outcome,
            "updated_at": store::now_rfc3339(),
        });
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    fn record_cold_delivery(&self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.cold_delivered += 1;
            self.persist(&guard);
        }
    }

    fn record_warm_forward(&self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.warm_forwarded += 1;
            self.persist(&guard);
        }
    }

    fn record_verify_outcome(&self, outcome: &str) {
        if let Ok(mut guard) = self.state.lock() {
            guard.last_verify_outcome = Some(outcome.to_string());
            self.persist(&guard);
        }
    }

    fn record_reset_outcome(&self, outcome: &str) {
        if let Ok(mut guard) = self.state.lock() {
            guard.last_reset_outcome = Some(outcome.to_string());
            self.persist(&guard);
        }
    }

    fn status(&self, pending_now: bool) -> DeepLinkStatus {
        let guard = self.state.lock().ok();
        let state = guard.as_deref();
        DeepLinkStatus {
            pending_now,
            cold_delivered: state.map(|s| s.cold_delivered).unwrap_or(0),
            warm_forwarded: state.map(|s| s.warm_forwarded).unwrap_or(0),
            last_verify_outcome: state.and_then(|s| s.last_verify_outcome.clone()),
            last_reset_outcome: state.and_then(|s| s.last_reset_outcome.clone()),
        }
    }
}

/// Extracts only the token value from a delivered focusboard:// link.
pub(crate) fn token_from_link(url: &str) -> Option<String> {
    let rest = url.trim().strip_prefix("focusboard://")?;
    let idx = rest.find("token=")?;
    let rest = &rest[idx + "token=".len()..];
    let token: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
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
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local/share"))
        })
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
            || matches!(
                b,
                b'/' | b'.' | b'_' | b'-' | b':' | b'=' | b'@' | b'+' | b','
            )
    }) {
        value.to_string()
    } else {
        format!(
            "\"{}\"",
            value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('$', "\\$")
        )
    }
}

/// Registers the focusboard:// scheme at the OS level for this session by
/// installing a desktop entry that points at the running binary and carries
/// the session's data dir, so an externally launched activation lands in the
/// same database. Best effort: failures never block startup.
fn register_scheme() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Ok(home) = std::env::var("HOME") else {
        return;
    };
    let apps = PathBuf::from(home).join(".local/share/applications");
    if std::fs::create_dir_all(&apps).is_err() {
        return;
    }
    let mut exec = String::new();
    if let Ok(dir) = std::env::var("FOCUSBOARD_DATA_DIR") {
        if !dir.is_empty() {
            exec.push_str(&format!(
                "env FOCUSBOARD_DATA_DIR={} ",
                desktop_exec_arg(&dir)
            ));
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
        .args([
            "default",
            "focusboard.desktop",
            "x-scheme-handler/focusboard",
        ])
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
                app.state::<DeepLinkLedger>().record_warm_forward();
                let _ = app.emit(DEEP_LINK_EVENT, url);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_focus();
                }
            }
        });
        return;
    }
}

pub(crate) fn map_result<T, F>(state: &AppState, f: F) -> Result<T, CommandError>
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
        auth::register(
            s,
            &state.sink,
            &state.verify_base,
            &email,
            &password,
            &display_name,
        )
    })?;
    Ok(RegisterOutcome {
        status,
        email: email.trim().to_string(),
    })
}

#[tauri::command]
fn verify_email(
    state: tauri::State<AppState>,
    ledger: tauri::State<DeepLinkLedger>,
    token: String,
) -> Result<auth::PublicUser, CommandError> {
    match map_result(&state, |s| auth::verify_email_token(s, &token)) {
        Ok(user) => {
            ledger.record_verify_outcome("verified");
            Ok(user)
        }
        Err(err) => {
            if matches!(
                err.code.as_str(),
                "token_already_used" | "token_expired" | "invalid_token"
            ) {
                ledger.record_verify_outcome(&err.code);
            }
            Err(err)
        }
    }
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
fn take_deep_link(
    pending: tauri::State<PendingDeepLink>,
    ledger: tauri::State<DeepLinkLedger>,
) -> Option<String> {
    let url = pending
        .0
        .lock()
        .ok()
        .and_then(|mut guard| guard.take().filter(|url| token_from_link(url).is_some()));
    if url.is_some() {
        ledger.record_cold_delivery();
    }
    url
}

#[tauri::command]
fn deep_link_status(
    pending: tauri::State<PendingDeepLink>,
    ledger: tauri::State<DeepLinkLedger>,
) -> DeepLinkStatus {
    ledger.status(pending.peek())
}

/// Runtime-state receipt field naming the identifier-derived durable store so
/// the isolated-store advisory can measure the real app.focusboard.desktop
/// data directory instead of a foreign path. Directory and file name only —
/// never token or session material.
#[derive(Serialize, Clone)]
struct StoreLocation {
    identifier: String,
    data_dir: String,
    sqlite_file: String,
}

#[tauri::command]
fn store_location(state: tauri::State<StoreLocation>) -> StoreLocation {
    state.inner().clone()
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

/// The reset email's action link targets the auth/reset route of the same
/// focusboard:// scheme the verification flow uses.
fn reset_base(verify_base: &str) -> String {
    format!("{verify_base}/reset")
}

#[tauri::command]
fn request_password_reset(
    state: tauri::State<AppState>,
    email: String,
) -> Result<(), CommandError> {
    map_result(&state, |s| {
        auth::request_password_reset(s, &state.sink, &reset_base(&state.verify_base), &email)
    })
}

#[tauri::command]
fn reset_password(
    state: tauri::State<AppState>,
    ledger: tauri::State<DeepLinkLedger>,
    token: String,
    password: String,
) -> Result<auth::PublicUser, CommandError> {
    match map_result(&state, |s| auth::reset_password(s, &token, &password)) {
        Ok(user) => {
            ledger.record_reset_outcome("reset");
            Ok(user)
        }
        Err(err) => {
            if matches!(
                err.code.as_str(),
                "token_already_used" | "token_expired" | "invalid_token"
            ) {
                ledger.record_reset_outcome(&err.code);
            }
            Err(err)
        }
    }
}

#[tauri::command]
fn change_password(
    state: tauri::State<AppState>,
    current_password: String,
    new_password: String,
) -> Result<auth::PublicUser, CommandError> {
    map_result(&state, |s| {
        auth::change_password(s, &current_password, &new_password)
    })
}

#[tauri::command]
fn list_sessions(state: tauri::State<AppState>) -> Result<Vec<auth::SessionInfo>, CommandError> {
    map_result(&state, auth::list_sessions)
}

#[tauri::command]
fn revoke_session(state: tauri::State<AppState>, session_id: String) -> Result<bool, CommandError> {
    map_result(&state, |s| auth::revoke_session(s, &session_id))
}

#[tauri::command]
fn revoke_all_other_sessions(state: tauri::State<AppState>) -> Result<u64, CommandError> {
    map_result(&state, auth::revoke_all_other_sessions)
}

#[tauri::command]
fn request_account_deletion(
    state: tauri::State<AppState>,
    confirmation: String,
) -> Result<(), CommandError> {
    map_result(&state, |s| auth::request_account_deletion(s, &confirmation))
}

#[tauri::command]
fn create_project(
    state: tauri::State<AppState>,
    name: String,
) -> Result<tasks::Project, CommandError> {
    require_then(&state, |s, u| tasks::create_project(s, u, &name))
}

#[tauri::command]
fn rename_project(
    state: tauri::State<AppState>,
    project_id: String,
    name: String,
) -> Result<tasks::Project, CommandError> {
    require_then(&state, |s, u| {
        tasks::rename_project(s, u, &project_id, &name)
    })
}

#[tauri::command]
fn list_projects(state: tauri::State<AppState>) -> Result<Vec<tasks::Project>, CommandError> {
    require_then(&state, |s, u| tasks::list_projects(s, u))
}

#[tauri::command]
fn list_archived_projects(
    state: tauri::State<AppState>,
) -> Result<Vec<tasks::Project>, CommandError> {
    require_then(&state, |s, u| tasks::list_archived_projects(s, u))
}

#[tauri::command]
fn set_project_archived(
    state: tauri::State<AppState>,
    project_id: String,
    archived: bool,
) -> Result<tasks::Project, CommandError> {
    require_then(&state, |s, u| {
        tasks::set_project_archived(s, u, &project_id, archived)
    })
}

#[tauri::command]
fn create_task(
    state: tauri::State<AppState>,
    title: String,
    project_id: Option<String>,
    due_date: Option<String>,
) -> Result<tasks::Task, CommandError> {
    // Opt-in golden-path fault: exactly one armed mutation request fails at
    // this boundary so the composer renders its PRD 9.2 network-error state;
    // unarmed, this is a no-op for every normal session.
    if golden_path::consume_composer_fault() {
        return Err(CommandError::new(
            "connection_error",
            "Focusboard could not reach its local service. The task was not added.",
        ));
    }
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

#[tauri::command]
fn set_task_priority(
    state: tauri::State<AppState>,
    task_id: String,
    priority: String,
) -> Result<tasks::Task, CommandError> {
    require_then(&state, |s, u| {
        tasks::set_task_priority(s, u, &task_id, &priority)
    })
}

#[tauri::command]
fn query_tasks(
    state: tauri::State<AppState>,
    search: Option<String>,
    priority: Option<String>,
    due: Option<String>,
    status: Option<String>,
    today: Option<String>,
) -> Result<Vec<tasks::Task>, CommandError> {
    require_then(&state, |s, u| {
        tasks::query_tasks(
            s,
            u,
            search.as_deref(),
            priority.as_deref(),
            due.as_deref(),
            status.as_deref(),
            today.as_deref(),
        )
    })
}

#[tauri::command]
fn list_subtasks(
    state: tauri::State<AppState>,
    task_id: String,
) -> Result<Vec<tasks::Subtask>, CommandError> {
    require_then(&state, |s, u| tasks::list_subtasks(s, u, &task_id))
}

#[tauri::command]
fn add_subtask(
    state: tauri::State<AppState>,
    task_id: String,
    title: String,
) -> Result<tasks::Subtask, CommandError> {
    require_then(&state, |s, u| tasks::add_subtask(s, u, &task_id, &title))
}

#[tauri::command]
fn set_subtask_done(
    state: tauri::State<AppState>,
    task_id: String,
    subtask_id: String,
    done: bool,
) -> Result<tasks::Subtask, CommandError> {
    require_then(&state, |s, u| {
        tasks::set_subtask_done(s, u, &task_id, &subtask_id, done)
    })
}

#[tauri::command]
fn remove_subtask(
    state: tauri::State<AppState>,
    task_id: String,
    subtask_id: String,
) -> Result<(), CommandError> {
    require_then(&state, |s, u| {
        tasks::remove_subtask(s, u, &task_id, &subtask_id)
    })
}

#[tauri::command]
fn list_task_activity(
    state: tauri::State<AppState>,
    task_id: String,
) -> Result<Vec<activity::ActivityEvent>, CommandError> {
    require_then(&state, |s, u| activity::list_for_task(s, u, &task_id))
}

#[tauri::command]
fn set_task_reminder(
    state: tauri::State<AppState>,
    task_id: String,
    reminder_time: String,
    timezone: String,
) -> Result<reminders::Reminder, CommandError> {
    require_then(&state, |s, u| {
        reminders::set_task_reminder(s, u, &task_id, &reminder_time, &timezone)
    })
}

#[tauri::command]
fn list_reminders(state: tauri::State<AppState>) -> Result<Vec<reminders::Reminder>, CommandError> {
    require_then(&state, |s, u| reminders::list_reminders(s, u))
}

#[tauri::command]
fn retry_reminder(
    state: tauri::State<AppState>,
    reminder_id: String,
) -> Result<reminders::Reminder, CommandError> {
    require_then(&state, |s, u| reminders::retry_reminder(s, u, &reminder_id))
}

#[tauri::command]
fn send_test_email(
    state: tauri::State<AppState>,
) -> Result<reminders::TestEmailOutcome, CommandError> {
    require_then(&state, |_s, u| reminders::send_test_email(u, &state.sink))
}

#[derive(Serialize)]
struct Settings {
    email: String,
    display_name: String,
    timezone: String,
    notifications_enabled: bool,
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<Settings, CommandError> {
    require_then(&state, |s, u| {
        let row: (String, String, i64) = s
            .conn
            .query_row(
                "SELECT email, display_name, notifications_enabled FROM users WHERE id = ?1",
                [&u.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not load settings: {e}"))
            })?;
        Ok(Settings {
            email: row.0,
            display_name: row.1,
            timezone: u.timezone.clone(),
            notifications_enabled: row.2 != 0,
        })
    })
}

#[tauri::command]
fn update_settings(
    state: tauri::State<AppState>,
    display_name: Option<String>,
    timezone: Option<String>,
    notifications_enabled: Option<bool>,
) -> Result<Settings, CommandError> {
    require_then(&state, |s, u| {
        if let Some(tz) = timezone.as_deref() {
            if tz.parse::<chrono_tz::Tz>().is_err() {
                return Err(CommandError::new(
                    "invalid_timezone",
                    "Choose a valid IANA timezone, like Asia/Shanghai.",
                ));
            }
        }
        let name = display_name
            .as_ref()
            .map(|n| n.trim().to_string())
            .unwrap_or_else(|| String::new());
        if name.chars().count() > 80 {
            return Err(CommandError::new(
                "name_too_long",
                "Keep display names under 80 characters.",
            ));
        }
        let n = s
            .conn
            .execute(
                "UPDATE users SET
                    display_name = COALESCE(?1, display_name),
                    timezone = COALESCE(?2, timezone),
                    notifications_enabled = COALESCE(?3, notifications_enabled)
                 WHERE id = ?4",
                rusqlite::params![
                    display_name.as_deref().map(|_| name.clone()),
                    timezone.as_deref(),
                    notifications_enabled.map(|v| v as i64),
                    u.id
                ],
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not save settings: {e}"))
            })?;
        if n == 0 {
            return Err(CommandError::new("not_found", "Account not found."));
        }
        let row: (String, String, String, i64) = s
            .conn
            .query_row(
                "SELECT email, display_name, timezone, notifications_enabled FROM users WHERE id = ?1",
                [&u.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|e| CommandError::new("storage_error", format!("Could not load settings: {e}")))?;
        Ok(Settings {
            email: row.0,
            display_name: row.1,
            timezone: row.2,
            notifications_enabled: row.3 != 0,
        })
    })
}

#[tauri::command]
fn start_focus_session(
    state: tauri::State<AppState>,
    task_id: String,
) -> Result<focus::FocusSession, CommandError> {
    require_then(&state, |s, u| focus::start_session(s, u, &task_id))
}

#[tauri::command]
fn pause_focus_session(
    state: tauri::State<AppState>,
    session_id: String,
) -> Result<focus::FocusSession, CommandError> {
    require_then(&state, |s, u| focus::pause_session(s, u, &session_id))
}

#[tauri::command]
fn resume_focus_session(
    state: tauri::State<AppState>,
    session_id: String,
) -> Result<focus::FocusSession, CommandError> {
    require_then(&state, |s, u| focus::resume_session(s, u, &session_id))
}

#[tauri::command]
fn finish_focus_session(
    state: tauri::State<AppState>,
    session_id: String,
) -> Result<focus::FocusSession, CommandError> {
    require_then(&state, |s, u| focus::finish_session(s, u, &session_id))
}

#[tauri::command]
fn cancel_focus_session(
    state: tauri::State<AppState>,
    session_id: String,
) -> Result<focus::FocusSession, CommandError> {
    require_then(&state, |s, u| focus::cancel_session(s, u, &session_id))
}

#[tauri::command]
fn list_focus_sessions(
    state: tauri::State<AppState>,
) -> Result<Vec<focus::FocusSession>, CommandError> {
    require_then(&state, |s, u| focus::list_sessions(s, u))
}

#[tauri::command]
fn active_focus_session(
    state: tauri::State<AppState>,
) -> Result<Option<focus::FocusSession>, CommandError> {
    require_then(&state, |s, u| focus::active_session(s, u))
}

/// Idempotent, restart-safe reminder scheduler. Reminders live in SQLite, so
/// a pass on every tick covers anything that came due while the app was
/// closed; the status-guarded claim inside tick() makes repeated passes and
/// restarts duplicate-free.
fn spawn_reminder_scheduler(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(3));
        if let Some(state) = app.try_state::<AppState>() {
            let _ = map_result(&state, |s| reminders::tick(s, &state.sink));
        }
    });
}

fn resolve_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, CommandError> {
    if let Ok(dir) = std::env::var("FOCUSBOARD_DATA_DIR") {
        if !dir.is_empty() {
            let path = PathBuf::from(dir);
            std::fs::create_dir_all(&path).map_err(|e| {
                CommandError::new("storage_error", format!("Data directory unavailable: {e}"))
            })?;
            return Ok(path);
        }
    }
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| CommandError::new("storage_error", "Data directory could not be resolved."))?;
    std::fs::create_dir_all(&dir).map_err(|e| {
        CommandError::new("storage_error", format!("Data directory unavailable: {e}"))
    })?;
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
        .plugin(tauri_plugin_deep_link::init())
        .setup(move |app| {
            let data_dir = resolve_data_dir(app.handle()).map_err(|e| e.message)?;
            let db_path = data_dir.join(SQLITE_FILE);
            let store =
                Store::open(&db_path).map_err(|e| format!("Could not open database: {e}"))?;
            let sink = MailSink::new(&data_dir);
            app.manage(AppState {
                store: std::sync::Mutex::new(store),
                sink,
                verify_base: VERIFY_BASE.to_string(),
            });
            // The built configuration declares the focusboard:// scheme; this
            // registers it with the OS so an external activation routes
            // through the handler instead of a direct binary launch. The
            // session-level desktop entry stays as a fallback for systems
            // where the plugin's registration cannot run.
            if app.deep_link().register_all().is_err() {
                register_scheme();
            }
            // Delivery counters mirror to disk only behind the explicit
            // evidence opt-in; a normal start writes nothing.
            let evidence_counters =
                std::env::var("FOCUSBOARD_EVIDENCE_COUNTERS").ok() == Some("1".to_string());
            app.manage(DeepLinkLedger::new(if evidence_counters {
                Some(data_dir.join("deeplink-counters.json"))
            } else {
                None
            }));
            app.manage(StoreLocation {
                identifier: app.config().identifier.clone(),
                data_dir: data_dir.display().to_string(),
                sqlite_file: SQLITE_FILE.to_string(),
            });
            app.manage(golden_path::GoldenPathDrive::new());
            spawn_deep_link_listener(app.handle().clone(), &data_dir);
            // Cold activation: the link that launched us is handed to the
            // webview when it asks, so no token-dependent frame is missed.
            app.manage(PendingDeepLink(std::sync::Mutex::new(activated_link)));
            spawn_reminder_scheduler(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            register,
            verify_email,
            sign_in,
            sign_out,
            session_status,
            request_verification_email,
            request_password_reset,
            reset_password,
            change_password,
            list_sessions,
            revoke_session,
            revoke_all_other_sessions,
            request_account_deletion,
            take_deep_link,
            create_project,
            rename_project,
            list_projects,
            list_archived_projects,
            set_project_archived,
            create_task,
            list_tasks,
            query_tasks,
            assign_task,
            set_task_due,
            set_task_status,
            set_task_priority,
            list_subtasks,
            add_subtask,
            set_subtask_done,
            remove_subtask,
            list_task_activity,
            set_task_reminder,
            list_reminders,
            retry_reminder,
            send_test_email,
            get_settings,
            update_settings,
            start_focus_session,
            pause_focus_session,
            resume_focus_session,
            finish_focus_session,
            cancel_focus_session,
            list_focus_sessions,
            active_focus_session,
            deep_link_status,
            store_location,
            golden_path::golden_path_drive_start,
            golden_path::golden_path_drive_register,
            golden_path::golden_path_drive_consume_verification,
            golden_path::golden_path_drive_sign_in,
            golden_path::golden_path_check_reminder_email,
            golden_path::golden_path_fault_arm
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focusboard");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("focusboard-test-{name}-{}", store::new_id("t")));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn open_store(dir: &PathBuf) -> Store {
        Store::open(&dir.join("focusboard.sqlite3")).unwrap()
    }

    fn read_sink_url(dir: &PathBuf, to: &str) -> String {
        let sink_dir = dir.join("mail-sink");
        let mut last: Option<String> = None;
        for entry in std::fs::read_dir(&sink_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = std::fs::read_to_string(&path).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
                if parsed["to"].as_str() == Some(to) {
                    assert!(!raw.contains("password"), "mail must not contain passwords");
                    last = parsed["action_url"].as_str().map(|s| s.to_string());
                }
            }
        }
        last.expect("mail sink must contain the verification message")
    }

    fn read_sink_token(dir: &PathBuf, to: &str) -> String {
        let url = read_sink_url(dir, to);
        let token = url.rsplit('=').next().unwrap().to_string();
        assert!(!token.is_empty());
        token
    }

    /// White-box self-drive of the release-critical PRD section 3 path at the
    /// domain/IPC layer: register, mail-sink verify, sign in, project, task
    /// with reminder and timezone, focus session, reminder delivery, task
    /// completion, then sign-out/sign-in with durable state. The host replay
    /// drives the same steps through the live UI.
    #[test]
    fn golden_path_section3_self_drive() {
        let dir = temp_dir("golden-path");
        let sink = MailSink::new(&dir);

        // Steps 1-3: register and consume the mail-sink verification link.
        let store = open_store(&dir);
        auth::register(
            &store,
            &sink,
            VERIFY_BASE,
            "path@example.com",
            "tulip-garnet-9",
            "Path",
        )
        .unwrap();
        let token = read_sink_token(&dir, "path@example.com");
        auth::verify_email_token(&store, &token).unwrap();

        // Step 4: sign in.
        let user = auth::sign_in(&store, "path@example.com", "tulip-garnet-9").unwrap();

        // Step 5: project.
        let project = tasks::create_project(&store, &user, "Launch plan").unwrap();

        // Step 6: task with due date and reminder time + timezone.
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let task = tasks::create_task(
            &store,
            &user,
            "Prepare first demo",
            Some(&project.id),
            Some(&today),
        )
        .unwrap();
        let due_local = (chrono::Utc::now() - chrono::Duration::seconds(2))
            .with_timezone(&chrono_tz::UTC)
            .format("%Y-%m-%dT%H:%M")
            .to_string();
        let reminder =
            reminders::set_task_reminder(&store, &user, &task.id, &due_local, "Asia/Shanghai")
                .unwrap();

        // Step 7: the task is visible in the user's lists with its date.
        let listed = tasks::list_tasks(&store, &user).unwrap();
        assert!(listed
            .iter()
            .any(|t| t.id == task.id && t.due_date == Some(today.clone())));

        // Step 8: focus session start -> pause -> resume -> finish.
        let session = focus::start_session(&store, &user, &task.id).unwrap();
        assert_eq!(session.task_title.as_deref(), Some("Prepare first demo"));
        let paused = focus::pause_session(&store, &user, &session.id).unwrap();
        assert_eq!(paused.state, focus::STATE_PAUSED);
        let resumed = focus::resume_session(&store, &user, &session.id).unwrap();
        assert_eq!(resumed.state, focus::STATE_RUNNING);
        let finished = focus::finish_session(&store, &user, &session.id).unwrap();
        assert_eq!(finished.state, focus::STATE_COMPLETED);

        // Step 10: the reminder fires at its instant, exactly one email.
        assert_eq!(reminders::tick(&store, &sink).unwrap(), 1);
        assert_eq!(reminders::tick(&store, &sink).unwrap(), 0);
        let delivered = reminders::get_reminder(&store, &user, &reminder.id).unwrap();
        assert_eq!(delivered.status, "sent");

        // Steps 8-9: complete the task exactly once; the summary state updates.
        let done =
            tasks::update_task(&store, &user, &task.id, None, None, Some("completed")).unwrap();
        assert_eq!(done.status, "completed");
        assert!(done.completed_at.is_some());
        let completed_again =
            tasks::update_task(&store, &user, &task.id, None, None, Some("completed")).unwrap();
        assert_eq!(completed_again.completed_at, done.completed_at);

        // Step 11: sign out, sign back in, state persists.
        auth::sign_out(&store).unwrap();
        let err = auth::current_user(&store).unwrap_err();
        assert_eq!(err.code, "unauthenticated");
        let again = auth::sign_in(&store, "path@example.com", "tulip-garnet-9").unwrap();
        assert_eq!(again.id, user.id);
        let persisted = tasks::list_tasks(&store, &again).unwrap();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].status, "completed");
        let projects = tasks::list_projects(&store, &again).unwrap();
        assert_eq!(projects.len(), 1);
        let sessions = focus::list_sessions(&store, &again).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].state, focus::STATE_COMPLETED);
        let reminders_after = reminders::list_reminders(&store, &again).unwrap();
        assert_eq!(reminders_after.len(), 1);
        assert_eq!(reminders_after[0].status, "sent");

        // Terminal contract: the golden path ends with the account still
        // signed in — the shell that remains is the signed-in Today surface,
        // not an anonymous restore-shell. No terminal sign-out runs.
        let terminal = auth::current_user(&store).unwrap();
        assert_eq!(terminal.id, user.id);
        assert_eq!(terminal.email, "path@example.com");

        // Reminder mail content: identity, title, due context, safe link.
        let raw = std::fs::read_to_string(
            dir.join("mail-sink")
                .join(format!("{}.json", delivered.provider_message_id.unwrap())),
        )
        .unwrap();
        assert!(raw.contains("Prepare first demo"));
        assert!(raw.contains("Focusboard"));
        assert!(raw.contains("focusboard://task?id="));
        assert!(!raw.contains("token="));
        assert!(!raw.contains("password"));

        // Step 12: warm second-instance focusboard:// activation. A later
        // activation forwards the link over the running instance's socket and
        // exits instead of booting a second full application.
        auth::register(
            &store,
            &sink,
            VERIFY_BASE,
            "warm@example.com",
            "gentle-river-8",
            "Warm",
        )
        .unwrap();
        let warm_url = read_sink_url(&dir, "warm@example.com");
        let warm_socket = dir.join("warm-deeplink.sock");
        let listener = UnixListener::bind(&warm_socket).unwrap();
        let received = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            String::from_utf8_lossy(&buf[..n]).trim().to_string()
        });
        assert!(
            forward_to_running_instance(&[warm_socket.clone()], &warm_url),
            "warm activation must be acknowledged by the running instance"
        );
        let forwarded = received.join().unwrap();
        assert_eq!(forwarded, warm_url);
        let warm_token = token_from_link(&forwarded).expect("forwarded link carries a token");
        assert_eq!(
            token_from_link(&warm_url).as_deref(),
            Some(warm_token.as_str())
        );

        // Step 13: cold focusboard:// boot. The launching link waits in
        // PendingDeepLink and is handed to the webview exactly once, filtered
        // to URLs that actually carry a token.
        auth::register(
            &store,
            &sink,
            VERIFY_BASE,
            "cold@example.com",
            "amber-harbor-6",
            "Cold",
        )
        .unwrap();
        let cold_url = read_sink_url(&dir, "cold@example.com");
        let pending = PendingDeepLink(std::sync::Mutex::new(Some(cold_url.clone())));
        let taken = {
            let mut guard = pending.0.lock().unwrap();
            guard.take().filter(|url| token_from_link(url).is_some())
        };
        assert_eq!(taken.as_deref(), Some(cold_url.as_str()));
        assert!(
            pending.0.lock().unwrap().take().is_none(),
            "the link is delivered exactly once"
        );
        let cold_token = token_from_link(taken.as_deref().unwrap()).unwrap();
        let verified = auth::verify_email_token(&store, &cold_token).unwrap();
        assert_eq!(verified.email, "cold@example.com");

        // Step 14: a reused link lands in the recoverable expired-link state
        // without breaking the session or the store.
        let err = auth::verify_email_token(&store, &cold_token).unwrap_err();
        assert_eq!(err.code, "token_already_used");
        let signed_in = auth::sign_in(&store, "cold@example.com", "amber-harbor-6").unwrap();
        assert_eq!(signed_in.email, "cold@example.com");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn deeplink_ledger_tracks_delivery_and_outcomes_without_links() {
        let ledger = DeepLinkLedger::new(None);
        let status = ledger.status(true);
        assert!(status.pending_now);
        assert_eq!(status.cold_delivered, 0);
        assert_eq!(status.warm_forwarded, 0);
        assert_eq!(status.last_verify_outcome, None);
        assert_eq!(status.last_reset_outcome, None);

        ledger.record_cold_delivery();
        ledger.record_cold_delivery();
        ledger.record_warm_forward();
        ledger.record_verify_outcome("verified");
        ledger.record_verify_outcome("token_already_used");
        ledger.record_reset_outcome("reset");
        let status = ledger.status(false);
        assert!(!status.pending_now);
        assert_eq!(status.cold_delivered, 2);
        assert_eq!(status.warm_forwarded, 1);
        assert_eq!(
            status.last_verify_outcome.as_deref(),
            Some("token_already_used")
        );
        assert_eq!(status.last_reset_outcome.as_deref(), Some("reset"));
    }

    #[test]
    fn counter_mirror_writes_only_behind_explicit_opt_in_and_never_carries_tokens() {
        let dir = temp_dir("counter-mirror");
        let path = dir.join("deeplink-counters.json");

        // Without the opt-in path, nothing is ever written to disk.
        let silent = DeepLinkLedger::new(None);
        silent.record_cold_delivery();
        silent.record_warm_forward();
        assert!(!path.exists());

        // With the opt-in path, real activations are countable off zero by an
        // external reader, and the file carries no link or token material.
        let mirrored = DeepLinkLedger::new(Some(path.clone()));
        mirrored.record_cold_delivery();
        mirrored.record_cold_delivery();
        mirrored.record_warm_forward();
        mirrored.record_verify_outcome("verified");
        let raw = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["cold_delivered"], 2);
        assert_eq!(parsed["warm_forwarded"], 1);
        assert_eq!(parsed["last_verify_outcome"], "verified");
        assert!(parsed["updated_at"].as_str().is_some());
        assert!(!raw.contains("focusboard://"));
        assert!(!raw.contains("token"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn register_verify_signin_quickadd_restart_persists() {
        let dir = temp_dir("full-path");
        let sink = MailSink::new(&dir);
        let verify_base = "focusboard://auth";

        // Pass 1: register -> verify -> sign in -> capture data.
        let user = {
            let store = open_store(&dir);
            let outcome = auth::register(
                &store,
                &sink,
                verify_base,
                "ada@example.com",
                "tulip-garnet-9",
                "Ada",
            )
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
            assert!(all
                .iter()
                .any(|t| t.id == task.id && t.due_date == Some("2026-09-16".into())));
            assert!(all
                .iter()
                .any(|t| t.id == inbox_only.id && t.due_date.is_none()));
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
        auth::register(
            &store,
            &sink,
            verify_base,
            "grace@example.com",
            "cobalt-lantern-7",
            "Grace",
        )
        .unwrap();
        let grace_token = read_sink_token(&dir, "grace@example.com");
        auth::verify_email_token(&store, &grace_token).unwrap();
        let grace = auth::sign_in(&store, "grace@example.com", "cobalt-lantern-7").unwrap();
        let grace_projects = tasks::list_projects(&store, &grace).unwrap();
        assert!(
            grace_projects.is_empty(),
            "other users must not see Ada's projects"
        );
        let grace_tasks = tasks::list_tasks(&store, &grace).unwrap();
        assert!(
            grace_tasks.is_empty(),
            "other users must not see Ada's tasks"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_password_is_generic_and_rate_limited() {
        let dir = temp_dir("rate-limit");
        let sink = MailSink::new(&dir);
        let store = open_store(&dir);
        auth::register(
            &store,
            &sink,
            "focusboard://auth",
            "lin@example.com",
            "quiet-harbor-3",
            "Lin",
        )
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
        assert_eq!(
            token_from_link("https://example.com/verify?token=abc123"),
            None
        );
        assert_eq!(token_from_link("focusboard://auth/verify"), None);
        assert_eq!(
            token_from_link("focusboard://auth/verify?email=a@b.c"),
            None
        );
        // Encoded or punctuated payload: only the alphanumeric token is taken.
        assert_eq!(
            token_from_link("focusboard://auth/verify?token=abc%20def"),
            Some("abc".to_string())
        );
        assert_eq!(
            token_from_link("  focusboard://auth/verify?token=tok9  "),
            Some("tok9".to_string())
        );
        assert_eq!(token_from_link("focusboard://auth/verify?token="), None);
    }

    #[test]
    fn malformed_expired_and_reused_link_tokens_stay_recoverable() {
        let dir = temp_dir("link-token-states");
        let sink = MailSink::new(&dir);
        let store = open_store(&dir);
        auth::register(
            &store,
            &sink,
            VERIFY_BASE,
            "kim@example.com",
            "amber-meadow-5",
            "Kim",
        )
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

    #[test]
    fn built_config_registers_focusboard_scheme() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let raw = std::fs::read_to_string(manifest.join("tauri.conf.json")).unwrap();
        let config: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(config["identifier"], "app.focusboard.desktop");
        let schemes = config["plugins"]["deep-link"]["desktop"]["schemes"]
            .as_array()
            .unwrap_or_else(|| panic!("deep-link desktop schemes missing from built config"));
        assert!(
            schemes.iter().any(|s| s.as_str() == Some("focusboard")),
            "focusboard scheme not registered in built config: {schemes:?}"
        );
    }

    #[test]
    fn preflight_data_dir_is_identifier_derived() {
        let dir = temp_dir("store-receipt-xdg");
        let prev_xdg = std::env::var("XDG_DATA_HOME").ok();
        let prev_focus = std::env::var("FOCUSBOARD_DATA_DIR").ok();
        std::env::remove_var("FOCUSBOARD_DATA_DIR");
        std::env::set_var("XDG_DATA_HOME", &dir);
        let resolved = preflight_data_dir();
        match prev_xdg {
            Some(value) => std::env::set_var("XDG_DATA_HOME", value),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        if let Some(value) = prev_focus {
            std::env::set_var("FOCUSBOARD_DATA_DIR", value);
        }
        assert_eq!(resolved.unwrap(), dir.join("app.focusboard.desktop"));
        assert_eq!(SQLITE_FILE, "focusboard.sqlite3");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
