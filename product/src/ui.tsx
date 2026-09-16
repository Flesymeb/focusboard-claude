import { ReactNode } from "react";

type IconName =
  | "inbox"
  | "today"
  | "projects"
  | "calendar"
  | "focus"
  | "settings"
  | "search"
  | "mail"
  | "plus"
  | "check"
  | "dot";

const PATHS: Record<IconName, ReactNode> = {
  inbox: <path d="M3 13l2.5-6h13L21 13v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-5zM3 13h5l1.5 2h5L16 13h5" />,
  today: (
    <>
      <rect x="3.5" y="5" width="17" height="15" rx="1.5" />
      <path d="M3.5 9.5h17M8 3v3.5M16 3v3.5" />
    </>
  ),
  projects: (
    <>
      <path d="M3.5 7.5a1.5 1.5 0 0 1 1.5-1.5h4l2 2.5h8a1.5 1.5 0 0 1 1.5 1.5v8A1.5 1.5 0 0 1 19 19.5H5A1.5 1.5 0 0 1 3.5 18V7.5z" />
    </>
  ),
  calendar: (
    <>
      <rect x="3.5" y="5" width="17" height="15" rx="1.5" />
      <path d="M3.5 9.5h17M8 3v3.5M16 3v3.5M8 13h3" />
    </>
  ),
  focus: (
    <>
      <circle cx="12" cy="12" r="7.5" />
      <path d="M12 8v4.5l3 2M12 2.5V4M12 20v1.5" />
    </>
  ),
  settings: (
    <>
      <circle cx="12" cy="12" r="3" />
      <path d="M12 4.5v2M12 17.5v2M4.5 12h2M17.5 12h2M6.7 6.7l1.4 1.4M15.9 15.9l1.4 1.4M6.7 17.3l1.4-1.4M15.9 8.1l1.4-1.4" />
    </>
  ),
  search: (
    <>
      <circle cx="11" cy="11" r="6" />
      <path d="M15.5 15.5L20 20" />
    </>
  ),
  mail: (
    <>
      <rect x="3.5" y="5.5" width="17" height="13" rx="1.5" />
      <path d="M4 7l8 6 8-6" />
    </>
  ),
  plus: <path d="M12 5.5v13M5.5 12h13" />,
  check: <path d="M5 12.5l4.5 4.5L19 7.5" />,
  dot: <circle cx="12" cy="12" r="4" />,
};

export function Icon({ name, size = 18 }: { name: IconName; size?: number }) {
  return (
    <svg
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {PATHS[name]}
    </svg>
  );
}

export function EmptyState(props: {
  title: string;
  body: string;
  action?: ReactNode;
  hint?: string;
}) {
  return (
    <div className="empty-state">
      <div className="empty-state-rule" aria-hidden="true" />
      <h2>{props.title}</h2>
      <p>{props.body}</p>
      {props.action}
      {props.hint ? <p className="hint">{props.hint}</p> : null}
    </div>
  );
}

export function ErrorState(props: { message: string; onRetry: () => void }) {
  return (
    <div className="error-state" role="alert">
      <p>{props.message}</p>
      <button type="button" className="button" onClick={props.onRetry}>
        Retry
      </button>
    </div>
  );
}

export function Loading({ label = "Loading" }: { label?: string }) {
  return (
    <div className="loading" role="status">
      <span className="loading-dot" aria-hidden="true" />
      {label}…
    </div>
  );
}
