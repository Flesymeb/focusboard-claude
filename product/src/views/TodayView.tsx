import { Project, Task, todayLocal } from "../api";
import { EmptyState, ErrorState, Loading } from "../ui";
import { TaskRow } from "./TaskRow";

export function TodayView(props: {
  tasks: Task[] | null;
  projects: Project[];
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
  const today = todayLocal();
  const dueToday = tasks.filter(
    (t) => t.status !== "completed" && t.due_date === today
  );
  const overdue = tasks.filter(
    (t) => t.status !== "completed" && t.due_date !== null && t.due_date < today
  );
  const completedToday = tasks.filter(
    (t) => t.status === "completed" && t.completed_at?.slice(0, 10) === today
  );

  if (dueToday.length === 0 && overdue.length === 0 && completedToday.length === 0) {
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
      {completedToday.length > 0 && (
        <>
          <h2 className="section-title">Done today</h2>
          <p className="section-note">
            {completedToday.length === 1
              ? "1 task completed."
              : `${completedToday.length} tasks completed.`}
          </p>
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
        </>
      )}
    </div>
  );
}
