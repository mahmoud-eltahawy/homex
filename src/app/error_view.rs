//! A single component that renders an [`AppError`] consistently across the
//! app.
//!
//! [`ErrorView`] is the error-handling counterpart of `ResourceView`. Where
//! `ResourceView` decides how to render loading/success/error for a
//! `Resource`, `ErrorView` decides how to render an error for whatever
//! produced it — a `Resource`, an `Action`, or a hand-rolled
//! `Signal<Option<AppError>>`.

use leptos::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::app::error::{AppError, Disposition};
use crate::app::icons::ErrorIcon;

// ─── Signal extraction helpers ────────────────────────────────────────────

/// Pull the error out of an `Action<_, Result<V, AppError>>`.
pub fn action_error<A, V>(action: Action<A, Result<V, AppError>>) -> Signal<Option<AppError>>
where
    A: Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    Signal::derive(move || action.value().get().and_then(|r| r.err()))
}

/// Pull the error out of a `Resource<Result<T, AppError>>`.
pub fn resource_error<T>(resource: Resource<Result<T, AppError>>) -> Signal<Option<AppError>>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    Signal::derive(move || resource.get().and_then(|r| r.err()))
}

/// Pull the error out of a `Signal<Option<Result<T, AppError>>>`.
pub fn result_signal_error<T>(
    result: Signal<Option<Result<T, AppError>>>,
) -> Signal<Option<AppError>>
where
    T: Send + Sync + Clone + 'static,
{
    Signal::derive(move || result.get().and_then(|r| r.err()))
}

// ─── Style ────────────────────────────────────────────────────────────────

/// Visual treatment of an [`ErrorView`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ErrorStyle {
    /// Small inline red text.
    #[default]
    Inline,
    /// Padded red-bordered box with an icon and a heading.
    Banner,
}

// ─── Component ────────────────────────────────────────────────────────────

/// Render an [`AppError`] consistently.
///
/// Renders nothing when `error` is `None`. When `Some(e)`, reads
/// `e.message` and decides whether to offer a retry button based on
/// `e.disposition`:
///
///   * `Transient` → retry offered (if `on_retry` is provided)
///   * `Fatal`     → no retry — clicking it would produce the same result
#[component]
pub fn ErrorView(
    error: Signal<Option<AppError>>,
    #[prop(optional, into)] on_retry: Option<Callback<()>>,
    #[prop(default = ErrorStyle::Inline)] style: ErrorStyle,
    /// Override the default `Banner` heading. Ignored for `Inline`.
    #[prop(optional, into)]
    heading: Option<String>,
) -> impl IntoView {
    let render = move || -> Option<AnyView> {
        let e = error.get()?;
        let retry = if matches!(e.disposition, Disposition::Transient) {
            on_retry
        } else {
            None
        };

        Some(match style {
            ErrorStyle::Inline => inline_view(e.message, retry),
            ErrorStyle::Banner => banner_view(e.message, retry, heading.as_deref()),
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
