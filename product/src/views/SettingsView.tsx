import { useCallback, useEffect, useState } from "react";
import {
  COMMON_TIMEZONES,
  Reminder,
  SessionInfo,
  Settings,
  Task,
  formatInTimezone,
  invoke,
  toCommandError,
} from "../api";
import { Icon } from "../ui";

const STATUS_COPY: Record<string, string> = {
  scheduled: "Scheduled",
  processing: "Sending",
  sent: "Sent",
  failed: "Failed",
  cancelled: "Cancelled",
  skipped: "Skipped — task already done",
};

export function SettingsView(props: {
  tasks: Task[];
  timezone: string;
  onChanged: () => void;
  onSessionEnded: (notice: string) => void;
}) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [reminders, setReminders] = useState<Reminder[] | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[] | null>(null);
  const [displayName, setDisplayName] = useState("");
  const [timezone, setTimezone] = useState(props.timezone || "UTC");
  const [notify, setNotify] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [savedNote, setSavedNote] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [passwordNote, setPasswordNote] = useState<string | null>(null);
  const [passwordBusy, setPasswordBusy] = useState(false);

  const [sessionError, setSessionError] = useState<string | null>(null);
  const [sessionNote, setSessionNote] = useState<string | null>(null);
  const [sessionBusy, setSessionBusy] = useState(false);

  const [deleteConfirm, setDeleteConfirm] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleteBusy, setDeleteBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const [s, r, sess] = await Promise.all([
        invoke<Settings>("get_settings"),
        invoke<Reminder[]>("list_reminders"),
        invoke<SessionInfo[]>("list_sessions"),
      ]);
      setSettings(s);
      setDisplayName(s.display_name);
      setTimezone(s.timezone);
      setNotify(s.notifications_enabled);
      setReminders(r);
      setSessions(sess);
      setLoadError(null);
    } catch (err) {
      setLoadError(toCommandError(err).message);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function save() {
    setBusy(true);
    setSaveError(null);
    setSavedNote(null);
    try {
      await invoke("update_settings", {
        displayName: displayName.trim(),
        timezone,
        notificationsEnabled: notify,
      });
      setSavedNote("Settings saved.");
      await refresh();
      props.onChanged();
    } catch (err) {
      setSaveError(toCommandError(err).message);
    } finally {
      setBusy(false);
    }
  }

  async function retry(reminderId: string) {
    setBusy(true);
    setSaveError(null);
    try {
      await invoke("retry_reminder", { reminderId });
      await refresh();
    } catch (err) {
      setSaveError(toCommandError(err).message);
    } finally {
      setBusy(false);
    }
  }

  async function changePassword() {
    setPasswordBusy(true);
    setPasswordError(null);
    setPasswordNote(null);
    try {
      await invoke("change_password", {
        currentPassword,
        newPassword,
      });
      setPasswordNote("Password updated. Other sessions were signed out.");
      setCurrentPassword("");
      setNewPassword("");
      await refresh();
    } catch (err) {
      setPasswordError(toCommandError(err).message);
    } finally {
      setPasswordBusy(false);
    }
  }

  async function refreshSessions() {
    try {
      setSessions(await invoke<SessionInfo[]>("list_sessions"));
      setSessionError(null);
    } catch (err) {
      const ce = toCommandError(err);
      if (ce.code === "unauthenticated" || ce.code === "session_expired") {
        props.onSessionEnded("Your session ended. Sign in to continue.");
        return;
      }
      setSessionError(ce.message);
    }
  }

  async function revokeSession(sessionId: string) {
    setSessionBusy(true);
    setSessionError(null);
    setSessionNote(null);
    try {
      const wasCurrent = await invoke<boolean>("revoke_session", { sessionId });
      if (wasCurrent) {
        props.onSessionEnded("That session was revoked. Sign in to continue.");
        return;
      }
      setSessionNote("Session revoked.");
      await refreshSessions();
    } catch (err) {
      setSessionError(toCommandError(err).message);
    } finally {
      setSessionBusy(false);
    }
  }

  async function revokeAllOtherSessions() {
    setSessionBusy(true);
    setSessionError(null);
    setSessionNote(null);
    try {
      const ended = await invoke<number>("revoke_all_other_sessions");
      setSessionNote(
        ended > 0
          ? `Signed out ${ended} other ${ended === 1 ? "session" : "sessions"}.`
          : "No other active sessions to revoke."
      );
      await refreshSessions();
    } catch (err) {
      setSessionError(toCommandError(err).message);
    } finally {
      setSessionBusy(false);
    }
  }

  async function requestDeletion() {
    setDeleteBusy(true);
    setDeleteError(null);
    try {
      await invoke("request_account_deletion", { confirmation: deleteConfirm });
      props.onSessionEnded(
        "Your account deletion request was received, and you have been signed out."
      );
    } catch (err) {
      setDeleteError(toCommandError(err).message);
      setDeleteBusy(false);
    }
  }

  if (loadError) {
    return (
      <div className="error-state" role="alert">
        <p>{loadError}</p>
        <button type="button" className="button" onClick={() => void refresh()}>
          Retry
        </button>
      </div>
    );
  }
  if (!settings || reminders === null || sessions === null) {
    return (
      <div className="loading" role="status">
        <span className="loading-dot" aria-hidden="true" /> Opening settings…
      </div>
    );
  }

  const taskTitle = (taskId: string) => props.tasks.find((t) => t.id === taskId)?.title ?? "Deleted task";

  return (
    <div className="view-body settings-view">
      <section className="settings-section" aria-label="Profile">
        <h2 className="section-title">Profile</h2>
        <div className="settings-grid">
          <label className="settings-field">
            <span>Email</span>
            <output>{settings.email}</output>
          </label>
          <label className="settings-field">
            <span>Display name</span>
            <input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              aria-label="Display name"
            />
          </label>
          <label className="settings-field">
            <span>Timezone</span>
            <select value={timezone} onChange={(e) => setTimezone(e.target.value)} aria-label="Timezone" data-testid="settings-timezone">
              {!COMMON_TIMEZONES.includes(timezone) && <option value={timezone}>{timezone}</option>}
              {COMMON_TIMEZONES.map((tz) => (
                <option key={tz} value={tz}>
                  {tz}
                </option>
              ))}
            </select>
          </label>
          <label className="settings-field check">
            <span>Reminder emails</span>
            <input
              type="checkbox"
              checked={notify}
              onChange={(e) => setNotify(e.target.checked)}
              aria-label="Enable reminder emails"
              data-testid="settings-notifications"
            />
          </label>
        </div>
        <div className="settings-actions">
          <button type="button" className="button primary" onClick={() => void save()} disabled={busy} data-testid="settings-save">
            Save changes
          </button>
          {savedNote ? <span className="saved-note" role="status">{savedNote}</span> : null}
        </div>
        {saveError ? (
          <p className="field-error" role="alert">
            {saveError}
          </p>
        ) : null}
      </section>

      <section className="settings-section" aria-label="Password">
        <h2 className="section-title">Password</h2>
        <form
          className="settings-grid"
          onSubmit={(e) => {
            e.preventDefault();
            void changePassword();
          }}
        >
          <label className="settings-field">
            <span>Current password</span>
            <input
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              autoComplete="current-password"
              aria-label="Current password"
              data-testid="settings-current-password"
            />
          </label>
          <label className="settings-field">
            <span>New password</span>
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              autoComplete="new-password"
              aria-label="New password"
              aria-describedby="password-policy-hint"
              data-testid="settings-new-password"
            />
            <small id="password-policy-hint" className="section-note">
              At least 8 characters, with at least one letter and one number.
            </small>
          </label>
          <div className="settings-actions">
            <button
              type="submit"
              className="button primary"
              disabled={passwordBusy || !currentPassword || !newPassword}
              data-testid="settings-change-password"
            >
              Change password
            </button>
            {passwordNote ? <span className="saved-note" role="status">{passwordNote}</span> : null}
          </div>
        </form>
        {passwordError ? (
          <p className="field-error" role="alert">
            {passwordError}
          </p>
        ) : null}
      </section>

      <section className="settings-section" aria-label="Sessions">
        <h2 className="section-title">Active sessions</h2>
        {sessions.length === 0 ? (
          <p className="section-note">No active sessions.</p>
        ) : (
          <ul className="delivery-list" data-testid="sessions-list">
            {sessions.map((s) => (
              <li key={s.id} className="delivery-row" data-session-id={s.id}>
                <Icon name="today" size={14} />
                <span className="delivery-task">
                  {s.device === "" ? "Unknown device" : s.device}
                  {s.current ? " — this device" : ""}
                </span>
                <span className="delivery-when">
                  signed in {formatInTimezone(s.created_at, timezone)}
                </span>
                <span className="delivery-when">
                  expires {formatInTimezone(s.expires_at, timezone)}
                </span>
                <button
                  type="button"
                  className="button small"
                  disabled={sessionBusy}
                  data-testid={`revoke-session-${s.id}`}
                  onClick={() => void revokeSession(s.id)}
                >
                  Revoke
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className="settings-actions">
          <button
            type="button"
            className="button"
            disabled={sessionBusy}
            data-testid="revoke-all-sessions"
            onClick={() => void revokeAllOtherSessions()}
          >
            Revoke all other sessions
          </button>
          {sessionNote ? <span className="saved-note" role="status">{sessionNote}</span> : null}
        </div>
        {sessionError ? (
          <p className="field-error" role="alert">
            {sessionError}
          </p>
        ) : null}
      </section>

      <section className="settings-section" aria-label="Reminder deliveries">
        <h2 className="section-title">Reminder deliveries</h2>
        {reminders.length === 0 ? (
          <p className="section-note">
            No reminders yet. Set a reminder time on any task that has a due date.
          </p>
        ) : (
          <ul className="delivery-list">
            {reminders.map((r) => (
              <li key={r.id} className="delivery-row" data-reminder-status={r.status}>
                <Icon name="mail" size={14} />
                <span className="delivery-task">{taskTitle(r.task_id)}</span>
                <span className="delivery-when">
                  {formatInTimezone(r.scheduled_at, r.timezone)} ({r.timezone})
                </span>
                <span className={`delivery-status state-${r.status}`}>{STATUS_COPY[r.status] ?? r.status}</span>
                {r.status === "failed" ? (
                  <button
                    type="button"
                    className="button small"
                    disabled={busy}
                    data-testid={`retry-reminder-${r.id}`}
                    onClick={() => void retry(r.id)}
                  >
                    Retry
                  </button>
                ) : r.sent_at ? (
                  <span className="delivery-sent">sent {formatInTimezone(r.sent_at, r.timezone)}</span>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="settings-section settings-danger" aria-label="Delete account">
        <h2 className="section-title">Delete account</h2>
        <p className="section-note">
          This requests deletion of your account and signs you out everywhere.
          This action is destructive and cannot be undone from this device.
        </p>
        <div className="settings-grid">
          <label className="settings-field">
            <span>Type DELETE to confirm</span>
            <input
              type="text"
              value={deleteConfirm}
              onChange={(e) => setDeleteConfirm(e.target.value)}
              aria-label="Type DELETE to confirm account deletion"
              data-testid="delete-confirmation"
            />
          </label>
          <div className="settings-actions">
            <button
              type="button"
              className="button"
              disabled={deleteBusy || deleteConfirm.trim() !== "DELETE"}
              data-testid="delete-account"
              onClick={() => void requestDeletion()}
            >
              Request account deletion
            </button>
          </div>
        </div>
        {deleteError ? (
          <p className="field-error" role="alert">
            {deleteError}
          </p>
        ) : null}
      </section>
    </div>
  );
}
