import { useEffect, useState } from "react";
import {
  COMMON_TIMEZONES,
  Project,
  Reminder,
  Task,
  invoke,
  toCommandError,
} from "../api";
import { Icon } from "../ui";

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
  showReminder?: boolean;
  reminder?: Reminder | null;
  defaultTimezone?: string;
}) {
  const { task, projects, onChanged } = props;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [projectValue, setProjectValue] = useState(task.project_id ?? "");
  const [dueValue, setDueValue] = useState(task.due_date ?? "");
  const [reminderTime, setReminderTime] = useState("");
  const [reminderTz, setReminderTz] = useState(props.defaultTimezone || "UTC");
  const [reminderNote, setReminderNote] = useState<string | null>(null);

  useEffect(() => {
    setProjectValue(task.project_id ?? "");
    setDueValue(task.due_date ?? "");
  }, [task.project_id, task.due_date]);

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

  return (
    <li className={`task-row${done ? " done" : ""}`} data-task-id={task.id} data-testid="task-row">
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
        <span className="task-title">{task.title}</span>
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
      </div>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
    </li>
  );
}
