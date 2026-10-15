//! Outcome-oriented views for Leptos.
//!
//! Two components and one trait:
//!
//! - [`ResourceView`] — render a [`Resource<Result<T, E>>`](leptos::prelude::Resource)
//!   as loading, success, or error, in one place.
//! - [`ErrorView`] — render an error the same way everywhere, in inline or
//!   banner layout.
//! - [`UserFacingError`] — the trait that drives both, decoupling the error
//!   type from the UI.
//!
//! # Getting started
//!
//! ```rust
//! use leptos_outcome::UserFacingError;
//!
//! #[derive(Clone)]
//! struct AppError {
//!     message: String,
//!     is_fatal: bool,
//! }
//!
//! impl UserFacingError for AppError {
//!     fn message(&self) -> String {
//!         self.message.clone()
//!     }
//!
//!     fn is_fatal(&self) -> bool {
//!         self.is_fatal
//!     }
//! }
//! ```
//!
//! # Design
//!
//! The crate has no opinion about what an error *is*. It only asks that the
//! error type be cheap to clone (component props are signals), `Send + Sync`
//! (Leptos resources cross threads under SSR), and able to answer two
//! questions:
//!
//! 1. What should the user read?
//! 2. Is retrying worth offering?
//!
//! Everything else — logging, diagnostics, HTTP status, tracing — stays in
//! your app, where it belongs.
#![forbid(unsafe_code)]

pub mod error;
pub mod extract;
pub mod icons;
pub mod resource_view;

// ─── Root re-exports ──────────────────────────────────────────────────────────

pub use error::{ErrorView, SimpleError, UserFacingError};
pub use extract::{action_error, resource_error, result_signal_error};
pub use resource_view::ResourceView;
