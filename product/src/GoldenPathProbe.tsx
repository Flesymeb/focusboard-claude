import { useEffect, useRef, useState } from "react";
import {
  FocusSession,
  Project,
  PublicUser,
  Reminder,
  Task,
  invoke,
  toCommandError,
  todayLocal,
} from "./api";

type ProbePhase = "idle" | "running" | "pass" | "fail";
type DriveView = "today" | "inbox" | "projects" | "calendar" | "focus" | "settings";

interface DeepLinkStatus {
  pending_now: boolean;
  cold_delivered: number;
  warm_forwarded: number;
  last_verify_outcome: string | null;
  last_reset_outcome: string | null;
}

interface StoreLocation {
  identifier: string;
  data_dir: string;
  sqlite_file: string;
}

interface ProbeReport {
  phase: ProbePhase;
  step: string;
  passed: number;
  reason: string;
  elapsedSeconds: number;
}

export const DRIVE_STEPS = [
  "start",
  "register",
  "verify",
  "sign_in",
  "project",
  "task",
  "reminder",
  "focus",
  "reminder_delivery",
  "reminder_email",
  "complete",
  "views",
  "sign_out",
  "sign_in_again",
  "persistence",
  "terminal_today",
] as const;

const VIEW_SEQUENCE: DriveView[] = [
  "today",
  "inbox",
  "projects",
  "calendar",
  "focus",
  "settings",
];

const VIEW_HEADING: Record<DriveView, string> = {
  today: "Today",
  inbox: "Inbox",
  projects: "Projects",
  calendar: "Calendar",
  focus: "Focus",
  settings: "Settings",
};

// Drive artifact names must never carry internal diagnostic phrasing: the
// replay's product-UI separation scan forbids phrases like "Golden path" in
// visible shell copy, so the driven project and task use neutral names.
const DRIVE_PROJECT_NAME = "Focus demo";
const DRIVE_TASK_TITLE = "Deep work demo task";

/** Codes must stay free of credentials: they name the step, never the data. */
function fail(code: string): never {
  throw { code, message: code };
}

/** "YYYY-MM-DDTHH:mm" two seconds ago in the account's timezone. */
function pastLocalMinute(timezone: string): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: timezone,
    hourCycle: "h23",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).formatToParts(new Date(Date.now() - 2000));
  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? "00";
  return `${get("year")}-${get("month")}-${get("day")}T${get("hour")}:${get("minute")}`;
}

async function pollFor(
  label: string,
  check: () => Promise<boolean>,
  timeoutMs: number,
  intervalMs: number
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (await check()) return;
    if (Date.now() >= deadline) fail(label);
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
}

/**
 * Races a step's IPC call against a bounded timeout so a wedged IPC reply
 * becomes an honest attributed failure instead of a silent hang. The probe
 * must never report "running" forever: a stuck step is a failed step.
 */
