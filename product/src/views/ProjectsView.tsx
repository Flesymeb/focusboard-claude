import { FormEvent, useEffect, useState } from "react";
import { Project, Task, invoke, toCommandError } from "../api";
import { EmptyState, ErrorState, Icon, Loading } from "../ui";

const BOARD_COLUMNS: { id: string; status: string; label: string }[] = [
  { id: "planned", status: "open", label: "Planned" },
  { id: "in-progress", status: "in_progress", label: "In progress" },
  { id: "done", status: "completed", label: "Done" },
];

function statusForColumn(columnId: string): string {
  return BOARD_COLUMNS.find((c) => c.id === columnId)?.status ?? "open";
}

/**
 * One board card. Moves happen either by native drag-and-drop onto a column
 * or through the card's keyboard-reachable column select — both issue exactly
 * one authoritative status command, and a move to the current column is a
 * no-op so no duplicate history entry is written.
 */
function BoardCard(props: {
  task: Task;
  onChanged: () => void;
  onDragStartCard: (taskId: string) => void;
  onDragEndCard: () => void;
}) {
  const { task } = props;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const columnId =
    task.status === "completed" ? "done" : task.status === "in_progress" ? "in-progress" : "planned";

  async function moveTo(target: string) {
    if (target === columnId) return;
    setBusy(true);
    setError(null);
    try {
      await invoke("set_task_status", { taskId: task.id, status: statusForColumn(target) });
      props.onChanged();
    } catch (err) {
      setError(toCommandError(err).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <li
      className={`board-card${task.status === "completed" ? " done" : ""}`}
      data-testid="board-card"
      data-status={task.status}
      draggable={!busy}
      onDragStart={(e) => {
        e.dataTransfer.setData("text/plain", task.id);
        e.dataTransfer.effectAllowed = "move";
        props.onDragStartCard(task.id);
      }}
      onDragEnd={props.onDragEndCard}
    >
      <span className={`priority-dot priority-${task.priority}`} aria-hidden="true" />
      <span className="board-card-title">{task.title}</span>
      <label className="board-move">
        <span className="visually-hidden">Move {task.title} to column</span>
        <select
          value={columnId}
          onChange={(e) => void moveTo(e.target.value)}
          disabled={busy}
          data-testid="board-move"
        >
          {BOARD_COLUMNS.map((c) => (
            <option key={c.id} value={c.id}>
              {c.label}
            </option>
          ))}
        </select>
      </label>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
    </li>
  );
}

function BoardColumn(props: {
  project: Project;
  column: (typeof BOARD_COLUMNS)[number];
  tasks: Task[];
  onChanged: () => void;
  dragTaskId: string | null;
  onDragStartCard: (taskId: string) => void;
  onDragEndCard: () => void;
}) {
  const [dropActive, setDropActive] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function receiveDrop(taskId: string) {
    setDropActive(false);
    const task = props.tasks.find((t) => t.id === taskId);
    // A drop in the card's own column is a no-op: status updates exactly once.
    if (!task || task.status === props.column.status) return;
    setError(null);
    try {
      await invoke("set_task_status", { taskId, status: props.column.status });
      props.onChanged();
    } catch (err) {
      setError(toCommandError(err).message);
    }
  }

  return (
    <section
      className={`board-column${dropActive ? " drop-active" : ""}`}
      aria-label={`${props.column.label} in ${props.project.name}`}
      data-testid={`board-column-${props.column.id}`}
      onDragOver={(e) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = "move";
        setDropActive(true);
      }}
      onDragLeave={() => setDropActive(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDropActive(false);
        const taskId = e.dataTransfer.getData("text/plain") || props.dragTaskId || "";
        if (taskId) void receiveDrop(taskId);
      }}
    >
      <header className="board-column-head">
        <h3>{props.column.label}</h3>
        <span className="board-count">{props.tasks.length}</span>
      </header>
      {props.tasks.length === 0 ? (
        <p className="section-note">Nothing here.</p>
      ) : (
        <ul className="board-card-list">
          {props.tasks.map((t) => (
            <BoardCard
              key={t.id}
              task={t}
              onChanged={props.onChanged}
              onDragStartCard={props.onDragStartCard}
              onDragEndCard={props.onDragEndCard}
            />
          ))}
        </ul>
      )}
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}

function ProjectBoard(props: {
  project: Project;
  tasks: Task[];
  onChanged: () => void;
  onArchive: () => void;
}) {
  const { project, tasks } = props;
  const [renaming, setRenaming] = useState(false);
  const [draft, setDraft] = useState(project.name);
  const [error, setError] = useState<string | null>(null);
  const [dragTaskId, setDragTaskId] = useState<string | null>(null);
  const open = tasks.filter((t) => t.status !== "completed");
  const completed = tasks.filter((t) => t.status === "completed");

  async function submitRename(e: FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await invoke("rename_project", { projectId: project.id, name: draft });
      setRenaming(false);
      props.onChanged();
    } catch (err) {
      setError(toCommandError(err).message);
    }
  }

  async function archive() {
    setError(null);
    try {
      await invoke("set_project_archived", { projectId: project.id, archived: true });
      props.onArchive();
    } catch (err) {
      setError(toCommandError(err).message);
    }
  }

  return (
    <section className="project-board" aria-label={`Project ${project.name}`} data-testid="project-board">
      <header className="project-head">
        <span className={`accent-dot accent-${project.accent}`} aria-hidden="true" />
        {renaming ? (
          <form onSubmit={submitRename} className="rename-form">
            <label className="visually-hidden" htmlFor={`rename-${project.id}`}>
              Project name
            </label>
            <input
              id={`rename-${project.id}`}
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              autoFocus
            />
            <button type="submit" className="button small">
              Save
            </button>
            <button
              type="button"
              className="button small ghost"
              onClick={() => {
                setRenaming(false);
                setDraft(project.name);
                setError(null);
              }}
            >
              Cancel
            </button>
          </form>
        ) : (
          <>
            <h2>{project.name}</h2>
            <button
              type="button"
              className="link"
              onClick={() => {
                setDraft(project.name);
                setRenaming(true);
              }}
            >
              Rename
            </button>
          </>
        )}
        <span className="project-count">
          {open.length} open{completed.length > 0 ? ` · ${completed.length} done` : ""}
        </span>
        <button
          type="button"
          className="link danger board-archive"
          onClick={() => void archive()}
          data-testid="archive-project"
        >
          Archive project
        </button>
      </header>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
      <div className="board-columns">
        {BOARD_COLUMNS.map((column) => (
          <BoardColumn
            key={column.id}
            project={project}
            column={column}
            tasks={tasks.filter((t) => t.status === column.status)}
            onChanged={props.onChanged}
            dragTaskId={dragTaskId}
            onDragStartCard={setDragTaskId}
            onDragEndCard={() => setDragTaskId(null)}
          />
        ))}
      </div>
    </section>
  );
}

/**
 * Projects: a Planned / In progress / Done board per project with drag-and-drop
 * plus a keyboard alternative, and archive/restore that hides a project from
 * active views without touching its tasks or history.
 */
export function ProjectsView(props: {
  tasks: Task[] | null;
  projects: Project[] | null;
  error: string | null;
  onRetry: () => void;
  onChanged: () => void;
  onQuickAddFocus: () => void;
}) {
  const { projects, tasks, error } = props;
  const [name, setName] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [archived, setArchived] = useState<Project[] | null>(null);
  const [restoreError, setRestoreError] = useState<string | null>(null);

  const refreshArchived = () => {
    invoke<Project[]>("list_archived_projects")
      .then(setArchived)
      .catch(() => undefined);
  };

  useEffect(() => {
    refreshArchived();
  }, []);

  if (error) {
    return <ErrorState message={error} onRetry={props.onRetry} />;
  }
  if (projects === null || tasks === null) {
    return <Loading label="Gathering projects" />;
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setCreateError(null);
    try {
      await invoke("create_project", { name });
      setName("");
      props.onChanged();
    } catch (err) {
      setCreateError(toCommandError(err).message);
    } finally {
      setBusy(false);
    }
  }

  async function restore(projectId: string) {
    setRestoreError(null);
    try {
      await invoke("set_project_archived", { projectId, archived: false });
      refreshArchived();
      props.onChanged();
    } catch (err) {
      setRestoreError(toCommandError(err).message);
    }
  }

  return (
    <div className="view-body">
      <form className="inline-create" onSubmit={submit} noValidate>
        <label className="visually-hidden" htmlFor="project-name">
          New project name
        </label>
        <input
          id="project-name"
          type="text"
          placeholder="New project name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button type="submit" className="button primary" disabled={busy}>
          <Icon name="plus" size={15} /> Create project
        </button>
      </form>
      {createError ? (
        <p className="field-error" role="alert">
          {createError}
        </p>
      ) : null}
      {projects.length === 0 ? (
        <EmptyState
          title="No projects yet"
          body="A project groups related tasks and gives them a home beyond the inbox."
          action={
            <button
              type="button"
              className="button primary"
              onClick={() => {
                document.getElementById("project-name")?.focus();
              }}
            >
              Name your first project
            </button>
          }
        />
      ) : (
        projects.map((p) => (
          <ProjectBoard
            key={p.id}
            project={p}
            tasks={tasks.filter((t) => t.project_id === p.id)}
            onChanged={props.onChanged}
            onArchive={() => {
              refreshArchived();
              props.onChanged();
            }}
          />
        ))
      )}
      <section className="archived-projects" aria-label="Archived projects" data-testid="archived-projects">
        <h2 className="section-title">Archived projects</h2>
        {archived === null ? (
          <p className="section-note">Loading archived projects…</p>
        ) : archived.length === 0 ? (
          <p className="section-note">No archived projects — archived work stays here for restoration.</p>
        ) : (
          <ul className="archived-list">
            {archived.map((p) => (
              <li key={p.id} className="archived-row" data-testid="archived-project-row">
                <span className={`accent-dot accent-${p.accent}`} aria-hidden="true" />
                <span className="archived-name">{p.name}</span>
                <button
                  type="button"
                  className="button small"
                  onClick={() => void restore(p.id)}
                  data-testid="restore-project"
                >
                  Restore
                </button>
              </li>
            ))}
          </ul>
        )}
        {restoreError ? (
          <p className="field-error" role="alert">
            {restoreError}
          </p>
        ) : null}
      </section>
    </div>
  );
}
