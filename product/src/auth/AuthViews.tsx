import { FormEvent, useEffect, useRef, useState } from "react";
import { PublicUser, invoke, toCommandError } from "../api";

function FieldError({ message }: { message: string | null }) {
  if (!message) return null;
  return (
    <p className="field-error" role="alert">
      {message}
    </p>
  );
}

export function SignInView(props: {
  onSignedIn: (user: PublicUser) => void;
  goTo: (view: "create-account" | "verify" | "forgot-password") => void;
  initialEmail?: string;
  notice?: string | null;
}) {
  const [email, setEmail] = useState(props.initialEmail ?? "");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const user = await invoke<PublicUser>("sign_in", { email, password });
      props.onSignedIn(user);
    } catch (err) {
      const ce = toCommandError(err);
      if (ce.code === "unverified") {
        props.goTo("verify");
        return;
      }
      setError(ce.message);
      setBusy(false);
    }
  }

  return (
    <section className="auth-panel" aria-labelledby="signin-title">
      <h1 id="signin-title">Sign in with email</h1>
      {props.notice ? (
        <p className="auth-notice" role="status">
          {props.notice}
        </p>
      ) : null}
      <form onSubmit={submit} noValidate>
        <label htmlFor="signin-email">Email</label>
        <input
          id="signin-email"
          type="email"
          autoComplete="email"
          placeholder="you@example.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />
        <label htmlFor="signin-password">Password</label>
        <input
          id="signin-password"
          type="password"
          autoComplete="current-password"
          placeholder="Your password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
        <FieldError message={error} />
        <button type="submit" className="button primary block" disabled={busy}>
          {busy ? "Signing in…" : "Continue"}
        </button>
      </form>
      <p className="auth-alt">
        Don't have an account?{" "}
        <button type="button" className="link" onClick={() => props.goTo("create-account")}>
          Create one
        </button>
      </p>
      <p className="auth-alt">
        <button type="button" className="link" onClick={() => props.goTo("forgot-password")}>
          Forgot password?
        </button>
      </p>
    </section>
  );
}

export function CreateAccountView(props: {
  onRegistered: (email: string) => void;
  goTo: (view: "sign-in") => void;
}) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await invoke("register", { email, password, displayName });
      props.onRegistered(email.trim());
    } catch (err) {
      setError(toCommandError(err).message);
      setBusy(false);
    }
  }

  return (
    <section className="auth-panel" aria-labelledby="create-title">
      <h1 id="create-title">Create your account</h1>
      <form onSubmit={submit} noValidate>
        <label htmlFor="create-name">Name</label>
        <input
          id="create-name"
          type="text"
          autoComplete="name"
          placeholder="How should we greet you?"
          value={displayName}
          onChange={(e) => setDisplayName(e.target.value)}
        />
        <label htmlFor="create-email">Email</label>
        <input
          id="create-email"
          type="email"
          autoComplete="email"
          placeholder="you@example.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />
        <label htmlFor="create-password">Password</label>
        <input
          id="create-password"
          type="password"
          autoComplete="new-password"
          placeholder="At least 8 characters"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
          aria-describedby="create-password-hint"
        />
        <p id="create-password-hint" className="hint">
          At least 8 characters, with one letter and one number.
        </p>
        <FieldError message={error} />
        <button type="submit" className="button primary block" disabled={busy}>
          {busy ? "Creating account…" : "Create account"}
        </button>
      </form>
      <p className="auth-alt">
        Already have an account?{" "}
        <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
          Sign in
        </button>
      </p>
    </section>
  );
}

