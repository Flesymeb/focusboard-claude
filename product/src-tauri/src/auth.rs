use crate::mail::{password_reset_message, verification_message, MailSink};
use crate::store::{new_id, now_rfc3339, CommandError, CommandResult, Store};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordVerifier};
use chrono::Duration;
use rusqlite::OptionalExtension;
use serde::Serialize;
use sha2::{Digest, Sha256};

const SESSION_TTL_DAYS: i64 = 30;
const VERIFY_TTL_HOURS: i64 = 24;
const RESET_TTL_MINUTES: i64 = 60;
const MAX_FAILED_ATTEMPTS: i64 = 5;
const ATTEMPT_WINDOW_MINUTES: i64 = 15;
const MAX_AUTH_REQUESTS: i64 = 3;
const REQUEST_WINDOW_MINUTES: i64 = 15;
const CURRENT_SESSION_KEY: &str = "current_session_token_hash";

#[derive(Debug, Serialize, Clone)]
pub struct PublicUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub timezone: String,
    pub status: String,
}

struct UserRow {
    id: String,
    email: String,
    password_hash: String,
    display_name: String,
    timezone: String,
    status: String,
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let bytes = hasher.finalize();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hash_password(password: &str) -> CommandResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| CommandError::new("internal_error", format!("Password hashing failed: {e}")))
}

fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

pub fn validate_email(email: &str) -> Result<(), CommandError> {
    let trimmed = email.trim();
    let ok = !trimmed.is_empty()
        && trimmed.len() <= 254
        && !trimmed.contains(char::is_whitespace)
        && trimmed.matches('@').count() == 1;
    if !ok {
        return Err(CommandError::new(
            "invalid_email",
            "Enter a valid email address, like name@example.com.",
        ));
    }
    let domain = trimmed.rsplit('@').next().unwrap_or("");
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err(CommandError::new(
            "invalid_email",
            "Enter a valid email address, like name@example.com.",
        ));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), CommandError> {
    if password.len() < 8 {
        return Err(CommandError::new(
            "weak_password",
            "Use at least 8 characters for your password.",
        ));
    }
    let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_letter || !has_digit {
        return Err(CommandError::new(
            "weak_password",
            "Include at least one letter and one number so the password is harder to guess.",
        ));
    }
    Ok(())
}

fn get_user_by_email(store: &Store, email: &str) -> Option<UserRow> {
    store
        .conn
        .query_row(
            "SELECT id, email, password_hash, display_name, timezone, status
             FROM users WHERE email = ?1 COLLATE NOCASE",
            [email],
            |r| {
                Ok(UserRow {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    password_hash: r.get(2)?,
                    display_name: r.get(3)?,
                    timezone: r.get(4)?,
                    status: r.get(5)?,
                })
            },
        )
        .optional()
        .ok()
        .flatten()
}

fn get_user_by_id(store: &Store, id: &str) -> Option<UserRow> {
    store
        .conn
        .query_row(
            "SELECT id, email, password_hash, display_name, timezone, status
             FROM users WHERE id = ?1",
            [id],
            |r| {
                Ok(UserRow {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    password_hash: r.get(2)?,
                    display_name: r.get(3)?,
                    timezone: r.get(4)?,
                    status: r.get(5)?,
                })
            },
        )
        .optional()
        .ok()
        .flatten()
}

fn public_user(user: &UserRow) -> PublicUser {
    PublicUser {
        id: user.id.clone(),
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        timezone: user.timezone.clone(),
        status: user.status.clone(),
    }
}

fn create_email_token(
    store: &Store,
    user_id: &str,
    purpose: &str,
    ttl: Duration,
) -> CommandResult<String> {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let expires_at =
        (chrono::Utc::now() + ttl).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    store
        .conn
        .execute(
            "INSERT INTO email_tokens(id, user_id, purpose, token_hash, expires_at)
             VALUES(?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                new_id("etk"),
                user_id,
                purpose,
                hash_token(&token),
                expires_at
            ],
        )
        .map_err(|e| {
            CommandError::new(
                "storage_error",
                format!("Could not create verification token: {e}"),
            )
        })?;
    Ok(token)
}

