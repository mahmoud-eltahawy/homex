//! A single component that renders a `ServerFnError` consistently across
//! the app.
//!
//! [`ErrorView`] is the error-handling counterpart of `ResourceView`. Where
//! `ResourceView` decides how to render loading/success/error for a
//! `Resource`, `ErrorView` decides how to render an error for whatever
//! produced it — a `Resource`, an `Action`, or a hand-rolled
//! `Signal<Option<ServerFnError>>`.
//!
//! The two signal-extraction helpers ([`action_error`], [`resource_error`])
//! exist because nearly every call site pulls the error out of one of those
//! two containers. The extraction itself is trivial but repetitive, and
//! centralizing it is what keeps the display logic in exactly one place.

use leptos::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::app::error::{self, Disposition};
use crate::app::icons::ErrorIcon;

// ─── Signal extraction helpers ────────────────────────────────────────────

/// Pull the error out of an `Action<_, Result<V, ServerFnError>>`.
///
/// Returns `None` while the action is idle or the last run succeeded, and
/// `Some(e)` if it failed. Hand the result to [`ErrorView`].
///
/// ```ignore
/// let save = Action::new_local(|v: &String| save_thing(v.clone()));
/// let err  = action_error(save);
/// ```
pub fn action_error<A, V>(
    action: Action<A, Result<V, ServerFnError>>,
) -> Signal<Option<ServerFnError>>
where
    A: Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    Signal::derive(move || action.value().get().and_then(|r| r.err()))
}

/// Pull the error out of a `Resource<Result<T, ServerFnError>>`.
///
/// Returns `None` while the resource is pending or the last load succeeded.
pub fn resource_error<T>(
    resource: Resource<Result<T, ServerFnError>>,
) -> Signal<Option<ServerFnError>>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    Signal::derive(move || resource.get().and_then(|r| r.err()))
}

/// Pull the error out of a `Signal<Option<Result<T, ServerFnError>>>`.
///
/// Useful when you're already working with a derived signal rather than a
/// raw `Action` or `Resource`.
pub fn result_signal_error<T>(
    result: Signal<Option<Result<T, ServerFnError>>>,
) -> Signal<Option<ServerFnError>>
where
    T: Send + Sync + Clone + 'static,
{
    Signal::derive(move || result.get().and_then(|r| r.err()))
}

// ─── Style ────────────────────────────────────────────────────────────────

/// Visual treatment of an [`ErrorView`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ErrorStyle {
    /// Small inline red text. For form fields, in-row validation, and any
    /// context where a full box would be too heavy. No icon, no heading.
    #[default]
    Inline,
    /// Padded red-bordered box with an icon and a heading. For
    /// section-level errors, upload failures, and page-level error states.
    Banner,
}

// ─── The component ────────────────────────────────────────────────────────

/// Render a `ServerFnError` consistently.
///
/// Renders nothing when `error` is `None`. When `Some(e)`, extracts the
/// user-facing message (stripping the wire-format sentinel) and decides
/// whether to offer a retry button based on the error's [`Disposition`]:
///
///   * `Transient` → retry offered (if `on_retry` is provided)
///   * `Fatal`     → no retry — clicking it would produce the same result
///
/// `on_retry` is only rendered for transient errors, so it's safe to pass
/// unconditionally at every call site.
///
/// # Examples
///
/// ```ignore
/// // Inline, from an Action:
/// let save = Action::new_local(|v: &String| save_thing(v.clone()));
/// view! { <ErrorView error=action_error(save)/> }
///
/// // Banner, from a Resource, with retry:
/// let posts = Resource::new(|| (), |_| fetch_posts());
/// view! {
///     <ErrorView
///         error=resource_error(posts)
///         on_retry=Callback::new(move |_| posts.refetch())
///         style=ErrorStyle::Banner
///     />
/// }
/// ```
#[component]
pub fn ErrorView(
    error: Signal<Option<ServerFnError>>,
    #[prop(optional, into)] on_retry: Option<Callback<()>>,
    #[prop(default = ErrorStyle::Inline)] style: ErrorStyle,
    /// Override the default `Banner` heading. Ignored for `Inline`.
    #[prop(optional, into)]
    heading: Option<String>,
) -> impl IntoView {
    let render = move || -> Option<AnyView> {
        let e = error.get()?;
        let e = e.to_string();
        let (disposition, msg) = error::parse_wire_error(&e);
        let msg = msg.to_string();
        let retry = if matches!(disposition, Disposition::Transient) {
            on_retry
        } else {
            None
        };

        Some(match style {
            ErrorStyle::Inline => inline_view(msg, retry),
            ErrorStyle::Banner => banner_view(msg, retry, heading.as_deref()),
        })
    };

    view! { {render} }
}

// ─── Renderers ────────────────────────────────────────────────────────────

fn inline_view(msg: String, retry: Option<Callback<()>>) -> AnyView {
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
    .into_any()
}

fn banner_view(msg: String, retry: Option<Callback<()>>, heading: Option<&str>) -> AnyView {
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
    .into_any()
}