async function call<T>(step: string, pending: Promise<T>, timeoutMs = 30000): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const guard = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () => reject({ code: `step_timeout_${step}`, message: "step timeout" }),
      timeoutMs
    );
  });
  try {
    return await Promise.race([pending, guard]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

function waitForHeading(view: DriveView): Promise<void> {
  return pollFor(
    `view_not_rendered_${view}`,
    async () =>
      document.querySelector(".view-head h1")?.textContent === VIEW_HEADING[view],
    3000,
    100
  );
}

async function runDrive(
  onAuthChange: (user: PublicUser | null) => void,
  visitView: (view: DriveView) => void,
  report: (r: ProbeReport) => void
): Promise<void> {
  let index = 0;
  let stepName: string = DRIVE_STEPS[0];
  const startedAt = performance.now();
  const progress = (phase: ProbePhase, step: string, reason = "") =>
    report({
      phase,
      step,
      passed: index,
      reason,
      elapsedSeconds: Math.round((performance.now() - startedAt) / 1000),
    });
  progress("running", stepName);
  try {
    const next = () => {
      index += 1;
      if (index < DRIVE_STEPS.length) {
        stepName = DRIVE_STEPS[index];
        progress("running", stepName);
      }
    };

    await call(stepName, invoke("golden_path_drive_start"));
    await next();

    await call(stepName, invoke("golden_path_drive_register"));
    await next();

    await call(stepName, invoke("golden_path_drive_consume_verification"));
    await next();

    const user = await call(stepName, invoke<PublicUser>("golden_path_drive_sign_in"));
    onAuthChange(user);
    await next();

    const project = await call(
      stepName,
      invoke<Project>("create_project", { name: DRIVE_PROJECT_NAME })
    );
    await next();

    const task = await call(
      stepName,
      invoke<Task>("create_task", {
        title: DRIVE_TASK_TITLE,
        projectId: project.id,
        dueDate: todayLocal(),
      })
    );
    await next();

    await call(
      stepName,
      invoke("set_task_reminder", {
        taskId: task.id,
        reminderTime: pastLocalMinute(user.timezone),
        timezone: user.timezone,
      })
    );
    await next();

    const started = await call(stepName, invoke<FocusSession>("start_focus_session", { taskId: task.id }));
    const paused = await call(stepName, invoke<FocusSession>("pause_focus_session", { sessionId: started.id }));
    if (paused.state !== "paused") fail("focus_not_paused");
    const resumed = await call(stepName, invoke<FocusSession>("resume_focus_session", { sessionId: started.id }));
    if (resumed.state !== "running") fail("focus_not_resumed");
    const finished = await call(stepName, invoke<FocusSession>("finish_focus_session", { sessionId: started.id }));
    if (finished.state !== "completed") fail("focus_not_completed");
    await next();

    await pollFor(
      "reminder_not_delivered",
      async () => {
        const list = await call(stepName, invoke<Reminder[]>("list_reminders"), 10000);
        return list.some((r) => r.task_id === task.id && r.status === "sent");
      },
      20000,
      1000
    );
    await next();

    const emailCheck = await call(
      stepName,
      invoke<{
        addressed_to_drive_account: boolean;
        names_the_task: boolean;
        carries_product_identity: boolean;
        links_back_safely: boolean;
        free_of_tokens_and_passwords: boolean;
      }>("golden_path_check_reminder_email")
    );
    if (
      !emailCheck.addressed_to_drive_account ||
      !emailCheck.names_the_task ||
      !emailCheck.carries_product_identity ||
      !emailCheck.links_back_safely ||
      !emailCheck.free_of_tokens_and_passwords
    ) {
      fail("reminder_email_not_verified");
    }
    await next();

    const done = await call(
      stepName,
      invoke<Task>("set_task_status", {
        taskId: task.id,
        status: "completed",
      })
    );
    if (done.status !== "completed") fail("task_not_completed");
    await next();

    for (const view of VIEW_SEQUENCE) {
      visitView(view);
      await waitForHeading(view);
    }
    visitView("today");
    await waitForHeading("today");
    await next();

    await call(stepName, invoke("sign_out"));
    onAuthChange(null);
    await next();

    const again = await call(stepName, invoke<PublicUser>("golden_path_drive_sign_in"));
    onAuthChange(again);
    await next();

    const tasks = await call(stepName, invoke<Task[]>("list_tasks"));
    if (tasks.length !== 1 || tasks[0].status !== "completed") fail("persistence_tasks");
    const projects = await call(stepName, invoke<Project[]>("list_projects"));
    if (projects.length !== 1) fail("persistence_projects");
    const sessions = await call(stepName, invoke<FocusSession[]>("list_focus_sessions"));
    if (sessions.length !== 1 || sessions[0].state !== "completed") fail("persistence_focus");
    const reminders = await call(stepName, invoke<Reminder[]>("list_reminders"));
    if (reminders.length !== 1 || reminders[0].status !== "sent") fail("persistence_reminders");

    // Terminal contract: the drive ends signed in on Today so the six
    // section-8 surfaces stay reachable through the primary nav for the
    // host's post-drive per-surface capture. No terminal sign-out runs.
    visitView("today");
    await waitForHeading("today");
    await next();

    report({
      phase: "pass",
      step: "done",
      passed: DRIVE_STEPS.length,
      reason: "",
      elapsedSeconds: Math.round((performance.now() - startedAt) / 1000),
    });
  } catch (err) {
    const ce = toCommandError(err);
    progress("fail", stepName, ce.code);
  }
}

function summaryText(report: ProbeReport): string {
  const total = DRIVE_STEPS.length;
  if (report.phase === "idle") return "IDLE";
  if (report.phase === "running") return `RUNNING ${report.passed} of ${total}`;
  if (report.phase === "pass") return `PASS ${total} of ${total}`;
  return `FAIL ${report.passed} of ${total} at ${report.step}: ${report.reason}`;
}

/**
 * Product-owned measurement gate for the PRD section-3 first-run path. The
 * drive starts ONLY from the explicit hidden #golden-path-trigger element —
 * never automatically, so it can never mutate app/auth state before the
 * host replay finishes its anonymous-form assertions. State is honest: pass
 * only after every step succeeded against live IPC and durable storage,
 * otherwise the failing step and its reason code. No password, token, or
 * email content ever reaches this element or its payloads.
 */
export function GoldenPathProbe(props: {
  onAuthChange: (user: PublicUser | null) => void;
  visitView: (view: DriveView) => void;
}) {
  const [report, setReport] = useState<ProbeReport>({ phase: "idle", step: "", passed: 0, reason: "", elapsedSeconds: 0 });
  const [deeplink, setDeeplink] = useState<DeepLinkStatus>({
    pending_now: false,
    cold_delivered: 0,
    warm_forwarded: 0,
    last_verify_outcome: null,
    last_reset_outcome: null,
  });
  const [store, setStore] = useState<StoreLocation | null>(null);
  const driving = useRef(false);
  const propsRef = useRef(props);
  propsRef.current = props;

  const refreshDeeplink = () => {
    invoke<DeepLinkStatus>("deep_link_status")
      .then((status) => setDeeplink(status))
      .catch(() => undefined);
  };
  const refreshDeeplinkRef = useRef(refreshDeeplink);
  refreshDeeplinkRef.current = refreshDeeplink;

  const start = () => {
    if (driving.current) return;
    driving.current = true;
    void runDrive(
      propsRef.current.onAuthChange,
      propsRef.current.visitView,
      (next) => {
        if (next.phase === "pass" || next.phase === "fail") {
          driving.current = false;
          window.setTimeout(() => refreshDeeplinkRef.current(), 0);
        }
        setReport(next);
      }
    );
  };

  useEffect(() => {
    let alive = true;
    // Store location is static per launch: read it once.
    invoke<StoreLocation>("store_location")
      .then((location) => {
        if (alive) setStore(location);
      })
      .catch(() => undefined);
    const id = window.setInterval(() => {
      // Pause during the drive: the measurement gate must not add concurrent
      // IPC contention to the journey it is measuring.
      if (driving.current) return;
      invoke<DeepLinkStatus>("deep_link_status")
        .then((status) => {
          if (alive) setDeeplink(status);
        })
        .catch(() => undefined);
    }, 1500);
    return () => {
      alive = false;
      window.clearInterval(id);
    };
  }, []);

  return (
    <div
      id="golden-path-probe"
      className="golden-path-probe"
      data-goldenpath-state={report.phase}
      data-goldenpath-step={report.step}
      data-goldenpath-passed={report.passed}
      data-goldenpath-total={DRIVE_STEPS.length}
      data-goldenpath-elapsed={report.elapsedSeconds}
      data-goldenpath-source="renderer-self-drive-ipc"
      data-deeplink-cold-delivered={deeplink.cold_delivered}
      data-deeplink-warm-forwarded={deeplink.warm_forwarded}
      data-deeplink-pending={deeplink.pending_now ? "true" : "false"}
      data-deeplink-last-outcome={deeplink.last_verify_outcome ?? ""}
      data-deeplink-last-reset-outcome={deeplink.last_reset_outcome ?? ""}
      data-store-identifier={store?.identifier ?? ""}
      data-store-data-dir={store?.data_dir ?? ""}
      data-store-sqlite-file={store?.sqlite_file ?? ""}
    >
      <span className="golden-path-probe-summary" data-goldenpath-summary="">
        {summaryText(report)}
      </span>
      <button
        type="button"
        id="golden-path-trigger"
        data-testid="golden-path-trigger"
        onClick={start}
        tabIndex={-1}
        aria-hidden="true"
      />
    </div>
  );
}
