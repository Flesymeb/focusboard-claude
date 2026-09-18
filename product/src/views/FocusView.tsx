import { useCallback, useEffect, useState } from "react";
import {
  FocusSession,
  Task,
  formatInTimezone,
  invoke,
  toCommandError,
} from "../api";
import { EmptyState, Icon } from "../ui";

function fmtDuration(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => `${n}`.padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(sec)}` : `${m}:${pad(sec)}`;
}

const STATE_LABEL: Record<string, string> = {
  running: "Focusing",
  paused: "Paused",
  completed: "Completed",
  cancelled: "Cancelled",
};

export function FocusView(props: {
  tasks: Task[] | null;
  timezone: string;
  onChanged: () => void;
}) {
  const { tasks } = props;
  const [sessions, setSessions] = useState<FocusSession[] | null>(null);
  const [active, setActive] = useState<FocusSession | null>(null);
  const [selectedTask, setSelectedTask] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [tick, setTick] = useState(0);

  const refresh = useCallback(async () => {
    try {
      const [all, current] = await Promise.all([
        invoke<FocusSession[]>("list_focus_sessions"),
        invoke<FocusSession | null>("active_focus_session"),
      ]);
      setSessions(all);
      setActive(current);
      setError(null);
    } catch (err) {
      setError(toCommandError(err).message);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Local ticking clock for the running session; the authoritative state
  // stays server-side and is re-synced after every action.
  useEffect(() => {
    const id = window.setInterval(() => setTick((t) => t + 1), 1000);
    return () => window.clearInterval(id);
  }, []);

  async function act(fn: () => Promise<unknown>) {
    setBusy(true);
    setActionError(null);
    try {
      await fn();
      await refresh();
    } catch (err) {
      setActionError(toCommandError(err).message);
    } finally {
      setBusy(false);
    }
  }

  if (error) {
    return (
      <div className="error-state" role="alert">
        <p>{error}</p>
        <button type="button" className="button" onClick={() => void refresh()}>
          Retry
        </button>
      </div>
    );
  }
  if (tasks === null || sessions === null) {
    return (
      <div className="loading" role="status">
        <span className="loading-dot" aria-hidden="true" /> Preparing focus mode…
      </div>
    );
  }

  const openTasks = tasks.filter((t) => t.status !== "completed");
  const history = sessions.filter((s) => s.state === "completed" || s.state === "cancelled");

  return (
    <div className="view-body focus-view">
      {active ? (
        <section className="focus-active" data-focus-state={active.state} data-testid="focus-active" aria-label="Active focus session">
          <p className="focus-state-line">
            <Icon name="focus" size={16} />
            <span className="focus-state-label">{STATE_LABEL[active.state] ?? active.state}</span>
          </p>
          <h2 className="focus-task-title">{active.task_title ?? "Untitled task"}</h2>
          <p className="focus-elapsed" role="timer" data-testid="focus-timer" aria-label="Elapsed focus time">
            {fmtDuration(active.state === "running" ? active.elapsed_seconds + tick : active.elapsed_seconds)}
          </p>
          <div className="focus-controls">
            {active.state === "running" && (
              <button
                type="button"
                className="button"
                disabled={busy}
                data-testid="focus-pause"
                onClick={() => void act(() => invoke("pause_focus_session", { sessionId: active.id }))}
              >
                Pause
              </button>
            )}
            {active.state === "paused" && (
              <button
                type="button"
                className="button primary"
                disabled={busy}
                data-testid="focus-resume"
                onClick={() => void act(() => invoke("resume_focus_session", { sessionId: active.id }))}
              >
                Resume
              </button>
            )}
            <button
              type="button"
              className="button primary"
              disabled={busy}
              data-testid="focus-finish"
              onClick={() =>
                void act(async () => {
                  await invoke("finish_focus_session", { sessionId: active.id });
                  props.onChanged();
                })
              }
            >
              Finish
            </button>
            <button
              type="button"
              className="button quiet"
              disabled={busy}
              data-testid="focus-cancel"
              onClick={() => void act(() => invoke("cancel_focus_session", { sessionId: active.id }))}
            >
              Cancel session
            </button>
          </div>
          <p className="section-note">
            Started {formatInTimezone(active.started_at, props.timezone)} · {active.pauses.length} pause
            {active.pauses.length === 1 ? "" : "s"}
          </p>
        </section>
      ) : (
        <section className="focus-start" aria-label="Start a focus session">
          <h2 className="section-title">Start a focus session</h2>
          {openTasks.length === 0 ? (
            <EmptyState
              title="Nothing to focus on yet"
              body="Capture or open a task first, then start a session to give it your attention."
            />
          ) : (
            <form
              className="focus-start-form"
              onSubmit={(e) => {
                e.preventDefault();
                if (!selectedTask) return;
                void act(async () => {
                  await invoke("start_focus_session", { taskId: selectedTask });
                  setSelectedTask("");
                });
              }}
            >
              <label className="focus-task-pick">
                <span className="visually-hidden">Task to focus on</span>
                <select
                  value={selectedTask}
                  onChange={(e) => setSelectedTask(e.target.value)}
                  aria-label="Task to focus on"
                  data-testid="focus-task-select"
                >
                  <option value="">Choose a task…</option>
                  {openTasks.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.title}
                    </option>
                  ))}
                </select>
              </label>
              <button type="submit" className="button primary" disabled={busy || !selectedTask} data-testid="focus-start">
                <Icon name="focus" size={15} /> Start
              </button>
            </form>
          )}
        </section>
      )}

      {actionError ? (
        <p className="field-error" role="alert">
          {actionError}
        </p>
      ) : null}

      <section aria-label="Session history" className="focus-history">
        <h2 className="section-title">Session history</h2>
        {history.length === 0 ? (
          <p className="section-note">No finished sessions yet. Completed focus work lands here.</p>
        ) : (
          <ul className="session-list">
            {history.map((s) => (
              <li key={s.id} className="session-row" data-session-state={s.state}>
                <span className="session-task">{s.task_title ?? "Untitled task"}</span>
                <span className="session-when">{formatInTimezone(s.started_at, props.timezone)}</span>
                <span className="session-length">{fmtDuration(s.elapsed_seconds)}</span>
                <span className={`session-state state-${s.state}`}>{STATE_LABEL[s.state] ?? s.state}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