pub fn send_verification_email(
    store: &Store,
    sink: &MailSink,
    verify_base: &str,
    user_id: &str,
    email: &str,
) -> CommandResult<()> {
    let token = create_email_token(
        store,
        user_id,
        "verify_email",
        Duration::hours(VERIFY_TTL_HOURS),
    )?;
    let message = verification_message(email, &token, verify_base);
    sink.deliver(&message)
        .map_err(|e| CommandError::new("mail_delivery_failed", e))
}

fn failed_attempts_since(store: &Store, email: &str, since: &str) -> i64 {
    store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM auth_attempts
             WHERE email = ?1 COLLATE NOCASE AND success = 0 AND attempted_at >= ?2",
            rusqlite::params![email, since],
            |r| r.get(0),
        )
        .unwrap_or(0)
}

fn record_attempt(store: &Store, email: &str, success: bool) -> CommandResult<()> {
    store
        .conn
        .execute(
            "INSERT INTO auth_attempts(email, attempted_at, success) VALUES(?1, ?2, ?3)",
            rusqlite::params![email, now_rfc3339(), success as i64],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not record attempt: {e}"))
        })?;
    if success {
        store
            .conn
            .execute(
                "DELETE FROM auth_attempts WHERE email = ?1 COLLATE NOCASE",
                [email],
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not clear attempts: {e}"))
            })?;
    }
    Ok(())
}

/// Per-email request log for the password-reset and verification-email
/// endpoints. Requests are recorded for every well-formed address — known or
/// not — so rate-limit behavior cannot distinguish registered accounts.
fn auth_requests_since(store: &Store, email: &str, kind: &str, since: &str) -> i64 {
    store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM auth_requests
             WHERE email = ?1 COLLATE NOCASE AND kind = ?2 AND requested_at >= ?3",
            rusqlite::params![email, kind, since],
            |r| r.get(0),
        )
        .unwrap_or(0)
}

fn record_auth_request(store: &Store, email: &str, kind: &str) -> CommandResult<()> {
    store
        .conn
        .execute(
            "INSERT INTO auth_requests(email, kind, requested_at) VALUES(?1, ?2, ?3)",
            rusqlite::params![email.trim(), kind, now_rfc3339()],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not record request: {e}"))
        })?;
    Ok(())
}

fn auth_request_limit_reached(store: &Store, email: &str, kind: &str) -> bool {
    let cutoff = (chrono::Utc::now() - Duration::minutes(REQUEST_WINDOW_MINUTES))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    auth_requests_since(store, email, kind, &cutoff) >= MAX_AUTH_REQUESTS
}

fn create_session(store: &Store, user_id: &str) -> CommandResult<String> {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let expires_at = (chrono::Utc::now() + Duration::days(SESSION_TTL_DAYS))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    store
        .conn
        .execute(
            "INSERT INTO sessions(id, user_id, token_hash, expires_at, revoked, device, created_at)
             VALUES(?1, ?2, ?3, ?4, 0, 'desktop', ?5)",
            rusqlite::params![
                new_id("ses"),
                user_id,
                hash_token(&token),
                expires_at,
                now_rfc3339()
            ],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not create session: {e}"))
        })?;
    store.kv_set(CURRENT_SESSION_KEY, &hash_token(&token))?;
    Ok(token)
}

/// Resolves the active session server-side. The raw token never leaves the
/// process; only its SHA-256 is persisted, and every private command re-checks
/// expiry and revocation through this path.
pub fn current_user(store: &Store) -> CommandResult<PublicUser> {
    let token_hash = store
        .kv_get(CURRENT_SESSION_KEY)
        .ok_or_else(|| CommandError::new("unauthenticated", "Sign in to continue."))?;
    let row: Option<(String, String, i64)> = store
        .conn
        .query_row(
            "SELECT s.user_id, s.expires_at, s.revoked
             FROM sessions s WHERE s.token_hash = ?1",
            [&token_hash],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Session lookup failed: {e}")))?;
    let (user_id, expires_at, revoked) =
        row.ok_or_else(|| CommandError::new("unauthenticated", "Sign in to continue."))?;
    if revoked == 1 {
        let _ = store.kv_delete(CURRENT_SESSION_KEY);
        return Err(CommandError::new(
            "unauthenticated",
            "Your session has ended. Sign in to continue.",
        ));
    }
    let expires = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|_| CommandError::new("internal_error", "Stored session expiry is malformed."))?;
    if chrono::Utc::now() >= expires {
        let _ = store.kv_delete(CURRENT_SESSION_KEY);
        return Err(CommandError::new(
            "session_expired",
            "Your session expired. Sign in to continue.",
        ));
    }
    let user = get_user_by_id(store, &user_id)
        .ok_or_else(|| CommandError::new("unauthenticated", "Sign in to continue."))?;
    if user.status == "disabled" {
        let _ = store.kv_delete(CURRENT_SESSION_KEY);
        return Err(CommandError::new(
            "account_disabled",
            "This account is disabled.",
        ));
    }
    Ok(public_user(&user))
}

