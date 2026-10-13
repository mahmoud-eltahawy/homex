//! Centralized error reporting.
//!
//! Every fallible operation on the server funnels through this module. An
//! `AppError` carries two faces:
//!
//!   * the **user message** — the only thing that crosses the wire,
//!   * the **diagnostic**   — full detail, logged server-side, never sent.
//!
//! Public errors (validation, not-found, "title required") have no hidden
//! diagnostic — the user message *is* the truth. Internal errors (DB failure,
//! IO error, unexpected state) hide behind a generic message; the real cause
//! is logged with whatever context the caller supplied.
//!
//! ## Wire format
//!
//! `ServerFnError` (Leptos) offers no structured variant, so we encode a
//! one-char **disposition tag** into the message string itself, delimited by
//! `\u{1}` (SOH). That control character cannot appear in user-authored text,
//! so a client cannot forge a disposition. [`parse_wire_error`] reverses the
//! encoding; [`user_message`] is the safe accessor for display sites.
//!
//! Errors that did not come from `AppError` (raw Leptos/router errors) have
//! no sentinel, and default to [`Disposition::Transient`] — retry is the
//! safer default when we don't know.

use std::fmt::{self, Display};

use leptos::prelude::ServerFnError;

#[cfg(feature = "ssr")]
use crate::app::constants::messages;

// ─── Wire format ─────────────────────────────────────────────────────────────

/// `\u{1}` (SOH). Not producible by any HTML form control or keyboard; safe
/// to use as a delimiter that a client cannot inject.
const SENTINEL: char = '\u{1}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// Retrying the same operation may succeed.
    Transient,
    /// Retrying will produce the identical result.
    Fatal,
}

#[cfg(feature = "ssr")]
fn encode(disposition: Disposition, msg: &str) -> String {
    let tag = match disposition {
        Disposition::Transient => 'T',
        Disposition::Fatal => 'F',
    };
    format!("{SENTINEL}{tag}{SENTINEL}{msg}")
}

/// Parse a wire error string. Returns the disposition and the user-visible
/// message. Strings not produced by `AppError::report` default to
/// [`Disposition::Transient`].
pub fn parse_wire_error(s: &str) -> (Disposition, &str) {
    let Some(rest) = s.strip_prefix(SENTINEL) else {
        return (Disposition::Transient, s);
    };
    let Some((tag, msg)) = rest.split_once(SENTINEL) else {
        return (Disposition::Transient, s);
    };
    let d = match tag {
        "F" => Disposition::Fatal,
        _ => Disposition::Transient,
    };
    (d, msg)
}

/// Extract the user-visible message from a `ServerFnError`. Safe to display.
/// Strips the disposition tag if present.
pub fn user_message(e: &ServerFnError) -> String {
    parse_wire_error(&e.to_string()).1.to_string()
}

/// Extract the disposition of a `ServerFnError`.
pub fn disposition(e: &ServerFnError) -> Disposition {
    parse_wire_error(&e.to_string()).0
}

/// A `Display`-able view of a `ServerFnError` that yields only the
/// user-visible message. Useful inside `view!` bodies where a `Display`
/// is expected.
pub struct UserFacing<'a>(pub &'a ServerFnError);

impl Display for UserFacing<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&user_message(self.0))
    }
}

// ─── Log level ───────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(feature = "ssr")]
impl LogLevel {
    fn emit(self, msg: &str) {
        match self {
            Self::Debug => leptos::logging::log!("[error:debug] {msg}"),
            Self::Info => leptos::logging::log!("[error:info] {msg}"),
            Self::Warn => leptos::logging::warn!("[error:warn] {msg}"),
            Self::Error => leptos::logging::error!("[error] {msg}"),
        }
    }
}

// ─── AppError ────────────────────────────────────────────────────────────────

/// A server-side error with a sanitized user-facing message.
///
/// Note: `AppError` deliberately does **not** implement `std::error::Error`.
/// `server_fn` provides a blanket `impl<E: std::error::Error> From<E> for
/// ServerFnError`, and implementing `Error` here would (a) collide with our
/// own `From<AppError> for ServerFnError`, and (b) if we dropped ours, route
/// through that blanket impl — which uses `Display` and **skips** both the
/// diagnostic logging and the wire-format encoding. The only correct path to
/// the wire is `report()`.
#[cfg(feature = "ssr")]
#[derive(Debug)]
pub struct AppError {
    /// What the client will see. Always safe to display.
    user_message: String,
    /// Full diagnostic. `None` for purely public errors — nothing to hide.
    diagnostic: Option<String>,
    /// Log level when a diagnostic is present. Ignored for public errors.
    level: LogLevel,
    /// Whether retrying the same operation could succeed.
    disposition: Disposition,
}

#[cfg(feature = "ssr")]
impl AppError {
    // ─── Constructors ─────────────────────────────────────────────────────

    /// Public error. Message is safe to display, no hidden diagnostic.
    /// Logged at `WARN` so it appears in server logs without looking like a
    /// crash — user-triggered rejections are still observable.
    ///
    /// Disposition defaults to `Fatal` (a validation failure won't fix
    /// itself on retry). Chain [`.transient()`] if it should be retryable.
    pub fn public(msg: impl Into<String>) -> Self {
        Self {
            user_message: msg.into(),
            diagnostic: None,
            level: LogLevel::Warn,
            disposition: Disposition::Fatal,
        }
    }

