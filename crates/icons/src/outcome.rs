//! Feedback glyphs for error and loading states.
//!
//! Extracted from `leptos_outcome`. Both components are used by that
//! crate's `ErrorView` / `ResourceView`, but they are generic enough
//! (a warning circle and a spinner) to reuse anywhere a status needs a
//! visual anchor.

use leptos::prelude::*;

use crate::generic::Icon;

/// A red circle-with-exclamation icon. Used by the banner layout of
/// `leptos_outcome::ErrorView`.
#[component]
pub fn ErrorIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6 text-red-400" label="Something Went Wrong">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <circle cx="12" cy="16" r="0.5" fill="currentColor" />
        </Icon>
    }
}

/// A spinning cyan arc. Used as the default loading fallback of
/// `leptos_outcome::ResourceView`.
#[component]
pub fn LoadingIcon() -> impl IntoView {
    view! {
        <Icon class="h-8 w-8 animate-spin text-cyan-400" label="Loading">
            <circle cx="12" cy="12" r="10" stroke-dasharray="70 200" stroke-dashoffset="0" />
            <circle cx="12" cy="12" r="2" fill="currentColor" />
        </Icon>
    }
}