/// Explicit server-side gate for every private operation.
pub fn require_user(store: &Store) -> CommandResult<PublicUser> {
    current_user(store)
}

pub fn register(
    store: &Store,
    sink: &MailSink,
    verify_base: &str,
    email: &str,
    password: &str,
    display_name: &str,
) -> CommandResult<String> {
    validate_email(email)?;
    validate_password(password)?;
    let email = email.trim().to_string();
    if let Some(existing) = get_user_by_email(store, &email) {
        if existing.status == "unverified" {
            // Re-issue the verification email for an unclaimed address instead
            // of revealing that the account already exists.
            send_verification_email(store, sink, verify_base, &existing.id, &existing.email)?;
        }
        // Deliberately generic: do not reveal whether the email already exists.
        return Ok("verification_sent".to_string());
    }
    let password_hash = hash_password(password)?;
    store
        .conn
        .execute(
            "INSERT INTO users(id, email, password_hash, display_name, status, created_at)
             VALUES(?1, ?2, ?3, ?4, 'unverified', ?5)",
            rusqlite::params![
                new_id("usr"),
                email,
                password_hash,
                display_name.trim(),
                now_rfc3339()
            ],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not create account: {e}"))
        })?;
    let user = get_user_by_email(store, &email)
        .ok_or_else(|| CommandError::new("internal_error", "Account vanished after creation."))?;
    send_verification_email(store, sink, verify_base, &user.id, &user.email)?;
    Ok("verification_sent".to_string())
}

pub fn verify_email_token(store: &Store, token: &str) -> CommandResult<PublicUser> {
    let token_hash = hash_token(token);
    let row: Option<(String, String, Option<String>)> = store
        .conn
        .query_row(
            "SELECT user_id, expires_at, consumed_at FROM email_tokens
             WHERE token_hash = ?1 AND purpose = 'verify_email'",
            [&token_hash],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Token lookup failed: {e}")))?;
    let Some((user_id, expires_at, consumed_at)) = row else {
        return Err(CommandError::new(
            "invalid_token",
            "This verification link is not valid. Request a new verification email.",
        ));
    };
    if consumed_at.is_some() {
        return Err(CommandError::new(
            "token_already_used",
            "This verification link was already used. Sign in to continue.",
        ));
    }
    let expires = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|_| CommandError::new("internal_error", "Stored token expiry is malformed."))?;
    if chrono::Utc::now() >= expires {
        return Err(CommandError::new(
            "token_expired",
            "This verification link expired. Request a new verification email.",
        ));
    }
    store
        .conn
        .execute(
            "UPDATE email_tokens SET consumed_at = ?1 WHERE token_hash = ?2 AND consumed_at IS NULL",
            rusqlite::params![now_rfc3339(), token_hash],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not consume token: {e}")))?;
    store
        .conn
        .execute(
            "UPDATE users SET status = 'active' WHERE id = ?1 AND status = 'unverified'",
            [&user_id],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not activate account: {e}"))
        })?;
    let user = get_user_by_id(store, &user_id)
        .ok_or_else(|| CommandError::new("internal_error", "Account not found."))?;
    Ok(public_user(&user))
}

