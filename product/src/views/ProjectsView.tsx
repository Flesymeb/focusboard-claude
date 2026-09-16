import { FormEvent, useState } from "react";
import { Project, Task, invoke, toCommandError } from "../api";
import { EmptyState, ErrorState, Icon, Loading } from "../ui";
import { TaskRow } from "./TaskRow";

function ProjectGroup(props: {
  project: Project;
  tasks: Task[];
  allProjects: Project[];
  onChanged: () => void;
}) {
  const { project, tasks } = props;
  const [renaming, setRenaming] = useState(false);
  const [draft, setDraft] = useState(project.name);
  const [error, setError] = useState<string | null>(null);
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

  return (
    <section className="project-group" aria-label={`Project ${project.name}`}>
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
      </header>
      {error ? (
        <p className="field-error" role="alert">
          {error}
        </p>
      ) : null}
      {tasks.length === 0 ? (
        <p className="section-note">
          No tasks here yet — assign one from Inbox with the project picker.
        </p>
      ) : (
        <ul className="task-list">
          {open.map((t) => (
            <TaskRow
              key={t.id}
              task={t}
              projects={props.allProjects}
              onChanged={props.onChanged}
            />
          ))}
          {completed.map((t) => (
            <TaskRow
              key={t.id}
              task={t}
              projects={props.allProjects}
              onChanged={props.onChanged}
            />
          ))}
        </ul>
      )}
    </section>
  );
}

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
          <ProjectGroup
            key={p.id}
            project={p}
            tasks={tasks.filter((t) => t.project_id === p.id)}
            allProjects={projects}
            onChanged={props.onChanged}
          />
        ))
      )}
    </div>
  );
}
