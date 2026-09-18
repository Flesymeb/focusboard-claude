import { useEffect, useState } from "react";
import {
  ActivityEvent,
  COMMON_TIMEZONES,
  Project,
  Reminder,
  Subtask,
  Task,
  invoke,
  toCommandError,
} from "../api";
import { Icon } from "../ui";

const PRIORITIES = ["low", "normal", "high"] as const;

/**
 * Task detail pane: subtasks and the concise per-task history. Both lists
 * refetch after every mutation so the history always shows what just happened.
 */
function TaskDetail(props: { task: Task; onChanged: () => void; version: number }) {
  const { task } = props;
  const [subtasks, setSubtasks] = useState<Subtask[] | null>(null);
  const [events, setEvents] = useState<ActivityEvent[] | null>(null);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    setSubtasks(null);
    setEvents(null);
    invoke<Subtask[]>("list_subtasks", { taskId: task.id })
      .then((rows) => alive && setSubtasks(rows))
      .catch((err) => alive && setError(toCommandError(err).message));
    invoke<ActivityEvent[]>("list_task_activity", { taskId: task.id })
      .then((rows) => alive && setEvents(rows))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [task.id, props.version]);

  async function run(fn: () => Promise<unknown>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
      props.onChanged();
    } catch (err) {
      setError(toCommandError(err).message);
    } finally {
      setBusy(false);
    }
  }

  const addSubtask = () => {
    const title = draft.trim();
    if (!title) return;
    void run(async () => {
      await invoke("add_subtask", { taskId: task.id, title });
      setDraft("");
    });
  };

  return (
    <div className="task-detail" data-testid="task-detail">
      <section className="task-subtasks" aria-label={`Subtasks for ${task.title}`}>
        <h3 className="detail-heading">Subtasks</h3>
        {subtasks === null ? (
          <p className="section-note">Loading subtasks…</p>
        ) : subtasks.length === 0 ? (
          <p className="section-note">No subtasks yet — break the work into steps below.</p>
        ) : (
          <ul className="subtask-list">
            {subtasks.map((s) => (
              <li key={s.id} className="subtask-row" data-testid="subtask-row" data-subtask-done={s.done}>
                <button
                  type="button"
                  className="task-check small"
                  onClick={() => void run(() => invoke("set_subtask_done", { taskId: task.id, subtaskId: s.id, done: !s.done }))}
                  disabled={busy}
                  data-testid="subtask-check"
                  aria-label={s.done ? `Reopen ${s.title}` : `Complete ${s.title}`}
                >
                  {s.done ? <Icon name="check" size={12} /> : null}
                </button>
                <span className="subtask-title">{s.title}</span>
                <button
                  type="button"
                  className="link danger"
                  onClick={() => void run(() => invoke("remove_subtask", { taskId: task.id, subtaskId: s.id }))}
                  disabled={busy}
                  data-testid="subtask-remove"
                  aria-label={`Remove subtask ${s.title}`}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
        )}
        <form
          className="subtask-add"
          onSubmit={(e) => {
            e.preventDefault();
            addSubtask();
          }}
        >
          <label className="visually-hidden" htmlFor={`subtask-input-${task.id}`}>
            New subtask title
          </label>
          <input
            id={`subtask-input-${task.id}`}
            type="text"
            placeholder="Add a subtask"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            disabled={busy}
            data-testid="subtask-input"
          />
          <button type="submit" className="button small" disabled={busy || !draft.trim()} data-testid="subtask-add">
            <Icon name="plus" size={13} /> Add
          </button>
        </form>
      </section>
      <section className="task-history" aria-label={`History for ${task.title}`}>
        <h3 className="detail-heading">History</h3>
        {events === null ? (
          <p className="section-note">Loading history…</p>
        ) : events.length === 0 ? (
          <p className="section-note">No changes recorded yet.</p>
        ) : (
          <ul className="activity-list" data-testid="activity-history">
            {events.map((e) => (
              <li key={e.id} className="activity-row" data-activity-type={e.event_type}>
                <span className="activity-summary">{e.summary}</span>
                <time className="activity-time" dateTime={e.created_at}>
                  {e.created_at.replace("T", " ").slice(0, 16)}
                </time>
              </li>
            ))}
          </ul>
        )}
      </section>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
    </div>
  );
}

/**
 * A single task row. Mutations are optimistic but every failure reverts the
 * visible value and shows an attributable error, so a rejected mutation never
 * fabricates success.
 */
export function TaskRow(props: {
  task: Task;
  projects: Project[];
  onChanged: () => void;
  showProject?: boolean;
  showDue?: boolean;
  showPriority?: boolean;
  showReminder?: boolean;
  reminder?: Reminder | null;
  defaultTimezone?: string;
}) {
  const { task, projects, onChanged } = props;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [projectValue, setProjectValue] = useState(task.project_id ?? "");
  const [dueValue, setDueValue] = useState(task.due_date ?? "");
  const [priorityValue, setPriorityValue] = useState(task.priority);
  const [detailOpen, setDetailOpen] = useState(false);
  const [version, setVersion] = useState(0);
  const [reminderTime, setReminderTime] = useState("");
  const [reminderTz, setReminderTz] = useState(props.defaultTimezone || "UTC");
  const [reminderNote, setReminderNote] = useState<string | null>(null);

  useEffect(() => {
    setProjectValue(task.project_id ?? "");
    setDueValue(task.due_date ?? "");
    setPriorityValue(task.priority);
  }, [task.project_id, task.due_date, task.priority]);

  const reminder = props.reminder ?? null;
  useEffect(() => {
    if (!reminder) {
      setReminderTime("");
      return;
    }
    const local = new Date(reminder.scheduled_at);
    const pad = (n: number) => `${n}`.padStart(2, "0");
    setReminderTime(`${pad(local.getHours())}:${pad(local.getMinutes())}`);
    setReminderTz(reminder.timezone);
  }, [reminder?.id, reminder?.scheduled_at]);

  async function mutate(fn: () => Promise<unknown>, revert: () => void) {
    setBusy(true);
    setError(null);
    try {
      await fn();
      onChanged();
      setVersion((v) => v + 1);
    } catch (err) {
      setError(toCommandError(err).message);
      revert();
    } finally {
      setBusy(false);
    }
  }

  const toggle = () =>
    mutate(
      () =>
        invoke("set_task_status", {
          taskId: task.id,
          status: task.status === "completed" ? "open" : "completed",
        }),
      () => undefined
    );

  const assign = (value: string) => {
    const previous = projectValue;
    setProjectValue(value);
    mutate(
      () => invoke("assign_task", { taskId: task.id, projectId: value || null }),
      () => setProjectValue(previous)
    );
  };

  const setDue = (value: string) => {
    const previous = dueValue;
    setDueValue(value);
    mutate(
      () => invoke("set_task_due", { taskId: task.id, dueDate: value || null }),
      () => setDueValue(previous)
    );
  };

  const setPriority = (value: string) => {
    const previous = priorityValue;
    setPriorityValue(value);
    mutate(
      () => invoke("set_task_priority", { taskId: task.id, priority: value }),
      () => setPriorityValue(previous)
    );
  };

  const applyReminder = () => {
    if (!reminderTime || !dueValue) {
      setError("A reminder needs a due date and a time.");
      return;
    }
    const moment = `${dueValue}T${reminderTime}`;
    mutate(
      () =>
        invoke("set_task_reminder", {
          taskId: task.id,
          reminderTime: moment,
          timezone: reminderTz,
        }),
      () => undefined
    ).then(() => setReminderNote("Reminder saved."));
  };

  const done = task.status === "completed";
  const showReminder = props.showReminder ?? true;
  const showPriority = props.showPriority ?? true;

  return (
    <li className={`task-row${done ? " done" : ""}`} data-task-id={task.id} data-testid="task-row" data-status={task.status} data-priority={task.priority}>
      <button
        type="button"
        className="task-check"
        onClick={toggle}
        disabled={busy}
        data-testid="task-check"
        aria-label={done ? `Mark ${task.title} as open` : `Complete ${task.title}`}
      >
        {done ? <Icon name="check" size={14} /> : null}
      </button>
      <div className="task-main">
        <button
          type="button"
          className="task-title detail-toggle"
          onClick={() => setDetailOpen((open) => !open)}
          aria-expanded={detailOpen}
          data-testid="task-detail-toggle"
        >
          {task.title}
        </button>
        {showPriority && (
          <label className="task-priority">
            <span className="visually-hidden">Priority for {task.title}</span>
            <select
              value={priorityValue}
              onChange={(e) => setPriority(e.target.value)}
              disabled={busy}
              data-testid="task-priority"
            >
              {PRIORITIES.map((p) => (
                <option key={p} value={p}>
                  {p === "low" ? "Low priority" : p === "normal" ? "Normal priority" : "High priority"}
                </option>
              ))}
            </select>
          </label>
        )}
        {(props.showProject ?? true) && (
          <label className="task-assign">
            <span className="visually-hidden">Project for {task.title}</span>
            <select
              value={projectValue}
              onChange={(e) => assign(e.target.value)}
              disabled={busy || projects.length === 0}
            >
              <option value="">No project</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
        )}
        {(props.showDue ?? true) && (
          <label className="task-due">
            <span className="visually-hidden">Due date for {task.title}</span>
            <input
              type="date"
              value={dueValue}
              onChange={(e) => setDue(e.target.value)}
              disabled={busy}
            />
          </label>
        )}
        {showReminder && (
          <span className="task-reminder">
            <label className="task-reminder-time">
              <span className="visually-hidden">Reminder time for {task.title}</span>
              <input
                type="time"
                value={reminderTime}
                onChange={(e) => setReminderTime(e.target.value)}
                disabled={busy}
                data-testid="reminder-time"
              />
            </label>
            <label className="task-reminder-tz">
              <span className="visually-hidden">Reminder timezone for {task.title}</span>
              <select
                value={reminderTz}
                onChange={(e) => setReminderTz(e.target.value)}
                disabled={busy}
                data-testid="reminder-timezone"
              >
                {!COMMON_TIMEZONES.includes(reminderTz) && (
                  <option value={reminderTz}>{reminderTz}</option>
                )}
                {COMMON_TIMEZONES.map((tz) => (
                  <option key={tz} value={tz}>
                    {tz}
                  </option>
                ))}
              </select>
            </label>
            <button
              type="button"
              className="button small"
              onClick={applyReminder}
              disabled={busy || !reminderTime}
              data-testid="reminder-apply"
            >
              <Icon name="mail" size={13} /> Remind
            </button>
            {reminder ? (
              <span className="reminder-state" data-reminder-status={reminder.status}>
                {reminder.status === "sent"
                  ? "Reminder sent"
                  : reminder.status === "scheduled"
                    ? "Reminder scheduled"
                    : reminder.status === "skipped"
                      ? "Reminder skipped (task done)"
                      : reminder.status === "failed"
                        ? "Reminder failed — retry in Settings"
                        : reminder.status === "cancelled"
                          ? "Reminder cancelled"
                          : reminder.status}
              </span>
            ) : reminderNote ? (
              <span className="reminder-state">{reminderNote}</span>
            ) : null}
          </span>
        )}
        {detailOpen && (
          <TaskDetail task={task} onChanged={onChanged} version={version} />
        )}
      </div>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
    </li>
  );
}
