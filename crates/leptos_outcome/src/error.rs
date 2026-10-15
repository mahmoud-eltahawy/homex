//! The [`UserFacingError`] trait, the [`SimpleError`] convenience type, and
//! the [`ErrorView`] component.

use leptos::{either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::icons::ErrorIcon;

// ─── Trait ────────────────────────────────────────────────────────────────────

/// The error-side contract that drives [`ErrorView`] and
/// [`ResourceView`](crate::ResourceView).
///
/// Implement this on whatever error type your server functions return. The
/// trait has two methods on purpose:
///
/// - [`message`](Self::message) is the text the user reads. It should be safe
///   to display verbatim — no diagnostics, no stack traces.
/// - [`is_fatal`](Self::is_fatal) tells the UI whether retrying the same
///   operation could succeed.
///
/// The type must be `Clone` (component props are signals, which clone
/// frequently), `Send + Sync` (Leptos resources cross threads under SSR), and
/// `'static`.
pub trait UserFacingError: Clone + Send + Sync + 'static {
    /// Text shown to the user. Should be safe to display verbatim.
    fn message(&self) -> String;

    /// Whether retrying the same operation could succeed.
    ///
    /// When this returns `false`, [`ErrorView`] offers a retry affordance if
    /// the caller provided one. When it returns `true`, no retry is shown —
    /// the click would produce the same result.
    fn is_fatal(&self) -> bool;
}

// ─── SimpleError ──────────────────────────────────────────────────────────────

/// A minimal [`UserFacingError`] for hosts that don't have a dedicated error
/// type yet.
///
/// Use this when you're prototyping, or in tests, or anywhere a one-off error
/// needs to flow through the same pipeline as your real one.
///
/// Construct with [`SimpleError::new_transient`] or
/// [`SimpleError::new_fatal`]. Read the message with
/// [`message()`](UserFacingError::message) and the disposition with
/// [`is_fatal()`](UserFacingError::is_fatal).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleError {
    /// The user-facing message.
    pub message: String,
    /// Whether retrying the same operation could succeed.
    fatal: bool,
}

impl SimpleError {
    /// A retryable error with the given message.
    pub fn new_transient(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fatal: false,
        }
    }

    /// An error that will produce the same result on retry.
    pub fn new_fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fatal: true,
        }
    }
}

impl UserFacingError for SimpleError {
    fn message(&self) -> String {
        self.message.clone()
    }

    fn is_fatal(&self) -> bool {
        self.fatal
    }
}

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SimpleError {}

// ─── Component ────────────────────────────────────────────────────────────────

/// Render a [`UserFacingError`] consistently.
///
/// Renders nothing when `error` is `None`. When `Some(e)`, reads
/// `e.message()` and decides whether to show a retry control based on
/// `e.is_fatal()`:
///
/// * `is_fatal() == false` — retry offered, if `on_retry` is provided.
/// * `is_fatal() == true`  — no retry. The click would produce the same result.
///
/// # Example
///
/// ```rust,ignore
/// <ErrorView
///     error=action_error(submit)
///     on_retry=Callback::new(move |_| submit.dispatch(()))
///     banner=true
///     heading="Upload failed"
/// />
/// ```
#[component]
pub fn ErrorView<E>(
    /// The error to render, or `None` to render nothing.
    error: Signal<Option<E>>,
    /// Called when the user clicks "Retry". Only shown for non-fatal errors.
    #[prop(optional, into)]
    on_retry: Option<Callback<()>>,
    /// `true` for a padded banner card, `false` for a single inline line.
    #[prop(default = false)]
    banner: bool,
    /// Override the default banner heading. Ignored in inline layout.
    #[prop(optional, into)]
    heading: Option<&'static str>,
) -> impl IntoView
where
    E: UserFacingError + 'static,
{
    move || {
        let e = error.get()?;
        let retry = if !e.is_fatal() { on_retry } else { None };

        Some(match banner {
            false => Either::Right(inline_view(e.message(), retry)),
            true => Either::Left(banner_view(e.message(), retry, heading)),
        })
    }
}

// ─── Renderers ────────────────────────────────────────────────────────────────

fn inline_view(msg: String, retry: Option<Callback<()>>) -> impl IntoView {
    view! {
        <div class="text-red-300 text-sm flex items-center gap-2">
            <span>{msg}</span>
            {retry.map(|cb| view! {
                <button
                    type="button"
                    on:click=move |_| cb.run(())
                    class="underline hover:text-red-200 transition-colors text-xs shrink-0"
                >
                    "retry"
                </button>
            })}
        </div>
    }
}

fn banner_view(msg: String, retry: Option<Callback<()>>, heading: Option<&str>) -> impl IntoView {
    let heading = heading.unwrap_or("Something went wrong").to_string();

    view! {
        <div class="bg-red-500/15 border border-red-500/30 rounded-xl p-4 space-y-2">
            <div class="flex items-start gap-3">
                <span class="text-red-400 shrink-0 mt-0.5"><ErrorIcon/></span>
                <div class="flex-1 min-w-0">
                    <div class="text-red-300 text-sm font-bold">{heading}</div>
                    <div class="text-red-300/80 text-xs break-all mt-1">{msg}</div>
                </div>
            </div>
            {retry.map(|cb| view! {
                <div class="flex justify-end">
                    <button
                        type="button"
                        on:click=move |_| cb.run(())
                        class="text-xs px-3 py-1 rounded-lg bg-red-500/20 \
                               hover:bg-red-500/30 text-red-200 transition"
                    >
                        "Retry"
                    </button>
                </div>
            })}
        </div>
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_errors_offer_retry() {
        let e = SimpleError::new_transient("retry me");
        assert_eq!(e.message(), "retry me");
        assert!(!e.is_fatal());
    }

    #[test]
    fn fatal_errors_suppress_retry() {
        let e = SimpleError::new_fatal("nope");
        assert_eq!(e.message(), "nope");
        assert!(e.is_fatal());
    }

    #[test]
    fn displays_as_message() {
        let e = SimpleError::new_fatal("bad input");
        assert_eq!(e.to_string(), "bad input");
    }

    #[test]
    fn serde_roundtrip_preserves_message_and_disposition() {
        let e = SimpleError::new_transient("try again");
        let s = serde_json::to_string(&e).unwrap();
        let back: SimpleError = serde_json::from_str(&s).unwrap();
        assert_eq!(back.message(), "try again");
        assert!(!back.is_fatal());
    }
}