pub fn sign_in(store: &Store, email: &str, password: &str) -> CommandResult<PublicUser> {
    validate_email(email)?;
    let email = email.trim().to_string();
    let cutoff = (chrono::Utc::now() - Duration::minutes(ATTEMPT_WINDOW_MINUTES))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    if failed_attempts_since(store, &email, &cutoff) >= MAX_FAILED_ATTEMPTS {
        return Err(CommandError::new(
            "attempt_limit",
            "Too many sign-in attempts. Wait a few minutes and try again.",
        ));
    }
    let Some(user) = get_user_by_email(store, &email) else {
        record_attempt(store, &email, false)?;
        return Err(CommandError::new(
            "invalid_credentials",
            "That email or password doesn't match an account.",
        ));
    };
    if user.status == "disabled" {
        return Err(CommandError::new(
            "account_disabled",
            "This account is disabled. Contact support to restore access.",
        ));
    }
    if !verify_password(password, &user.password_hash) {
        record_attempt(store, &email, false)?;
        return Err(CommandError::new(
            "invalid_credentials",
            "That email or password doesn't match an account.",
        ));
    }
    if user.status == "unverified" {
        // Successful password proof, but the address is not verified yet.
        return Err(CommandError::new(
            "unverified",
            "Verify your email address before signing in. You can request a new verification email.",
        ));
    }
    record_attempt(store, &email, true)?;
    create_session(store, &user.id)?;
    Ok(public_user(&user))
}

pub fn sign_out(store: &Store) -> CommandResult<()> {
    if let Some(token_hash) = store.kv_get(CURRENT_SESSION_KEY) {
        store
            .conn
            .execute(
                "UPDATE sessions SET revoked = 1 WHERE token_hash = ?1",
                [&token_hash],
            )
            .map_err(|e| {
                CommandError::new("storage_error", format!("Could not end session: {e}"))
            })?;
    }
    store.kv_delete(CURRENT_SESSION_KEY)?;
    Ok(())
}

pub fn request_verification_email(
    store: &Store,
    sink: &MailSink,
    verify_base: &str,
    email: &str,
) -> CommandResult<()> {
    validate_email(email)?;
    let email = email.trim().to_string();
    if auth_request_limit_reached(store, &email, "verify_email") {
        return Err(CommandError::new(
            "attempt_limit",
            "Too many email requests. Wait a few minutes and try again.",
        ));
    }
    record_auth_request(store, &email, "verify_email")?;
    if let Some(user) = get_user_by_email(store, &email) {
        if user.status == "unverified" {
            send_verification_email(store, sink, verify_base, &user.id, &user.email)?;
        }
    }
    // Always reported as accepted so the endpoint cannot probe for accounts.
    Ok(())
}

/// Forgot-password entry point. Every well-formed request is recorded against
/// the rate limit and answered identically, whether or not the address is
/// registered — a reset email is delivered only for active accounts.
pub fn request_password_reset(
    store: &Store,
    sink: &MailSink,
    reset_base: &str,
    email: &str,
) -> CommandResult<()> {
    validate_email(email)?;
    let email = email.trim().to_string();
    if auth_request_limit_reached(store, &email, "password_reset") {
        return Err(CommandError::new(
            "attempt_limit",
            "Too many reset requests. Wait a few minutes and try again.",
        ));
    }
    record_auth_request(store, &email, "password_reset")?;
    if let Some(user) = get_user_by_email(store, &email) {
        if user.status == "active" {
            let token = create_email_token(
                store,
                &user.id,
                "password_reset",
                Duration::minutes(RESET_TTL_MINUTES),
            )?;
            let message = password_reset_message(&user.email, &token, reset_base);
            sink.deliver(&message)
                .map_err(|e| CommandError::new("mail_delivery_failed", e))?;
        }
    }
    Ok(())
}

