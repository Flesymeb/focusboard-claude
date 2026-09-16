import { useEffect, useState } from "react";
import { Project, Task, invoke, toCommandError } from "../api";
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
}) {
  const { task, projects, onChanged } = props;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [projectValue, setProjectValue] = useState(task.project_id ?? "");
  const [dueValue, setDueValue] = useState(task.due_date ?? "");

  useEffect(() => {
    setProjectValue(task.project_id ?? "");
    setDueValue(task.due_date ?? "");
  }, [task.project_id, task.due_date]);

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

  const done = task.status === "completed";

  return (
    <li className={`task-row${done ? " done" : ""}`} data-task-id={task.id}>
      <button
        type="button"
        className="task-check"
        onClick={toggle}
        disabled={busy}
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
      </div>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
    </li>
  );
}
