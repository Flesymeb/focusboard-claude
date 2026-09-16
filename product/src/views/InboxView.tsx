import { Project, Task } from "../api";
import { EmptyState, ErrorState, Loading } from "../ui";
import { TaskRow } from "./TaskRow";

export function InboxView(props: {
  tasks: Task[] | null;
  projects: Project[];
  error: string | null;
  onRetry: () => void;
  onChanged: () => void;
  onQuickAddFocus: () => void;
}) {
  const { tasks, error } = props;
  if (error) {
    return <ErrorState message={error} onRetry={props.onRetry} />;
  }
  if (tasks === null) {
    return <Loading label="Opening your inbox" />;
  }
  const open = tasks.filter((t) => t.status !== "completed");
  const completed = tasks.filter((t) => t.status === "completed");

  if (tasks.length === 0) {
    return (
      <EmptyState
        title="All clear in your Inbox"
        body="Capture tasks, ideas, and requests. They'll show up here."
        action={
          <button type="button" className="button primary" onClick={props.onQuickAddFocus}>
            Add a task
          </button>
        }
        hint="Press N anywhere to add a task"
      />
    );
  }

  return (
    <div className="view-body">
      {open.length === 0 ? (
        <p className="section-note">No open tasks — completed work stays below for reference.</p>
      ) : (
        <ul className="task-list">
          {open.map((t) => (
            <TaskRow key={t.id} task={t} projects={props.projects} onChanged={props.onChanged} />
          ))}
        </ul>
      )}
      {completed.length > 0 && (
        <>
          <h2 className="section-title">Completed</h2>
          <ul className="task-list muted">
            {completed.map((t) => (
              <TaskRow key={t.id} task={t} projects={props.projects} onChanged={props.onChanged} />
            ))}
          </ul>
        </>
      )}
    </div>
  );
}
