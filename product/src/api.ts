export interface PublicUser {
  id: string;
  email: string;
  display_name: string;
  timezone: string;
  status: string;
}

export interface Project {
  id: string;
  name: string;
  description: string;
  accent: string;
  archived: boolean;
  sort_order: number;
  created_at: string;
}

export interface Task {
  id: string;
  project_id: string | null;
  title: string;
  notes: string;
  status: string;
  priority: string;
  due_date: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface Subtask {
  id: string;
  task_id: string;
  title: string;
  done: boolean;
  position: number;
}

export interface ActivityEvent {
  id: string;
  entity_id: string;
  entity_type: string;
  event_type: string;
  summary: string;
  created_at: string;
}

export interface Reminder {
  id: string;
  task_id: string;
  user_id: string;
  scheduled_at: string;
  timezone: string;
  channel: string;
  status: string;
  provider_message_id: string | null;
  sent_at: string | null;
  retry_count: number;
  dedup_key: string;
}

export interface FocusPause {
  paused_at: string;
  resumed_at: string | null;
}

export interface FocusSession {
  id: string;
  task_id: string | null;
  task_title: string | null;
  started_at: string;
  ended_at: string | null;
  state: "running" | "paused" | "completed" | "cancelled";
  pauses: FocusPause[];
  elapsed_seconds: number;
}

export interface Settings {
  email: string;
  display_name: string;
  timezone: string;
  notifications_enabled: boolean;
}

export interface SessionInfo {
  id: string;
  device: string;
  created_at: string;
  expires_at: string;
  current: boolean;
}

export interface CommandErrorShape {
  code: string;
  message: string;
}

/** Normalizes Tauri IPC rejections into a stable {code, message} shape. */
export function toCommandError(err: unknown): CommandErrorShape {
  if (err && typeof err === "object" && "code" in err) {
    const e = err as Partial<CommandErrorShape>;
    return { code: e.code ?? "unknown", message: e.message ?? "Something went wrong." };
  }
  if (err instanceof Error) {
    return { code: "connection_error", message: "Focusboard could not reach its local service." };
  }
  return { code: "unknown", message: "Something went wrong. Try again." };
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export function todayLocal(): string {
  const d = new Date();
  const m = `${d.getMonth() + 1}`.padStart(2, "0");
  const day = `${d.getDate()}`.padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}

export const COMMON_TIMEZONES = [
  "UTC",
  "Asia/Shanghai",
  "Asia/Hong_Kong",
  "Asia/Taipei",
  "Asia/Tokyo",
  "Asia/Singapore",
  "Asia/Kolkata",
  "Asia/Dubai",
  "Europe/London",
  "Europe/Berlin",
  "Europe/Paris",
  "Europe/Moscow",
  "America/New_York",
  "America/Chicago",
  "America/Denver",
  "America/Los_Angeles",
  "Australia/Sydney",
  "Pacific/Auckland",
];

/** Formats an absolute instant as local wall-clock time in a named timezone. */
export function formatInTimezone(instant: string, timezone: string): string {
  try {
    return new Intl.DateTimeFormat("en-GB", {
      timeZone: timezone,
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(instant));
  } catch {
    return instant;
  }
}

/** The YYYY-MM-DD calendar date of an instant in a named timezone. */
export function dateInTimezone(instant: string, timezone: string): string {
  try {
    return new Intl.DateTimeFormat("en-CA", {
      timeZone: timezone,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    }).format(new Date(instant));
  } catch {
    return instant.slice(0, 10);
  }
}

/** Today's YYYY-MM-DD date in a named timezone. */
export function todayInTimezone(timezone: string): string {
  return dateInTimezone(new Date().toISOString(), timezone);
}