export function VerifyEmailView(props: {
  email: string;
  onVerified: (email: string) => void;
  goTo: (view: "sign-in") => void;
  tokenFromLink?: string | null;
}) {
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const lastAutoVerified = useRef<string | null>(null);

  async function verify(value: string) {
    setBusy(true);
    setError(null);
    try {
      await invoke("verify_email", { token: tokenFromInput(value) });
      props.onVerified(props.email);
    } catch (err) {
      setError(toCommandError(err).message);
      setBusy(false);
    }
  }

  // A link activation verifies immediately; the token value itself is never
  // rendered into the paste field so it stays out of logs and captures.
  useEffect(() => {
    const linkToken = props.tokenFromLink;
    if (!linkToken || lastAutoVerified.current === linkToken) return;
    lastAutoVerified.current = linkToken;
    void verify(linkToken);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.tokenFromLink]);

  async function resend() {
    setStatus(null);
    setError(null);
    try {
      await invoke("request_verification_email", { email: props.email });
      setStatus("A new verification email is on its way to " + props.email + ".");
    } catch (err) {
      setError(toCommandError(err).message);
    }
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (!token.trim()) return;
    await verify(token);
  }

  return (
    <section className="auth-panel" aria-labelledby="verify-title">
      <h1 id="verify-title">Check your email</h1>
      <p className="auth-lead">
        We sent a verification link to <strong>{props.email}</strong>. Open the message from your
        local mail sink and follow the link, or paste the link below.
      </p>
      <form onSubmit={submit} noValidate>
        <label htmlFor="verify-token">Verification link or token</label>
        <input
          id="verify-token"
          type="text"
          placeholder="Paste the link from your email"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          required
        />
        <FieldError message={error} />
        {status ? (
          <p className="auth-notice" role="status">
            {status}
          </p>
        ) : null}
        <button type="submit" className="button primary block" disabled={busy || !token.trim()}>
          {busy ? "Verifying…" : "Verify email"}
        </button>
      </form>
      <p className="auth-alt">
        Didn't get it?{" "}
        <button type="button" className="link" onClick={resend}>
          Send the email again
        </button>{" "}
        ·{" "}
        <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
          Back to sign in
        </button>
      </p>
    </section>
  );
}

/** Extracts a verification token from a pasted focusboard:// link or raw token. */
export function tokenFromInput(input: string): string {
  const value = input.trim();
  const match = value.match(/token=([A-Za-z0-9]+)/);
  return match ? match[1] : value;
}

/** Extracts a token only from a genuine focusboard:// activation link. */
export function tokenFromLinkUrl(url: string): string | null {
  if (!url.startsWith("focusboard://")) return null;
  const match = url.match(/token=([A-Za-z0-9]+)/);
  return match ? match[1] : null;
}

export function ForgotPasswordView(props: { goTo: (view: "sign-in") => void }) {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await invoke("request_password_reset", { email });
      setSent(true);
    } catch (err) {
      setError(toCommandError(err).message);
      setBusy(false);
    }
  }

  if (sent) {
    return (
      <section className="auth-panel" aria-labelledby="forgot-sent-title">
        <h1 id="forgot-sent-title">Check your email</h1>
        <p className="auth-lead">
          If an account exists for <strong>{email.trim()}</strong>, a reset link is on its way. The
          link is single-use and expires in 60 minutes.
        </p>
        <p className="auth-alt">
          <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
            Back to sign in
          </button>
        </p>
      </section>
    );
  }

  return (
    <section className="auth-panel" aria-labelledby="forgot-title">
      <h1 id="forgot-title">Reset your password</h1>
      <p className="auth-lead">
        Enter the email you use for Focusboard and we'll send a reset link.
      </p>
      <form onSubmit={submit} noValidate>
        <label htmlFor="forgot-email">Email</label>
        <input
          id="forgot-email"
          type="email"
          autoComplete="email"
          placeholder="you@example.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />
        <FieldError message={error} />
        <button type="submit" className="button primary block" disabled={busy}>
          {busy ? "Sending…" : "Send reset link"}
        </button>
      </form>
      <p className="auth-alt">
        <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
          Back to sign in
        </button>
      </p>
    </section>
  );
}

type ResetPageState = "form" | "expired" | "used" | "invalid" | "done";

