import { useMemo, useState } from "react";
import { Reminder, Task, formatInTimezone } from "../api";
import { EmptyState, Icon } from "../ui";

const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

function monthTitle(year: number, month: number): string {
  return new Date(year, month, 1).toLocaleDateString("en-GB", {
    month: "long",
    year: "numeric",
  });
}

/** Monday-first six-week grid of dates in YYYY-MM-DD keys. */
function monthGrid(year: number, month: number): string[] {
  const first = new Date(Date.UTC(year, month, 1));
  const offset = (first.getUTCDay() + 6) % 7;
  const start = new Date(Date.UTC(year, month, 1 - offset));
  const cells: string[] = [];
  for (let i = 0; i < 42; i++) {
    const d = new Date(start.getTime() + i * 86400000);
    const m = `${d.getUTCMonth() + 1}`.padStart(2, "0");
    const day = `${d.getUTCDate()}`.padStart(2, "0");
    cells.push(`${d.getUTCFullYear()}-${m}-${day}`);
  }
  return cells;
}

export function CalendarView(props: {
  tasks: Task[] | null;
  reminders: Reminder[];
  timezone: string;
  error: string | null;
  onRetry: () => void;
}) {
  const { tasks, error } = props;
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth());
  const [mode, setMode] = useState<"month" | "agenda">("month");

  const tasksByDate = useMemo(() => {
    const map = new Map<string, Task[]>();
    for (const t of tasks ?? []) {
      if (!t.due_date) continue;
      const list = map.get(t.due_date) ?? [];
      list.push(t);
      map.set(t.due_date, list);
    }
    return map;
  }, [tasks]);

  const todayKey = useMemo(() => {
    const pad = (n: number) => `${n}`.padStart(2, "0");
    return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
  }, []);

  const remindersByTask = useMemo(() => {
    const map = new Map<string, Reminder>();
    for (const r of props.reminders) {
      // Show the most recent reminder per task.
      const existing = map.get(r.task_id);
      if (!existing || r.scheduled_at > existing.scheduled_at) map.set(r.task_id, r);
    }
    return map;
  }, [props.reminders]);

  if (error) {
    return (
      <div className="error-state" role="alert">
        <p>{error}</p>
        <button type="button" className="button" onClick={props.onRetry}>
          Retry
        </button>
      </div>
    );
  }
  if (tasks === null) {
    return (
      <div className="loading" role="status">
        <span className="loading-dot" aria-hidden="true" /> Loading the calendar…
      </div>
    );
  }

  const agenda = (tasks ?? [])
    .filter((t) => t.due_date)
    .sort((a, b) => (a.due_date! < b.due_date! ? -1 : a.due_date! > b.due_date! ? 1 : 0));

  const move = (delta: number) => {
    const next = new Date(year, month + delta, 1);
    setYear(next.getFullYear());
    setMonth(next.getMonth());
  };

  return (
    <div className="view-body calendar-view">
      <div className="calendar-toolbar">
        <div className="calendar-nav">
          <button type="button" className="button small" aria-label="Previous month" onClick={() => move(-1)}>
            ‹
          </button>
          <span className="calendar-title">{monthTitle(year, month)}</span>
          <button type="button" className="button small" aria-label="Next month" onClick={() => move(1)}>
            ›
          </button>
        </div>
        <div className="view-toggle" role="tablist" aria-label="Calendar mode">
          <button
            type="button"
            role="tab"
            aria-selected={mode === "month"}
            className={`button small${mode === "month" ? " primary" : ""}`}
            onClick={() => setMode("month")}
          >
            Month
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={mode === "agenda"}
            className={`button small${mode === "agenda" ? " primary" : ""}`}
            onClick={() => setMode("agenda")}
          >
            Agenda
          </button>
        </div>
      </div>

      {mode === "month" ? (
        tasks.length === 0 ? (
          <EmptyState
            title="No scheduled work yet"
            body="Give a task a due date and it will appear on the day it belongs to."
            hint="Quick add is always one keystroke away — press N"
          />
        ) : (
          <div className="calendar-grid" role="grid" aria-label="Month calendar" data-testid="calendar-grid">
            {WEEKDAYS.map((d) => (
              <div key={d} className="calendar-weekday" role="columnheader">
                {d}
              </div>
            ))}
            {monthGrid(year, month).map((key) => {
              const dayNum = Number(key.slice(8));
              const inMonth = Number(key.slice(5, 7)) === month + 1;
              const dayTasks = tasksByDate.get(key) ?? [];
              return (
                <div
                  key={key}
                  role="gridcell"
                  aria-label={key}
                  className={`calendar-cell${inMonth ? "" : " outside"}${key === todayKey ? " today" : ""}`}
                >
                  <span className="calendar-daynum">{dayNum}</span>
                  <ul className="calendar-events">
                    {dayTasks.map((t) => {
                      const r = remindersByTask.get(t.id);
                      return (
                        <li
                          key={t.id}
                          className={`calendar-event${t.status === "completed" ? " done" : ""}`}
                          data-calendar-date={key}
                        >
                          <Icon name="dot" size={9} />
                          <span className="calendar-event-title">{t.title}</span>
                          {r ? (
                            <span className="calendar-event-reminder">
                              {formatInTimezone(r.scheduled_at, r.timezone).slice(-5)} ⏰
                            </span>
                          ) : null}
                        </li>
                      );
                    })}
                  </ul>
                </div>
              );
            })}
          </div>
        )
      ) : agenda.length === 0 ? (
        <EmptyState
          title="Nothing on the agenda"
          body="Tasks with due dates line up here in order, with their reminder times."
        />
      ) : (
        <ul className="agenda-list" data-testid="calendar-agenda">
          {agenda.map((t) => {
            const r = remindersByTask.get(t.id);
            return (
              <li key={t.id} className="agenda-row" data-agenda-date={t.due_date ?? undefined}>
                <span className="agenda-date">{t.due_date}</span>
                <span className={`agenda-title${t.status === "completed" ? " done" : ""}`}>{t.title}</span>
                {r ? (
                  <span className="agenda-reminder">
                    <Icon name="mail" size={13} /> {formatInTimezone(r.scheduled_at, r.timezone)} ({r.timezone})
                  </span>
                ) : null}
              </li>
            );
          })}
        </ul>
      )}
      <p className="section-note">All dates are shown in your timezone: {props.timezone}</p>
    </div>
  );
}
