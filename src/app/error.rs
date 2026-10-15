//! Centralized error reporting.
//!
//! `AppError` carries two faces:
//!
//! * the **code** — a serializable enum shared by client and server,
//! * the **diagnostic** — full detail, logged server-side, never sent.
//!
//! The message the user reads is derived from `ErrorCode` on demand by
//! [`AppError::user_message`]. That method is the single source of truth
//! for what each error says.

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::app::constants::messages;

// ─── Disposition ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Disposition {
    /// Retrying the same operation may succeed.
    Transient,
    /// Retrying will produce the identical result.
    Fatal,
}

// ─── ErrorCode ────────────────────────────────────────────────────────────

/// Every catalogued error the app can produce. The client renders these
/// by matching on the code, so the wire form and the display form are
/// guaranteed to agree.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCode {
    // Resources
    CollectionNotFound,
    SectionNotFound,
    FileNotFound,
    JobNotFound,

    // Validation
    TitleRequired,
    NameRequired,
    InvalidToken,
    InvalidPath,

    // Upload
    NoFilesReceived,
    NoImageReceived,
    SectionSlugMissing,
    TargetMissing,
    MediaKindMismatch,
    UnsupportedFormat,

    // Sections
    SectionNotEmpty {
        collection_count: usize,
    },
    UniqueSectionIdFailed,

    // Classify
    ClassificationInProgress,
    NoFoldersSelected,
    NoFilesFound,
    NotAFolder,
    NotAFile,
    TooManyFolders {
        count: usize,
        max: usize,
    },

    // Poster
    ImageTooLarge {
        mb: usize,
    },

    /// A user-safe message with runtime detail the catalogue can't express
    /// as a fixed string. The client displays it verbatim.
    Detailed(String),

    /// Generic "something went wrong"; the diagnostic field carries detail.
    Internal,
}

// ─── Log level ────────────────────────────────────────────────────────────

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
    #[cfg(not(feature = "ssr"))]
    fn emit(self, _msg: &str) {}
}

// ─── AppError ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub disposition: Disposition,
    #[serde(skip)]
    pub diagnostic: Option<String>,
}

// ─── Message rendering ────────────────────────────────────────────────────

impl AppError {
    /// The text shown to the user. Runs on the client; also used by
    /// [`Display`].
    pub fn user_message(&self) -> String {
        use ErrorCode as E;
        match &self.code {
            E::CollectionNotFound => messages::COLLECTION_NOT_FOUND.into(),
            E::SectionNotFound => messages::SECTION_NOT_FOUND.into(),
            E::FileNotFound => messages::FILE_NOT_FOUND.into(),
            E::JobNotFound => messages::JOB_NOT_FOUND.into(),

            E::TitleRequired => messages::TITLE_REQUIRED.into(),
            E::NameRequired => messages::NAME_REQUIRED.into(),
            E::InvalidToken => messages::INVALID_TOKEN.into(),
            E::InvalidPath => messages::INVALID_PATH.into(),

            E::NoFilesReceived => messages::NO_FILES_RECEIVED.into(),
            E::NoImageReceived => messages::NO_IMAGE_RECEIVED.into(),
            E::SectionSlugMissing => messages::SECTION_SLUG_MISSING.into(),
            E::TargetMissing => messages::UPLOAD_TARGET_MISSING.into(),
            E::MediaKindMismatch => messages::MEDIA_KIND_MISMATCH.into(),
            E::UnsupportedFormat => messages::UNSUPPORTED_FORMAT.into(),

            E::SectionNotEmpty { collection_count } => format!(
                "{} ({} collection{})",
                messages::SECTION_NOT_EMPTY,
                collection_count,
                if *collection_count == 1 { "" } else { "s" },
            ),
            E::UniqueSectionIdFailed => messages::UNIQUE_SECTION_ID_FAILED.into(),

            E::ClassificationInProgress => messages::CLASSIFICATION_IN_PROGRESS.into(),
            E::NoFoldersSelected => messages::NO_FOLDERS_SELECTED.into(),
            E::NoFilesFound => messages::NO_FILES_FOUND.into(),
            E::NotAFolder => messages::NOT_A_FOLDER.into(),
            E::NotAFile => messages::NOT_A_FILE.into(),
            E::TooManyFolders { count, max } => {
                format!("Too many folders selected ({count}). Classify at most {max} at a time.")
            }

            E::ImageTooLarge { mb } => format!("Image size exceeds {mb} MB"),

            E::Detailed(s) => s.clone(),
            E::Internal => messages::INTERNAL_ERROR.into(),
        }
    }
}