/// Consumes a single-use reset token and completes the password change.
/// Expired, already-used, and unknown tokens each fail with their own
/// recoverable error so the reset page can render the matching state. A
/// completed reset revokes every session the user still holds.
pub fn reset_password(store: &Store, token: &str, new_password: &str) -> CommandResult<PublicUser> {
    let token = token.trim();
    let token_hash = hash_token(token);
    let row: Option<(String, String, Option<String>)> = store
        .conn
        .query_row(
            "SELECT user_id, expires_at, consumed_at FROM email_tokens
             WHERE token_hash = ?1 AND purpose = 'password_reset'",
            [&token_hash],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| CommandError::new("storage_error", format!("Token lookup failed: {e}")))?;
    let Some((user_id, expires_at, consumed_at)) = row else {
        return Err(CommandError::new(
            "invalid_token",
            "This reset link is not valid. Request a fresh reset email.",
        ));
    };
    if consumed_at.is_some() {
        return Err(CommandError::new(
            "token_already_used",
            "This reset link was already used. Request a fresh one to set a new password.",
        ));
    }
    let expires = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|_| CommandError::new("internal_error", "Stored token expiry is malformed."))?;
    if chrono::Utc::now() >= expires {
        return Err(CommandError::new(
            "token_expired",
            "This reset link expired. Request a fresh reset email.",
        ));
    }
    validate_password(new_password)?;
    store
        .conn
        .execute(
            "UPDATE email_tokens SET consumed_at = ?1 WHERE token_hash = ?2 AND consumed_at IS NULL",
            rusqlite::params![now_rfc3339(), token_hash],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not consume token: {e}")))?;
    let password_hash = hash_password(new_password)?;
    store
        .conn
        .execute(
            "UPDATE users SET password_hash = ?1 WHERE id = ?2",
            rusqlite::params![password_hash, user_id],
        )
        .map_err(|e| {
            CommandError::new("storage_error", format!("Could not update password: {e}"))
        })?;
    // The reset invalidates every live session for this account; the current
    // desktop pointer is cleared only when it pointed at one of them.
    store
        .conn
        .execute(
            "UPDATE sessions SET revoked = 1 WHERE user_id = ?1",
            [&user_id],
        )
        .map_err(|e| CommandError::new("storage_error", format!("Could not end sessions: {e}")))?;
    if let Some(current_hash) = store.kv_get(CURRENT_SESSION_KEY) {
        let belongs: Option<i64> = store
            .conn
            .query_row(
                "SELECT 1 FROM sessions WHERE token_hash = ?1 AND user_id = ?2",
                rusqlite::params![current_hash, user_id],
                |r| r.get(0),
            )
            .optional()
            .unwrap_or(None);
        if belongs.is_some() {
            store.kv_delete(CURRENT_SESSION_KEY)?;
        }
    }
    let user = get_user_by_id(store, &user_id)
        .ok_or_else(|| CommandError::new("internal_error", "Account not found."))?;
    Ok(public_user(&user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::MailSink;
    use crate::store::new_id;
    use std::path::PathBuf;

    const RESET_BASE: &str = "focusboard://auth/reset";

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("focusboard-auth-{name}-{}", new_id("t")));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn open_store(dir: &PathBuf) -> Store {
        Store::open(&dir.join("focusboard.sqlite3")).unwrap()
    }

    /// Registers, verifies, signs in, and returns the signed-in store context.
    fn active_account(dir: &PathBuf, sink: &MailSink, email: &str, password: &str) -> Store {
        let store = open_store(dir);
        auth_register_verify(&store, sink, email, password);
        sign_in_ok(&store, email, password);
        store
    }

    fn auth_register_verify(store: &Store, sink: &MailSink, email: &str, password: &str) {
        register(store, sink, "focusboard://auth", email, password, "Ada").unwrap();
        let token = sink_token(sink, email);
        verify_email_token(store, &token).unwrap();
    }

    fn sign_in_ok(store: &Store, email: &str, password: &str) {
        sign_in(store, email, password).unwrap();
    }

    fn sink_token(sink: &MailSink, email: &str) -> String {
        sink.read_messages()
            .iter()
            .filter(|m| m.to == email && crate::token_from_link(&m.action_url).is_some())
            .max_by(|a, b| a.created_at.cmp(&b.created_at))
            .and_then(|m| crate::token_from_link(&m.action_url))
            .expect("expected an email with an action token")
    }

    #[test]
    fn reset_journey_updates_password_and_revokes_sessions() {
        let dir = temp_dir("journey");
        let sink = MailSink::new(&dir);
        let store = active_account(&dir, &sink, "ada@example.com", "tulip-garnet-9");
        assert!(current_user(&store).is_ok());

        request_password_reset(&store, &sink, RESET_BASE, "ada@example.com").unwrap();
        let token = sink_token(&sink, "ada@example.com");

        let user = reset_password(&store, &token, "harbor-quartz-4").unwrap();
        assert_eq!(user.email, "ada@example.com");

        // The reset invalidated the pre-reset session.
        assert!(current_user(&store).is_err());

        // Old password is rejected; the new one signs in durably.
        let err = sign_in(&store, "ada@example.com", "tulip-garnet-9").unwrap_err();
        assert_eq!(err.code, "invalid_credentials");
        sign_in_ok(&store, "ada@example.com", "harbor-quartz-4");
        assert!(current_user(&store).is_ok());

        // Single-use: a second reset with the same token is recoverable.
        let err = reset_password(&store, &token, "another-lumen-8").unwrap_err();
        assert_eq!(err.code, "token_already_used");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_token_expiry_is_recoverable() {
        let dir = temp_dir("expiry");
        let sink = MailSink::new(&dir);
        let store = active_account(&dir, &sink, "grace@example.com", "tulip-garnet-9");

        request_password_reset(&store, &sink, RESET_BASE, "grace@example.com").unwrap();
        let token = sink_token(&sink, "grace@example.com");
        let expired = (chrono::Utc::now() - Duration::minutes(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        store
            .conn
            .execute(
                "UPDATE email_tokens SET expires_at = ?1 WHERE purpose = 'password_reset'",
                [&expired],
            )
            .unwrap();

        let err = reset_password(&store, &token, "harbor-quartz-4").unwrap_err();
        assert_eq!(err.code, "token_expired");
        // The stale credential still works; nothing was changed.
        sign_in_ok(&store, "grace@example.com", "tulip-garnet-9");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn weak_reset_password_keeps_token_usable_with_actionable_error() {
        let dir = temp_dir("policy");
        let sink = MailSink::new(&dir);
        let store = active_account(&dir, &sink, "lin@example.com", "tulip-garnet-9");

        request_password_reset(&store, &sink, RESET_BASE, "lin@example.com").unwrap();
        let token = sink_token(&sink, "lin@example.com");

        let err = reset_password(&store, &token, "short").unwrap_err();
        assert_eq!(err.code, "weak_password");
        assert!(err.message.contains("8 characters"));

        // The token was not consumed by the rejected attempt.
        reset_password(&store, &token, "harbor-quartz-4").unwrap();
        sign_in_ok(&store, "lin@example.com", "harbor-quartz-4");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn forgot_password_response_hides_account_existence() {
        let dir = temp_dir("enumeration");
        let sink = MailSink::new(&dir);
        let store = active_account(&dir, &sink, "ada@example.com", "tulip-garnet-9");
        let before = sink.read_messages().len();

        // Unknown address: accepted identically, no email delivered.
        request_password_reset(&store, &sink, RESET_BASE, "ghost@example.com").unwrap();
        assert_eq!(sink.read_messages().len(), before);

        // Known address: accepted, reset email delivered.
        request_password_reset(&store, &sink, RESET_BASE, "ada@example.com").unwrap();
        assert_eq!(sink.read_messages().len(), before + 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_requests_are_rate_limited_per_email() {
        let dir = temp_dir("ratelimit");
        let sink = MailSink::new(&dir);
        let store = open_store(&dir);

        for _ in 0..MAX_AUTH_REQUESTS {
            request_password_reset(&store, &sink, RESET_BASE, "stranger@example.com").unwrap();
        }
        let err =
            request_password_reset(&store, &sink, RESET_BASE, "stranger@example.com").unwrap_err();
        assert_eq!(err.code, "attempt_limit");
        assert!(sink.read_messages().is_empty());

        // A different address still has its own budget.
        request_password_reset(&store, &sink, RESET_BASE, "other@example.com").unwrap();

        // Verification-email requests share the same protection.
        for _ in 0..MAX_AUTH_REQUESTS {
            request_verification_email(&store, &sink, "focusboard://auth", "v@example.com")
                .unwrap();
        }
        let err = request_verification_email(&store, &sink, "focusboard://auth", "v@example.com")
            .unwrap_err();
        assert_eq!(err.code, "attempt_limit");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_email_carries_single_use_action_link() {
        let dir = temp_dir("message");
        let sink = MailSink::new(&dir);
        let message = password_reset_message("ada@example.com", "tok123", RESET_BASE);
        sink.deliver(&message).unwrap();
        let stored = &sink.read_messages()[0];
        assert_eq!(stored.subject, "Reset your Focusboard password");
        assert!(stored
            .action_url
            .starts_with("focusboard://auth/reset?token=tok123"));
        assert!(!stored.body_text.to_lowercase().contains("password:"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
