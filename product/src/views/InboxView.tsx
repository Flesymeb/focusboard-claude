import { useEffect, useState } from "react";
import { Project, Task, invoke, toCommandError, todayInTimezone } from "../api";
import { EmptyState, ErrorState, Icon, Loading } from "../ui";
import { TaskRow } from "./TaskRow";

type PriorityFilter = "" | "low" | "normal" | "high";
type DueFilter = "" | "today" | "overdue" | "any" | "none";
type CompletionFilter = "" | "open" | "completed";

/**
 * The Inbox narrows its visible set through a search term plus priority, due,
 * and completion filters. Filtering runs as a parameterized backend query so
 * the narrowed list is authoritative, not a client-side guess.
 */
export function InboxView(props: {
  tasks: Task[] | null;
  projects: Project[];
  timezone: string;
  error: string | null;
  onRetry: () => void;
  onChanged: () => void;
  onQuickAddFocus: () => void;
}) {
  const [search, setSearch] = useState("");
  const [priority, setPriority] = useState<PriorityFilter>("");
  const [due, setDue] = useState<DueFilter>("");
  const [completion, setCompletion] = useState<CompletionFilter>("");
  const [results, setResults] = useState<Task[] | null>(null);
  const [queryError, setQueryError] = useState<string | null>(null);
  const [reload, setReload] = useState(0);

  useEffect(() => {
    let alive = true;
    setResults(null);
    setQueryError(null);
    invoke<Task[]>("query_tasks", {
      search: search.trim() || null,
      priority: priority || null,
      due: due || null,
      status: completion || null,
      today: due ? todayInTimezone(props.timezone) : null,
    })
      .then((rows) => {
        if (alive) setResults(rows);
      })
      .catch((err) => {
        if (alive) setQueryError(toCommandError(err).message);
      });
    return () => {
      alive = false;
    };
  }, [search, priority, due, completion, reload, props.timezone]);

  if (props.error) {
    return <ErrorState message={props.error} onRetry={props.onRetry} />;
  }

  // Before the first query resolves, fall back to the shell's task list so the
  // surface renders immediately instead of flashing a second loading state.
  const tasks = results ?? props.tasks;
  if (tasks === null) {
    return <Loading label="Opening your inbox" />;
  }
  if (queryError) {
    return <ErrorState message={queryError} onRetry={() => setReload((n) => n + 1)} />;
  }

  const filtersActive = search.trim() !== "" || priority !== "" || due !== "" || completion !== "";
  const open = tasks.filter((t) => t.status !== "completed");
  const completed = tasks.filter((t) => t.status === "completed");

  if (tasks.length === 0) {
    return filtersActive ? (
      <div className="view-body">
        <FilterBar
          search={search}
          priority={priority}
          due={due}
          completion={completion}
          onSearch={setSearch}
          onPriority={setPriority}
          onDue={setDue}
          onCompletion={setCompletion}
        />
        <EmptyState
          title="Nothing matches these filters"
          body="No tasks match the current search and filters. Adjust or clear them to see more."
          action={
            <button
              type="button"
              className="button"
              onClick={() => {
                setSearch("");
                setPriority("");
                setDue("");
                setCompletion("");
              }}
              data-testid="inbox-clear-filters"
            >
              Clear filters
            </button>
          }
        />
      </div>
    ) : (
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
      <FilterBar
        search={search}
        priority={priority}
        due={due}
        completion={completion}
        onSearch={setSearch}
        onPriority={setPriority}
        onDue={setDue}
        onCompletion={setCompletion}
      />
      {filtersActive && tasks.length > 0 && (
        <p className="section-note" data-testid="inbox-result-count">
          {tasks.length} {tasks.length === 1 ? "task matches" : "tasks match"} the current filters.
        </p>
      )}
      {open.length === 0 ? (
        <p className="section-note">No open tasks — completed work stays below for reference.</p>
      ) : (
        <ul className="task-list">
          {open.map((t) => (
            <TaskRow key={t.id} task={t} projects={props.projects} onChanged={() => {
              props.onChanged();
              setReload((n) => n + 1);
            }} />
          ))}
        </ul>
      )}
      {completed.length > 0 && (
        <>
          <h2 className="section-title">Completed</h2>
          <ul className="task-list muted">
            {completed.map((t) => (
              <TaskRow key={t.id} task={t} projects={props.projects} onChanged={() => {
                props.onChanged();
                setReload((n) => n + 1);
              }} />
            ))}
          </ul>
        </>
      )}
    </div>
  );
}

function FilterBar(props: {
  search: string;
  priority: PriorityFilter;
  due: DueFilter;
  completion: CompletionFilter;
  onSearch: (v: string) => void;
  onPriority: (v: PriorityFilter) => void;
  onDue: (v: DueFilter) => void;
  onCompletion: (v: CompletionFilter) => void;
}) {
  return (
    <div className="filter-bar" role="search" data-testid="inbox-filters">
      <label className="filter-search">
        <span className="visually-hidden">Search tasks</span>
        <Icon name="search" size={15} />
        <input
          type="search"
          placeholder="Search tasks"
          value={props.search}
          onChange={(e) => props.onSearch(e.target.value)}
          data-testid="inbox-search"
        />
      </label>
      <label className="filter-field">
        <span className="visually-hidden">Filter by priority</span>
        <select
          value={props.priority}
          onChange={(e) => props.onPriority(e.target.value as PriorityFilter)}
          data-testid="filter-priority"
        >
          <option value="">All priorities</option>
          <option value="high">High priority</option>
          <option value="normal">Normal priority</option>
          <option value="low">Low priority</option>
        </select>
      </label>
      <label className="filter-field">
        <span className="visually-hidden">Filter by due date</span>
        <select
          value={props.due}
          onChange={(e) => props.onDue(e.target.value as DueFilter)}
          data-testid="filter-due"
        >
          <option value="">Any due date</option>
          <option value="today">Due today</option>
          <option value="overdue">Overdue</option>
          <option value="any">Has a due date</option>
          <option value="none">No due date</option>
        </select>
      </label>
      <label className="filter-field">
        <span className="visually-hidden">Filter by completion</span>
        <select
          value={props.completion}
          onChange={(e) => props.onCompletion(e.target.value as CompletionFilter)}
          data-testid="filter-completion"
        >
          <option value="">Open and completed</option>
          <option value="open">Open only</option>
          <option value="completed">Completed only</option>
        </select>
      </label>
    </div>
  );
}