export function ResetPasswordView(props: {
  token: string | null;
  onDone: () => void;
  goTo: (view: "forgot-password" | "sign-in") => void;
}) {
  const [state, setState] = useState<ResetPageState>(props.token ? "form" : "invalid");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (!props.token) return;
    setBusy(true);
    setError(null);
    try {
      await invoke("reset_password", { token: props.token, password });
      setState("done");
    } catch (err) {
      const ce = toCommandError(err);
      if (ce.code === "token_expired") setState("expired");
      else if (ce.code === "token_already_used") setState("used");
      else if (ce.code === "invalid_token") setState("invalid");
      else setError(ce.message);
      setBusy(false);
    }
  }

  function requestFreshLink() {
    props.goTo("forgot-password");
  }

  if (state === "invalid") {
    return (
      <section className="auth-panel" aria-labelledby="reset-title">
        <h1 id="reset-title">This reset link isn't valid</h1>
        <p className="auth-lead">
          The link may be mistyped or already replaced by a newer one. Request a fresh reset email
          and follow its link.
        </p>
        <button type="button" className="button primary block" onClick={requestFreshLink}>
          Request a fresh link
        </button>
        <p className="auth-alt">
          <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
            Back to sign in
          </button>
        </p>
      </section>
    );
  }

  if (state === "expired") {
    return (
      <section className="auth-panel" aria-labelledby="reset-title">
        <h1 id="reset-title">This reset link expired</h1>
        <p className="auth-lead">
          Reset links expire after 60 minutes. Your password is unchanged — request a fresh link to
          set a new one.
        </p>
        <button type="button" className="button primary block" onClick={requestFreshLink}>
          Request a fresh link
        </button>
        <p className="auth-alt">
          <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
            Back to sign in
          </button>
        </p>
      </section>
    );
  }

  if (state === "used") {
    return (
      <section className="auth-panel" aria-labelledby="reset-title">
        <h1 id="reset-title">This reset link was already used</h1>
        <p className="auth-lead">
          Each link works once. If your new password isn't working yet, request a fresh link.
        </p>
        <button type="button" className="button primary block" onClick={requestFreshLink}>
          Request a fresh link
        </button>
        <p className="auth-alt">
          <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
            Back to sign in
          </button>
        </p>
      </section>
    );
  }

  if (state === "done") {
    return (
      <section className="auth-panel" aria-labelledby="reset-done-title">
        <h1 id="reset-done-title">Password updated</h1>
        <p className="auth-lead">
          Your password has been changed and other sessions were signed out.
        </p>
        <button type="button" className="button primary block" onClick={props.onDone}>
          Sign in with your new password
        </button>
      </section>
    );
  }

  return (
    <section className="auth-panel" aria-labelledby="reset-form-title">
      <h1 id="reset-form-title">Set a new password</h1>
      <form onSubmit={submit} noValidate>
        <label htmlFor="reset-password">New password</label>
        <input
          id="reset-password"
          type="password"
          autoComplete="new-password"
          placeholder="At least 8 characters"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
          aria-describedby="reset-password-hint"
        />
        <p id="reset-password-hint" className="hint">
          At least 8 characters, with one letter and one number.
        </p>
        <FieldError message={error} />
        <button type="submit" className="button primary block" disabled={busy || !password}>
          {busy ? "Updating…" : "Update password"}
        </button>
      </form>
      <p className="auth-alt">
        <button type="button" className="link" onClick={() => props.goTo("sign-in")}>
          Back to sign in
        </button>
      </p>
    </section>
  );
}

/** Classifies a focusboard:// activation link by its auth route. */
export function linkKindFromUrl(url: string): "verify" | "reset" | null {
  if (!url.startsWith("focusboard://")) return null;
  if (!/token=[A-Za-z0-9]+/.test(url)) return null;
  return url.includes("auth/reset") ? "reset" : "verify";
}