// ─── UserFacingError bridge ───────────────────────────────────────────────

impl leptos_outcome::UserFacingError for AppError {
    fn message(&self) -> String {
        AppError::user_message(self)
    }
    fn is_fatal(&self) -> bool {
        matches!(self.disposition, Disposition::Fatal)
    }
}

// ─── Constructors ─────────────────────────────────────────────────────────

impl AppError {
    /// A catalogued public error. Logged at `WARN` on the server.
    pub fn public(code: ErrorCode) -> Self {
        LogLevel::Warn.emit(&format!("[public] {code:?}"));
        Self {
            code,
            disposition: Disposition::Fatal,
            diagnostic: None,
        }
    }

    /// A public error carrying a message that isn't in the catalogue.
    /// Use sparingly — prefer adding a code.
    pub fn detailed(msg: impl Into<String>) -> Self {
        Self::public(ErrorCode::Detailed(msg.into()))
    }

    /// Internal error. Client sees a generic message; cause is logged at
    /// `ERROR` with `context` prefixed.
    pub fn internal(context: impl Into<String>, source: impl Display) -> Self {
        let diagnostic = format!("{}: {}", context.into(), source);
        LogLevel::Error.emit(&diagnostic);
        Self {
            code: ErrorCode::Internal,
            disposition: Disposition::Transient,
            diagnostic: Some(diagnostic),
        }
    }

    /// Internal error with a specific user-facing code.
    pub fn hidden(code: ErrorCode, context: impl Into<String>, source: impl Display) -> Self {
        let diagnostic = format!("{}: {}", context.into(), source);
        LogLevel::Error.emit(&diagnostic);
        Self {
            code,
            disposition: Disposition::Fatal,
            diagnostic: Some(diagnostic),
        }
    }

    /// Alias for `public` — documents validation intent.
    pub fn validation(code: ErrorCode) -> Self {
        Self::public(code)
    }

    pub fn transient(mut self) -> Self {
        self.disposition = Disposition::Transient;
        self
    }
    pub fn fatal(mut self) -> Self {
        self.disposition = Disposition::Fatal;
        self
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.user_message())
    }
}

impl std::error::Error for AppError {}

// ─── FromServerFnError ────────────────────────────────────────────────────

impl server_fn::error::FromServerFnError for AppError {
    type Encoder = server_fn::codec::JsonEncoding;

    fn from_server_fn_error(value: server_fn::error::ServerFnErrorErr) -> Self {
        let diagnostic = format!("server_fn: {value:?}");
        LogLevel::Error.emit(&diagnostic);
        Self {
            code: ErrorCode::Internal,
            disposition: Disposition::Transient,
            diagnostic: Some(diagnostic),
        }
    }
}

// ─── Conversions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
impl From<multer::Error> for AppError {
    fn from(e: multer::Error) -> Self {
        AppError::internal("multipart", e)
    }
}

// ─── Context extension ────────────────────────────────────────────────────

pub trait ErrorContext<T> {
    fn context(self, ctx: impl Into<String>) -> Result<T, AppError>;
}

impl<T, E: Display> ErrorContext<T> for Result<T, E> {
    fn context(self, ctx: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| AppError::internal(ctx, e))
    }
}
