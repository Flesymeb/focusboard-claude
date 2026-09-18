import { useCallback, useEffect, useState } from "react";
import {
  COMMON_TIMEZONES,
  Reminder,
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
}) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [reminders, setReminders] = useState<Reminder[] | null>(null);
  const [displayName, setDisplayName] = useState("");
  const [timezone, setTimezone] = useState(props.timezone || "UTC");
  const [notify, setNotify] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [savedNote, setSavedNote] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const [s, r] = await Promise.all([
        invoke<Settings>("get_settings"),
        invoke<Reminder[]>("list_reminders"),
      ]);
      setSettings(s);
      setDisplayName(s.display_name);
      setTimezone(s.timezone);
      setNotify(s.notifications_enabled);
      setReminders(r);
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
  if (!settings || reminders === null) {
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
    </div>
  );
}
