//! The four media-kind glyphs, plus the enum that selects among them.
//!
//! The crate deliberately doesn't know about HomeX's `MediaKind`: the
//! caller maps its own domain type onto [`MediaIconKind`] and hands that in.

use leptos::prelude::*;

use crate::generic::Icon;

#[component]
pub fn MovieIcon() -> impl IntoView {
    view! {
        <Icon class="w-6 h-6 text-amber-400" label="Movie" filled=true>
            <path d="M4 5h16a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z" />
            <path d="M3 10h18" stroke="#0c0b1a" stroke-width="2" />
            <path d="M6 5v5M10 5v5M14 5v5M18 5v5" stroke="#0c0b1a" stroke-width="2" />
            <path d="M8 10l4-5M12 10l4-5" stroke="#0c0b1a" stroke-width="2" fill="none" />
        </Icon>
    }
}

#[component]
pub fn SeriesIcon() -> impl IntoView {
    view! {
        <Icon class="w-6 h-6 text-purple-400" label="Series" filled=true>
            <rect x="3" y="4" width="18" height="13" rx="2" />
            <path d="M8 20h8l-1-3H9z" fill="currentColor" />
            <path d="M10 8h4v3h-4z" fill="#0c0b1a" />
            <path d="M6 4l3 3M18 4l-3 3" stroke="#0c0b1a" stroke-width="2" fill="none" />
        </Icon>
    }
}

#[component]
pub fn AudioIcon() -> impl IntoView {
    view! {
        <Icon class="w-6 h-6 text-cyan-400" label="Audio" filled=true>
            <path d="M4 13v-1a8 8 0 1 1 16 0v1" />
            <path d="M4 13a2 2 0 0 0-2 2v1a2 2 0 0 0 2 2h2v-5H4zM20 13a2 2 0 0 1 2 2v1a2 2 0 0 1-2 2h-2v-5h2z" />
            <rect x="6" y="14" width="12" height="4" rx="1" fill="#0c0b1a" />
        </Icon>
    }
}

#[component]
pub fn AudioGroupIcon() -> impl IntoView {
    view! {
        <Icon class="w-6 h-6 text-pink-400" label="Audio Group" filled=true>
            <rect x="5" y="4" width="14" height="14" rx="2" opacity="0.45" />
            <rect x="3" y="6" width="14" height="14" rx="2" />
            <circle cx="10" cy="13" r="4" fill="#0c0b1a" />
            <circle cx="10" cy="13" r="1.5" fill="currentColor" />
        </Icon>
    }
}

/// The four media-kind glyphs this crate knows how to draw.
///
/// HomeX's `MediaKind` collapses into this via `(kind, nested)`:
///
/// | `MediaKind` | `nested` | `MediaIconKind` |
/// |-------------|----------|-----------------|
/// | `Video`     | `false`  | `Movie`         |
/// | `Video`     | `true`   | `Series`        |
/// | `Audio`     | `false`  | `Audio`         |
/// | `Audio`     | `true`   | `AudioGroup`    |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaIconKind {
    Movie,
    Series,
    Audio,
    AudioGroup,
}

impl MediaIconKind {
    /// Convenience for `(is_video, is_nested)` call sites.
    pub fn from_parts(is_video: bool, is_nested: bool) -> Self {
        match (is_video, is_nested) {
            (true, false) => Self::Movie,
            (true, true) => Self::Series,
            (false, false) => Self::Audio,
            (false, true) => Self::AudioGroup,
        }
    }
}

/// Render the glyph for `kind`.
///
/// Mirrors the original `crate::app::icons::icon_for` signature, but takes
/// the crate-local enum instead of the app's `MediaKind`.
pub fn icon_for(kind: MediaIconKind) -> AnyView {
    match kind {
        MediaIconKind::Movie => view! { <MovieIcon/> }.into_any(),
        MediaIconKind::Series => view! { <SeriesIcon/> }.into_any(),
        MediaIconKind::Audio => view! { <AudioIcon/> }.into_any(),
        MediaIconKind::AudioGroup => view! { <AudioGroupIcon/> }.into_any(),
    }
}
