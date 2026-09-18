import { FocusSession, Project, Task, dateInTimezone, todayInTimezone } from "../api";
import { EmptyState, ErrorState, Loading } from "../ui";
import { TaskRow } from "./TaskRow";

function fmtMinutes(seconds: number): string {
  const m = Math.round(seconds / 60);
  if (m < 1) return "under a minute";
  if (m === 1) return "1 minute";
  return `${m} minutes`;
}

export function TodayView(props: {
  tasks: Task[] | null;
  projects: Project[];
  focusSessions: FocusSession[];
  timezone: string;
  error: string | null;
  onRetry: () => void;
  onChanged: () => void;
}) {
  const { tasks, error } = props;
  if (error) {
    return <ErrorState message={error} onRetry={props.onRetry} />;
  }
  if (tasks === null) {
    return <Loading label="Planning your day" />;
  }
  const today = todayInTimezone(props.timezone);
  const dueToday = tasks.filter(
    (t) => t.status !== "completed" && t.due_date === today
  );
  const overdue = tasks.filter(
    (t) => t.status !== "completed" && t.due_date !== null && t.due_date < today
  );
  const completedToday = tasks.filter(
    (t) =>
      t.status === "completed" &&
      t.completed_at !== null &&
      dateInTimezone(t.completed_at, props.timezone) === today
  );
  const focusToday = props.focusSessions.filter(
    (s) => dateInTimezone(s.started_at, props.timezone) === today
  );
  const focusSeconds = focusToday
    .filter((s) => s.state === "completed")
    .reduce((sum, s) => sum + s.elapsed_seconds, 0);

  if (
    dueToday.length === 0 &&
    overdue.length === 0 &&
    completedToday.length === 0 &&
    focusToday.length === 0
  ) {
    return (
      <EmptyState
        title="Nothing scheduled for today"
        body="Give a task a due date and it will land here on the right morning."
        hint="Tasks without a due date stay in Inbox until you schedule them"
      />
    );
  }

  return (
    <div className="view-body">
      {overdue.length > 0 && (
        <>
          <h2 className="section-title overdue">Overdue</h2>
          <ul className="task-list">
            {overdue.map((t) => (
              <TaskRow key={t.id} task={t} projects={props.projects} onChanged={props.onChanged} />
            ))}
          </ul>
        </>
      )}
      <h2 className="section-title">Due today</h2>
      {dueToday.length === 0 ? (
        <p className="section-note">Nothing left due today.</p>
      ) : (
        <ul className="task-list">
          {dueToday.map((t) => (
            <TaskRow key={t.id} task={t} projects={props.projects} onChanged={props.onChanged} />
          ))}
        </ul>
      )}
      {focusToday.length > 0 && (
        <>
          <h2 className="section-title">Focus work today</h2>
          <ul className="focus-day-list">
            {focusToday.map((s) => (
              <li key={s.id} className="focus-day-row" data-focus-state={s.state}>
                <span className="focus-day-task">{s.task_title ?? "Untitled task"}</span>
                <span className={`session-state state-${s.state}`}>{s.state}</span>
              </li>
            ))}
          </ul>
        </>
      )}
      {(completedToday.length > 0 || focusSeconds > 0) && (
        <>
          <h2 className="section-title">Done today</h2>
          <p className="section-note">
            {completedToday.length === 0
              ? "No tasks completed yet."
              : completedToday.length === 1
                ? "1 task completed."
                : `${completedToday.length} tasks completed.`}
            {focusSeconds > 0
              ? ` ${fmtMinutes(focusSeconds)} of focused work recorded.`
              : ""}
          </p>
          {completedToday.length > 0 && (
            <ul className="task-list muted">
              {completedToday.map((t) => (
                <TaskRow
                  key={t.id}
                  task={t}
                  projects={props.projects}
                  onChanged={props.onChanged}
                  showDue={false}
                />
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  );
}
