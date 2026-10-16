//! Media-player controls: transport, volume, fullscreen.
//!
//! Sized with Tailwind utilities (`size-5`, `size-6`). The media player's
//! buttons rely on these sizes, so the icons declare them rather than the
//! caller.

use leptos::prelude::*;

use crate::generic::Icon;

// ─── Playback ─────────────────────────────────────────────────────────────

#[component]
pub fn PlayIcon() -> impl IntoView {
    view! {
        <Icon class="size-6 shrink-0" label="Play">
            <polygon points="5 3 19 12 5 21 5 3" fill="currentColor" stroke="none" />
            <polygon points="7 6 17 12 7 18" fill="none" stroke="rgba(255,255,255,0.3)" />
        </Icon>
    }
}

#[component]
pub fn PauseIcon() -> impl IntoView {
    view! {
        <Icon class="size-6 shrink-0" label="Pause">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
            <line x1="6" y1="12" x2="10" y2="12" stroke="currentColor" opacity="0.5" />
            <line x1="14" y1="12" x2="18" y2="12" stroke="currentColor" opacity="0.5" />
        </Icon>
    }
}

// ─── Audio ────────────────────────────────────────────────────────────────

#[component]
pub fn VolumeIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Volume">
            <path d="M11 5L6 9H2v6h4l5 4V5z" />
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
        </Icon>
    }
}

#[component]
pub fn MuteIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Mute">
            <path d="M11 5L6 9H2v6h4l5 4V5z" />
            <line x1="23" y1="9" x2="17" y2="15" />
            <line x1="17" y1="9" x2="23" y2="15" />
        </Icon>
    }
}

// ─── Fullscreen ───────────────────────────────────────────────────────────

#[component]
pub fn FullscreenIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Full Screen">
            <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M12 8v8M8 12h8" opacity="0.5" />
        </Icon>
    }
}

#[component]
pub fn FullscreenExitIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Full Screen Exit">
            <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M12 8v8M8 12h8" opacity="0.5" />
        </Icon>
    }
}

// ─── Playlist navigation ──────────────────────────────────────────────────

#[component]
pub fn PrevPageIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Previous">
            <polyline points="9 6 15 12 9 18" />
        </Icon>
    }
}

#[component]
pub fn NextPageIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Next">
            <polyline points="15 6 9 12 15 18" />
        </Icon>
    }
}

// ─── Playlist actions ─────────────────────────────────────────────────────

#[component]
pub fn DownloadIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Download">
            <path d="M12 3v12" />
            <polyline points="7 10 12 15 17 10" />
            <path d="M5 21h14" />
            <path d="M7 18h10" />
        </Icon>
    }
}

#[component]
pub fn EditIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Edit">
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
        </Icon>
    }
}

#[component]
pub fn DeleteIcon() -> impl IntoView {
    view! {
        <Icon class="size-5 shrink-0" label="Delete">
            <path d="M3 6h18" />
            <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
            <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
            <line x1="10" y1="11" x2="10" y2="17" />
            <line x1="14" y1="11" x2="14" y2="17" />
        </Icon>
    }
}