    /// Internal error. Client sees `messages::INTERNAL_ERROR`; the real cause
    /// is logged at `ERROR` with `context` prefixed. Disposition defaults to
    /// `Transient` (a DB blip might clear on retry).
    pub fn internal(context: impl Into<String>, source: impl Display) -> Self {
        Self {
            user_message: messages::INTERNAL_ERROR.into(),
            diagnostic: Some(format!("{}: {}", context.into(), source)),
            level: LogLevel::Error,
            disposition: Disposition::Transient,
        }
    }

    /// Internal error with a custom user-facing message. Same logging as
    /// `internal`, but the client sees `user_msg` instead of the generic
    /// `INTERNAL_ERROR`. Use for cases where the user can fix the problem but
    /// we still want to log the technical cause — e.g. a malformed request.
    /// Disposition defaults to `Fatal` (a malformed request stays malformed).
    pub fn hidden(
        user_msg: impl Into<String>,
        context: impl Into<String>,
        source: impl Display,
    ) -> Self {
        Self {
            user_message: user_msg.into(),
            diagnostic: Some(format!("{}: {}", context.into(), source)),
            level: LogLevel::Error,
            disposition: Disposition::Fatal,
        }
    }

    /// `"<what> not found"` — a 404-style public error.
    pub fn not_found(what: &str) -> Self {
        Self::public(format!("{what} not found"))
    }

    /// Public validation failure. Alias for `public` — documents intent.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::public(msg)
    }

    // ─── Builder ──────────────────────────────────────────────────────────

    /// Override the log level of the diagnostic. Ignored for public errors
    /// (they always log at `WARN`).
    pub fn at_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Mark this error retryable.
    pub fn transient(mut self) -> Self {
        self.disposition = Disposition::Transient;
        self
    }

    /// Mark this error as not worth retrying (the default for public errors).
    pub fn fatal(mut self) -> Self {
        self.disposition = Disposition::Fatal;
        self
    }

    // ─── Terminal ─────────────────────────────────────────────────────────

    /// Log the diagnostic (if any) and produce the `ServerFnError` that
    /// crosses the wire.
    pub fn report(self) -> ServerFnError {
        if let Some(diag) = &self.diagnostic {
            self.level.emit(diag);
        }
        ServerFnError::new(encode(self.disposition, &self.user_message))
    }
}

#[cfg(feature = "ssr")]
impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.user_message)
    }
}

// Deliberately no `impl std::error::Error for AppError` — see the struct
// doc-comment. `From<AppError> for ServerFnError` below is the only bridge.

#[cfg(feature = "ssr")]
impl From<AppError> for ServerFnError {
    fn from(e: AppError) -> Self {
        e.report()
    }
}

// ─── Context extension for any `Display` error ───────────────────────────────

/// Attach context to any fallible operation, mapping the error into an
/// `AppError::internal`.
///
///     let row = toasty::create!(...).exec(db).await
///         .context("insert collection")?;
#[cfg(feature = "ssr")]
pub trait ErrorContext<T> {
    fn context(self, ctx: impl Into<String>) -> Result<T, AppError>;
}

#[cfg(feature = "ssr")]
impl<T, E: Display> ErrorContext<T> for Result<T, E> {
    fn context(self, ctx: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| AppError::internal(ctx, e))
    }
}

// ─── Toasty bridge ───────────────────────────────────────────────────────────

/// Bridge `toasty::Result<T>` to `Result<T, ServerFnError>`.
#[cfg(feature = "ssr")]
pub trait ToastyErr<T> {
    /// Log at `ERROR`, hide the real error behind the generic message.
    fn srv(self) -> Result<T, ServerFnError>;

    /// Same as `srv`, but attach a context string to the log line.
    fn srv_ctx(self, ctx: impl Into<String>) -> Result<T, ServerFnError>;
}

#[cfg(feature = "ssr")]
impl<T> ToastyErr<T> for toasty::Result<T> {
    fn srv(self) -> Result<T, ServerFnError> {
        self.map_err(|e| AppError::internal("db", e).report())
    }

    fn srv_ctx(self, ctx: impl Into<String>) -> Result<T, ServerFnError> {
        self.map_err(|e| AppError::internal(ctx, e).report())
    }
}

// ─── Macros ──────────────────────────────────────────────────────────────────

/// Return a public error. Logs at `WARN`, user sees the formatted string.
#[macro_export]
macro_rules! bail_public {
    ($($arg:tt)*) => {
        return ::std::result::Result::Err(
            $crate::app::error::AppError::public(format!($($arg)*)).into()
        )
    };
}

/// Return an internal error. User sees the generic message; `context` and
/// `source` go to the server log at `ERROR`.
#[macro_export]
macro_rules! bail_internal {
    ($ctx:expr, $err:expr) => {
        return ::std::result::Result::Err($crate::app::error::AppError::internal($ctx, $err).into())
    };
}

/// Log a server-only error and continue. For failures the user must not see
/// and that must not abort the operation — background cleanup, best-effort
/// eviction, etc.
#[macro_export]
macro_rules! log_server_error {
    ($ctx:expr, $err:expr) => {
        ::leptos::logging::error!("[server] {}: {}", $ctx, $err)
    };
    ($ctx:expr) => {
        ::leptos::logging::error!("[server] {}", $ctx)
    };
}
