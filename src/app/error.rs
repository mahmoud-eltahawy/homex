//! Centralized error reporting.
//!
//! `AppError` is the error type returned by every `#[server]` function. It
//! carries two faces:
//!
//!   * the **user message** — the only thing that crosses the wire,
//!   * the **diagnostic**   — full detail, logged server-side, never sent.
//!
//! Public errors (validation, not-found, "title required") log at `WARN` on
//! the server so they show up without looking like crashes. Internal errors
//! log at `ERROR` with the context and source; the client sees a generic
//! message.
//!
//! `AppError` implements [`server_fn::error::FromServerFnError`], so it can
//! be used directly as the error type of a `#[server]` function. The
//! `message` and `disposition` fields are serialized; the `diagnostic` is
//! not — it never leaves the server.

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::app::constants::messages;

// ─── Disposition ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Disposition {
    /// Retrying the same operation may succeed.
    Transient,
    /// Retrying will produce the identical result.
    Fatal,
}

// ─── Log level ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[cfg(feature = "ssr")]
    fn emit(self, msg: &str) {
        match self {
            Self::Debug => leptos::logging::log!("[error:debug] {msg}"),
            Self::Info => leptos::logging::log!("[error:info] {msg}"),
            Self::Warn => leptos::logging::warn!("[error:warn] {msg}"),
            Self::Error => leptos::logging::error!("[error] {msg}"),
        }
    }

    /// No-op on the client — error logging is a server concern.
    #[cfg(not(feature = "ssr"))]
    fn emit(self, _msg: &str) {}
}

// ─── AppError ────────────────────────────────────────────────────────────────

/// The error type returned by every `#[server]` function.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppError {
    /// What the client will see. Always safe to display.
    pub message: String,
    /// Whether retrying the same operation could succeed.
    pub disposition: Disposition,
    /// Full diagnostic. `None` for purely public errors. Never serialized.
    #[serde(skip)]
    pub diagnostic: Option<String>,
}

impl AppError {
    // ─── Constructors ─────────────────────────────────────────────────────

    /// Public error. Message is safe to display, no hidden diagnostic.
    /// Logged at `WARN` on the server.
    ///
    /// Disposition defaults to `Fatal` (a validation failure won't fix
    /// itself on retry). Chain [`.transient()`] if it should be retryable.
    pub fn public(msg: impl Into<String>) -> Self {
        let message = msg.into();
        LogLevel::Warn.emit(&format!("[public] {message}"));
        Self {
            message,
            disposition: Disposition::Fatal,
            diagnostic: None,
        }
    }

    /// Internal error. Client sees `messages::INTERNAL_ERROR`; the real cause
    /// is logged at `ERROR` with `context` prefixed.
    pub fn internal(context: impl Into<String>, source: impl Display) -> Self {
        let diagnostic = format!("{}: {}", context.into(), source);
        LogLevel::Error.emit(&diagnostic);
        Self {
            message: messages::INTERNAL_ERROR.into(),
            disposition: Disposition::Transient,
            diagnostic: Some(diagnostic),
        }
    }

    /// Internal error with a custom user-facing message.
    pub fn hidden(
        user_msg: impl Into<String>,
        context: impl Into<String>,
        source: impl Display,
    ) -> Self {
        let diagnostic = format!("{}: {}", context.into(), source);
        LogLevel::Error.emit(&diagnostic);
        Self {
            message: user_msg.into(),
            disposition: Disposition::Fatal,
            diagnostic: Some(diagnostic),
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

    /// Mark this error retryable.
    pub fn transient(mut self) -> Self {
        self.disposition = Disposition::Transient;
        self
    }

    /// Mark this error as not worth retrying.
    pub fn fatal(mut self) -> Self {
        self.disposition = Disposition::Fatal;
        self
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

// ─── FromServerFnError ───────────────────────────────────────────────────────

impl server_fn::error::FromServerFnError for AppError {
    type Encoder = server_fn::codec::JsonEncoding;

    fn from_server_fn_error(value: server_fn::error::ServerFnErrorErr) -> Self {
        let diagnostic = format!("server_fn: {value:?}");
        LogLevel::Error.emit(&diagnostic);
        Self {
            message: messages::INTERNAL_ERROR.into(),
            disposition: Disposition::Transient,
            diagnostic: Some(diagnostic),
        }
    }
}

// ─── Conversions for common error types ──────────────────────────────────────

/// Framework-level multipart errors become internal errors with context.
#[cfg(feature = "ssr")]
impl From<multer::Error> for AppError {
    fn from(e: multer::Error) -> Self {
        AppError::internal("multipart", e)
    }
}

// ─── Context extension ───────────────────────────────────────────────────────

/// Attach context to any fallible operation, mapping the error into an
/// `AppError::internal`.
///
///     let row = toasty::create!(...).exec(db).await
///         .context("insert collection")?;
pub trait ErrorContext<T> {
    fn context(self, ctx: impl Into<String>) -> Result<T, AppError>;
}

impl<T, E: Display> ErrorContext<T> for Result<T, E> {
    fn context(self, ctx: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| AppError::internal(ctx, e))
    }
}

// ─── Toasty bridge ───────────────────────────────────────────────────────────

/// Bridge `toasty::Result<T>` to `Result<T, AppError>`.
#[cfg(feature = "ssr")]
pub trait ToastyErr<T> {
    /// Log at `ERROR`, hide the real error behind the generic message.
    fn srv(self) -> Result<T, AppError>;

    /// Same as `srv`, but attach a context string to the log line.
    fn srv_ctx(self, ctx: impl Into<String>) -> Result<T, AppError>;
}

#[cfg(feature = "ssr")]
impl<T> ToastyErr<T> for toasty::Result<T> {
    fn srv(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::internal("db", e))
    }

    fn srv_ctx(self, ctx: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| AppError::internal(ctx, e))
    }
}

// ─── Macros ──────────────────────────────────────────────────────────────────

/// Return a public error. Logs at `WARN` on the server.
#[macro_export]
macro_rules! bail_public {
    ($($arg:tt)*) => {
        return ::std::result::Result::Err(
            $crate::app::error::AppError::public(format!($($arg)*))
        )
    };
}

/// Return an internal error. `context` and `source` go to the server log.
#[macro_export]
macro_rules! bail_internal {
    ($ctx:expr, $err:expr) => {
        return ::std::result::Result::Err($crate::app::error::AppError::internal($ctx, $err))
    };
}

/// Log a server-only error and continue.
#[macro_export]
macro_rules! log_server_error {
    ($ctx:expr, $err:expr) => {
        ::leptos::logging::error!("[server] {}: {}", $ctx, $err)
    };
    ($ctx:expr) => {
        ::leptos::logging::error!("[server] {}", $ctx)
    };
}
