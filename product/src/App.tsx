import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import {
  FocusSession,
  Project,
  PublicUser,
  Reminder,
  Task,
  invoke,
  toCommandError,
} from "./api";
import {
  CreateAccountView,
  ForgotPasswordView,
  ResetPasswordView,
  SignInView,
  VerifyEmailView,
  linkKindFromUrl,
  tokenFromLinkUrl,
} from "./auth/AuthViews";
import { GoldenPathProbe } from "./GoldenPathProbe";
import { CalendarView } from "./views/CalendarView";
import { FocusView } from "./views/FocusView";
import { InboxView } from "./views/InboxView";
import { ProjectsView } from "./views/ProjectsView";
import { SettingsView } from "./views/SettingsView";
import { TodayView } from "./views/TodayView";
import { Icon } from "./ui";

type AuthView = "sign-in" | "create-account" | "verify" | "forgot-password" | "reset";

type Session =
  | { kind: "loading" }
  | {
      kind: "anonymous";
      view: AuthView;
      email: string;
      notice: string | null;
      tokenFromLink: string | null;
    }
  | { kind: "signedIn"; user: PublicUser };

type MainView = "today" | "inbox" | "projects" | "calendar" | "focus" | "settings";

function verifyTokenFromHash(): string | null {
  const hash = window.location.hash;
  const match = hash.match(/[#/]verify\?token=([A-Za-z0-9]+)/) ?? hash.match(/token=([A-Za-z0-9]+)/);
  return match ? match[1] : null;
}

/** Routes a hash-delivered activation link to its verify or reset surface. */
function linkTokenFromHash(): { kind: "verify" | "reset"; token: string } | null {
  const hash = window.location.hash;
  const reset = hash.match(/reset\?token=([A-Za-z0-9]+)/);
  if (reset) return { kind: "reset", token: reset[1] };
  const token = verifyTokenFromHash();
  return token ? { kind: "verify", token } : null;
}

function authViewForLink(kind: "verify" | "reset"): AuthView {
  return kind === "reset" ? "reset" : "verify";
}

export default function App() {
  const [session, setSession] = useState<Session>({ kind: "loading" });
  const [mainView, setMainView] = useState<MainView>("today");
  const [tasks, setTasks] = useState<Task[] | null>(null);
  const [projects, setProjects] = useState<Project[] | null>(null);
  const [reminders, setReminders] = useState<Reminder[]>([]);
  const [focusSessions, setFocusSessions] = useState<FocusSession[]>([]);
  const [listError, setListError] = useState<string | null>(null);
  const [quickAdd, setQuickAdd] = useState("");
  const [quickAddError, setQuickAddError] = useState<string | null>(null);
  const [quickAddFailed, setQuickAddFailed] = useState(false);
  const [quickAddBusy, setQuickAddBusy] = useState(false);
  const quickAddRef = useRef<HTMLInputElement>(null);
  const signedIn = session.kind === "signedIn";

  // Measurement gate for the section-3 first-run path: mounted in every
  // session state, visually hidden, idle until a driver activates it.
  const probe = (
    <GoldenPathProbe
      onAuthChange={(user) =>
        user
          ? setSession({ kind: "signedIn", user })
          : setSession({
              kind: "anonymous",
              view: "sign-in",
              email: "",
              notice: null,
              tokenFromLink: null,
            })
      }
      visitView={(view) => setMainView(view)}
    />
  );

  const bootstrap = useCallback(async () => {
    try {
      const hashLink = linkTokenFromHash();
      const linkUrl = (await invoke<string | null>("take_deep_link")) ?? "";
      const linkToken = hashLink
        ? hashLink.token
        : tokenFromLinkUrl(linkUrl);
      const linkKind = hashLink?.kind ?? linkKindFromUrl(linkUrl);
      const user = await invoke<PublicUser | null>("session_status");
      if (user) {
        setSession({ kind: "signedIn", user });
      } else {
        setSession({
          kind: "anonymous",
          view: linkToken && linkKind ? authViewForLink(linkKind) : "sign-in",
          email: "",
          notice: null,
          tokenFromLink: linkToken,
        });
      }
    } catch (err) {
      const ce = toCommandError(err);
      setSession({
        kind: "anonymous",
        view: "sign-in",
        email: "",
        notice: ce.code === "connection_error" ? ce.message : null,
        tokenFromLink: null,
      });
    }
  }, []);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  // Warm deep-link activation: the running instance is focused by the OS and
  // the delivered link is routed to the verify view for the anonymous shell.
  useEffect(() => {
    let disposed = false;
    let dispose: (() => void) | undefined;
    void import("@tauri-apps/api/event").then(({ listen }) =>
      listen<string>("deep-link", (event) => {
        const url = event.payload ?? "";
        const token = tokenFromLinkUrl(url);
        if (!token) return;
        const view = authViewForLink(linkKindFromUrl(url) ?? "verify");
        setSession((prev) =>
          prev.kind === "anonymous"
            ? { ...prev, view, tokenFromLink: token }
            : prev,
        );
      }),
    ).then((unlisten) => {
      if (disposed) unlisten();
      else dispose = unlisten;
    });
    return () => {
      disposed = true;
      dispose?.();
    };
  }, []);

  const refreshLists = useCallback(async () => {
    if (session.kind !== "signedIn") return;
    setListError(null);
    try {
      const [t, p, r, f] = await Promise.all([
        invoke<Task[]>("list_tasks"),
        invoke<Project[]>("list_projects"),
        invoke<Reminder[]>("list_reminders").catch(() => [] as Reminder[]),
        invoke<FocusSession[]>("list_focus_sessions").catch(() => [] as FocusSession[]),
      ]);
      setTasks(t);
      setProjects(p);
      setReminders(r);
      setFocusSessions(f);
      // Settings can change the profile; keep the session's user snapshot in
      // step so Today/Calendar render with the saved timezone immediately.
      try {
        const u = await invoke<PublicUser | null>("session_status");
        if (u) {
          setSession((prev) =>
            prev.kind === "signedIn" &&
            (prev.user.timezone !== u.timezone || prev.user.display_name !== u.display_name)
              ? { kind: "signedIn", user: u }
              : prev
          );
        }
      } catch {
        // Profile sync is best-effort; the lists above are authoritative.
      }
    } catch (err) {
      const ce = toCommandError(err);
      if (ce.code === "unauthenticated" || ce.code === "session_expired") {
        setSession({
          kind: "anonymous",
          view: "sign-in",
          email: "",
          notice: "Your session ended. Sign in to continue — your draft is kept.",
          tokenFromLink: null,
        });
        return;
      }
      setListError(ce.message);
    }
  }, [session.kind]);

  const userId = session.kind === "signedIn" ? session.user.id : null;

  useEffect(() => {
    if (!userId) return;
    setTasks(null);
    setProjects(null);
    setReminders([]);
    setFocusSessions([]);
    void refreshLists();
    // Depend on the user identity only: refreshLists is stable per session kind.
  }, [userId]);

  // "N" focuses quick-add from anywhere outside a text field.
  useEffect(() => {
    if (!signedIn) return;
    function onKey(e: KeyboardEvent) {
      const target = e.target as HTMLElement | null;
      const typing =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable);
      if (!typing && (e.key === "n" || e.key === "N") && !e.metaKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        quickAddRef.current?.focus();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [signedIn]);

  // PRD 9.2 network-error state: a failed create keeps the composed text in
  // the input, names the failed action with honest copy, and offers an
  // explicit Retry that re-submits the same text exactly once per click.
  async function runQuickAdd() {
    if (quickAddBusy) return;
    setQuickAddError(null);
    setQuickAddFailed(false);
    const title = quickAdd.trim();
    if (!title) {
      setQuickAddError("A task needs a title before it can be added.");
      return;
    }
    setQuickAddBusy(true);
    try {
      await invoke("create_task", { title });
      setQuickAdd("");
      await refreshLists();
    } catch (err) {
      const ce = toCommandError(err);
      if (ce.code === "unauthenticated" || ce.code === "session_expired") {
        setSession({
          kind: "anonymous",
          view: "sign-in",
          email: "",
          notice: "Your session ended. Sign in to continue — your draft is kept.",
          tokenFromLink: null,
        });
        return;
      }
      // Keep the typed title so valid input is never lost to a rejection.
      const shown = title.length > 40 ? `${title.slice(0, 37)}…` : title;
      setQuickAddError(`Adding “${shown}” failed: ${ce.message}`);
      setQuickAddFailed(true);
    } finally {
      setQuickAddBusy(false);
    }
  }

  async function submitQuickAdd(e: FormEvent) {
    e.preventDefault();
    await runQuickAdd();
  }

  function resetToSignedOut(notice: string) {
    setTasks(null);
    setProjects(null);
    setReminders([]);
    setFocusSessions([]);
    setQuickAdd("");
    setQuickAddError(null);
    setQuickAddFailed(false);
    setMainView("today");
    setSession({
      kind: "anonymous",
      view: "sign-in",
      email: "",
      notice,
      tokenFromLink: null,
    });
  }

  async function signOut() {
    try {
      await invoke("sign_out");
    } catch {
      // Sign-out is best-effort; the UI routes to sign-in regardless.
    }
    resetToSignedOut("You're signed out.");
  }

  if (session.kind === "loading") {
    return (
      <>
        {probe}
        <div className="auth-shell">
          <div className="auth-welcome">
            <p className="brand">
              <Icon name="projects" size={22} /> Focusboard
            </p>
          </div>
        </div>
      </>
    );
  }

  if (session.kind === "anonymous") {
    const { view, email, notice, tokenFromLink } = session;
    return (
      <>
        {probe}
        <div className="auth-shell">
        <div className="auth-welcome">
          <p className="brand">
            <Icon name="projects" size={22} /> Focusboard
          </p>
          <h1>Welcome.</h1>
          <p className="auth-tagline">
            Sign in to plan, focus, and get your best work done.
          </p>
        </div>
        {view === "sign-in" && (
          <SignInView
            initialEmail={email}
            notice={notice}
            onSignedIn={(user) => setSession({ kind: "signedIn", user })}
            goTo={(v) =>
              setSession({
                kind: "anonymous",
                view: v,
                email,
                notice: null,
                tokenFromLink,
              })
            }
          />
        )}
        {view === "create-account" && (
          <CreateAccountView
            goTo={(v) =>
              setSession({
                kind: "anonymous",
                view: v,
                email,
                notice: null,
                tokenFromLink,
              })
            }
            onRegistered={(registeredEmail) =>
              setSession({
                kind: "anonymous",
                view: "verify",
                email: registeredEmail,
                notice: null,
                tokenFromLink: null,
              })
            }
          />
        )}
        {view === "verify" && (
          <VerifyEmailView
            email={email || "your inbox"}
            tokenFromLink={tokenFromLink}
            onVerified={() =>
              setSession({
                kind: "anonymous",
                view: "sign-in",
                email,
                notice: "Email verified. Sign in to continue.",
                tokenFromLink: null,
              })
            }
            goTo={(v) =>
              setSession({
                kind: "anonymous",
                view: v,
                email,
                notice: null,
                tokenFromLink: null,
              })
            }
          />
        )}
        {view === "forgot-password" && (
          <ForgotPasswordView
            goTo={(v) =>
              setSession({
                kind: "anonymous",
                view: v,
                email,
                notice: null,
                tokenFromLink: null,
              })
            }
          />
        )}
        {view === "reset" && (
          <ResetPasswordView
            token={tokenFromLink}
            onDone={() =>
              setSession({
                kind: "anonymous",
                view: "sign-in",
                email,
                notice: "Password updated. Sign in with your new password.",
                tokenFromLink: null,
              })
            }
            goTo={(v) =>
              setSession({
                kind: "anonymous",
                view: v,
                email,
                notice: null,
                tokenFromLink: null,
              })
            }
          />
        )}
        </div>
      </>
    );
  }

  const inboxCount = tasks?.filter((t) => t.status !== "completed").length ?? 0;

  const navItems: { id: MainView; label: string; icon: "today" | "inbox" | "projects" | "calendar" | "focus" | "settings"; badge?: number }[] = [
    { id: "today", label: "Today", icon: "today" },
    { id: "inbox", label: "Inbox", icon: "inbox", badge: inboxCount },
    { id: "projects", label: "Projects", icon: "projects" },
    { id: "calendar", label: "Calendar", icon: "calendar" },
    { id: "focus", label: "Focus", icon: "focus" },
    { id: "settings", label: "Settings", icon: "settings" },
  ];

  // The probe must stay the stable first child of every session branch: a
  // root-element type change here remounts it mid-drive and erases the
  // terminal state the host replay reads.
  return (
    <>
      {probe}
      <div className="app-shell">
        <aside className="sidebar">
        <p className="brand">
          <Icon name="projects" size={20} /> Focusboard
        </p>
        <nav aria-label="Primary">
          <ul>
            {navItems.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  className={`nav-item${mainView === item.id ? " active" : ""}`}
                  aria-current={mainView === item.id ? "page" : undefined}
                  data-testid={`nav-${item.id}`}
                  onClick={() => setMainView(item.id)}
                >
                  <Icon name={item.icon} />
                  <span>{item.label}</span>
                  {item.badge ? <span className="badge">{item.badge}</span> : null}
                </button>
              </li>
            ))}
          </ul>
        </nav>
        <div className="sidebar-footer">
          <button type="button" className="nav-item" onClick={signOut}>
            <Icon name="settings" />
            <span>Sign out</span>
          </button>
        </div>
      </aside>

      <main className="content">
        <form className="quick-add" onSubmit={submitQuickAdd} noValidate>
          <label className="visually-hidden" htmlFor="quick-add-input">
            Quick add a task
          </label>
          <input
            id="quick-add-input"
            ref={quickAddRef}
            type="text"
            placeholder="Add a task — press Enter to capture it"
            data-testid="quick-add-input"
            value={quickAdd}
            onChange={(e) => {
              setQuickAdd(e.target.value);
              if (quickAddError) setQuickAddError(null);
              if (quickAddFailed) setQuickAddFailed(false);
            }}
          />
          <button type="submit" className="button primary" data-testid="quick-add-submit" disabled={quickAddBusy}>
            <Icon name="plus" size={15} /> Add task
          </button>
        </form>
        {quickAddError ? (
          <div
            className="quick-add-failure"
            role="alert"
            data-testid="quick-add-failure"
            data-failed={quickAddFailed ? "true" : "false"}
          >
            <p className="field-error quick-add-error">{quickAddError}</p>
            {quickAddFailed ? (
              <>
                <p className="quick-add-kept">Your text is kept — Retry adds it once the service responds.</p>
                <button
                  type="button"
                  className="button"
                  data-testid="quick-add-retry"
                  disabled={quickAddBusy}
                  onClick={() => void runQuickAdd()}
                >
                  Retry
                </button>
              </>
            ) : null}
          </div>
        ) : null}

        <header className="view-head">
          <h1>
            {mainView === "today"
              ? "Today"
              : mainView.charAt(0).toUpperCase() + mainView.slice(1)}
          </h1>
        </header>

        {mainView === "today" && (
          <TodayView
            tasks={tasks}
            projects={projects ?? []}
            focusSessions={focusSessions}
            timezone={session.user.timezone}
            error={listError}
            onRetry={() => void refreshLists()}
            onChanged={() => void refreshLists()}
          />
        )}
        {mainView === "inbox" && (
          <InboxView
            tasks={tasks}
            projects={projects ?? []}
            timezone={session.user.timezone}
            error={listError}
            onRetry={() => void refreshLists()}
            onChanged={() => void refreshLists()}
            onQuickAddFocus={() => quickAddRef.current?.focus()}
          />
        )}
        {mainView === "projects" && (
          <ProjectsView
            tasks={tasks}
            projects={projects}
            error={listError}
            onRetry={() => void refreshLists()}
            onChanged={() => void refreshLists()}
            onQuickAddFocus={() => quickAddRef.current?.focus()}
          />
        )}
        {mainView === "calendar" && (
          <CalendarView
            tasks={tasks}
            reminders={reminders}
            timezone={session.user.timezone}
            error={listError}
            onRetry={() => void refreshLists()}
          />
        )}
        {mainView === "focus" && (
          <FocusView
            tasks={tasks}
            timezone={session.user.timezone}
            onChanged={() => void refreshLists()}
          />
        )}
        {mainView === "settings" && (
          <SettingsView
            tasks={tasks ?? []}
            timezone={session.user.timezone}
            onChanged={() => void refreshLists()}
            onSessionEnded={resetToSignedOut}
          />
        )}
      </main>
      </div>
    </>
  );
}
