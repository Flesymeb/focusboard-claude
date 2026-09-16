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
