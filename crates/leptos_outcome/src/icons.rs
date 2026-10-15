//! The two icons the crate renders itself.
//!
//! They are public so hosts that want visual consistency can reuse them, but
//! the crate treats them as an implementation detail: a future version may
//! swap them out or add a customization hook.

use leptos::prelude::*;

#[component]
fn Icon(
    children: Children,
    #[prop(into)] class: String,
    #[prop(into)] label: String,
    #[prop(default = false)] filled: bool,
    #[prop(default = "1.5")] stroke_width: &'static str,
) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class=class
            viewBox="0 0 24 24"
            fill=if filled { "currentColor" } else { "none" }
            stroke=if !filled { "currentColor" } else { "none" }
            stroke-width=stroke_width
            stroke-linecap="round"
            stroke-linejoin="round"
            role="img"
            aria-label=label.clone()
        >
            {children()}
        </svg>
    }
}

/// A red circle-with-exclamation icon. Used by
/// [`ErrorView`](crate::ErrorView)'s banner layout.
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

/// A spinning cyan arc. Used by [`ResourceView`](crate::ResourceView) as the
/// default loading fallback.
#[component]
pub fn LoadingIcon() -> impl IntoView {
    view! {
        <Icon class="h-8 w-8 animate-spin text-cyan-400" label="Loading">
            <circle cx="12" cy="12" r="10" stroke-dasharray="70 200" stroke-dashoffset="0" />
            <circle cx="12" cy="12" r="2" fill="currentColor" />
        </Icon>
    }
}
