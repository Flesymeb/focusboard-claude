import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import {
  Project,
  PublicUser,
  Task,
  invoke,
  toCommandError,
} from "./api";
import {
  CreateAccountView,
  SignInView,
  VerifyEmailView,
} from "./auth/AuthViews";
import { InboxView } from "./views/InboxView";
import { ProjectsView } from "./views/ProjectsView";
import { TodayView } from "./views/TodayView";
import { EmptyState, Icon } from "./ui";

type AuthView = "sign-in" | "create-account" | "verify";

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

function DeferredSurface({ name }: { name: string }) {
  return (
    <EmptyState
      title={`${name} is on the way`}
      body={`The ${name.toLowerCase()} surface is planned for a later Focusboard loop. Nothing is lost by waiting — your tasks and projects live in Today, Inbox, and Projects.`}
    />
  );
}

export default function App() {
  const [session, setSession] = useState<Session>({ kind: "loading" });
  const [mainView, setMainView] = useState<MainView>("today");
  const [tasks, setTasks] = useState<Task[] | null>(null);
  const [projects, setProjects] = useState<Project[] | null>(null);
  const [listError, setListError] = useState<string | null>(null);
  const [quickAdd, setQuickAdd] = useState("");
  const [quickAddError, setQuickAddError] = useState<string | null>(null);
  const [quickAddBusy, setQuickAddBusy] = useState(false);
  const quickAddRef = useRef<HTMLInputElement>(null);
  const signedIn = session.kind === "signedIn";

  const bootstrap = useCallback(async () => {
    try {
      const linkToken = verifyTokenFromHash();
      const user = await invoke<PublicUser | null>("session_status");
      if (user) {
        setSession({ kind: "signedIn", user });
      } else {
        setSession({
          kind: "anonymous",
          view: linkToken ? "verify" : "sign-in",
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

  const refreshLists = useCallback(async () => {
    if (session.kind !== "signedIn") return;
    setListError(null);
    try {
      const [t, p] = await Promise.all([
        invoke<Task[]>("list_tasks"),
        invoke<Project[]>("list_projects"),
      ]);
      setTasks(t);
      setProjects(p);
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

  async function submitQuickAdd(e: FormEvent) {
    e.preventDefault();
    if (quickAddBusy) return;
    setQuickAddError(null);
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
      setQuickAddError(ce.message);
    } finally {
      setQuickAddBusy(false);
    }
  }

  async function signOut() {
    try {
      await invoke("sign_out");
    } catch {
      // Sign-out is best-effort; the UI routes to sign-in regardless.
    }
    setTasks(null);
    setProjects(null);
    setQuickAdd("");
    setQuickAddError(null);
    setMainView("today");
    setSession({
      kind: "anonymous",
      view: "sign-in",
      email: "",
      notice: "You're signed out.",
      tokenFromLink: null,
    });
  }

  if (session.kind === "loading") {
    return (
      <div className="auth-shell">
        <div className="auth-welcome">
          <p className="brand">
            <Icon name="projects" size={22} /> Focusboard
          </p>
        </div>
      </div>
    );
  }

  if (session.kind === "anonymous") {
    const { view, email, notice, tokenFromLink } = session;
    return (
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
      </div>
    );
  }

  const inboxCount = tasks?.filter((t) => t.status !== "completed").length ?? 0;

  const navItems: { id: MainView; label: string; icon: "today" | "inbox" | "projects" | "calendar" | "focus" | "settings"; badge?: number; deferred?: boolean }[] = [
    { id: "today", label: "Today", icon: "today" },
    { id: "inbox", label: "Inbox", icon: "inbox", badge: inboxCount },
    { id: "projects", label: "Projects", icon: "projects" },
    { id: "calendar", label: "Calendar", icon: "calendar", deferred: true },
    { id: "focus", label: "Focus", icon: "focus", deferred: true },
    { id: "settings", label: "Settings", icon: "settings", deferred: true },
  ];

  return (
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
                  onClick={() => setMainView(item.id)}
                >
                  <Icon name={item.icon} />
                  <span>{item.label}</span>
                  {item.badge ? <span className="badge">{item.badge}</span> : null}
                  {item.deferred ? <span className="badge soft">soon</span> : null}
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
            value={quickAdd}
            onChange={(e) => {
              setQuickAdd(e.target.value);
              if (quickAddError) setQuickAddError(null);
            }}
          />
          <button type="submit" className="button primary" disabled={quickAddBusy}>
            <Icon name="plus" size={15} /> Add task
          </button>
        </form>
        {quickAddError ? (
          <p className="field-error quick-add-error" role="alert">
            {quickAddError}
          </p>
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
            error={listError}
            onRetry={() => void refreshLists()}
            onChanged={() => void refreshLists()}
          />
        )}
        {mainView === "inbox" && (
          <InboxView
            tasks={tasks}
            projects={projects ?? []}
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
        {mainView === "calendar" && <DeferredSurface name="Calendar" />}
        {mainView === "focus" && <DeferredSurface name="Focus" />}
        {mainView === "settings" && <DeferredSurface name="Settings" />}
      </main>
    </div>
  );
}
