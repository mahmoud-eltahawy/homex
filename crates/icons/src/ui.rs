//! Chrome / affordance icons: search, navigation, edit, upload, etc.

use leptos::prelude::*;

use crate::generic::Icon;

#[component]
pub fn SearchIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Search">
            <circle cx="11" cy="11" r="7" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
            <circle cx="8.5" cy="8.5" r="0.5" fill="currentColor" />
        </Icon>
    }
}

#[component]
pub fn XIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Cancel">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
        </Icon>
    }
}

#[component]
pub fn MenuIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6" label="Menu">
            <line x1="4" y1="6" x2="20" y2="6" />
            <line x1="4" y1="12" x2="20" y2="12" />
            <line x1="4" y1="18" x2="20" y2="18" />
        </Icon>
    }
}

#[component]
pub fn PrevPageIcon() -> impl IntoView {
    view! {
        <Icon class="w-5 h-5" label="Previous">
            <polyline points="9 6 15 12 9 18" />
        </Icon>
    }
}

#[component]
pub fn NextPageIcon() -> impl IntoView {
    view! {
        <Icon class="w-5 h-5" label="Next">
            <polyline points="15 6 9 12 15 18" />
        </Icon>
    }
}

#[component]
pub fn DownloadIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Download">
            <path d="M12 3v12" />
            <polyline points="7 10 12 15 17 10" />
            <path d="M5 21h14" />
            <path d="M7 18h10" />
        </Icon>
    }
}

#[component]
pub fn UploadIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5 text-gray-400 hover:text-white transition-colors" label="Upload">
            <path d="M4 14.9A7 7 0 0 1 7 4.1a7 7 0 0 1 12.7 2.1A5 5 0 0 1 19 16h-5" />
            <polyline points="12 12 12 20 9 17 12 12 15 17" />
        </Icon>
    }
}

#[component]
pub fn EditIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Edit">
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
        </Icon>
    }
}

#[component]
pub fn DeleteIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Delete">
            <path d="M3 6h18" />
            <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
            <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
            <line x1="10" y1="11" x2="10" y2="17" />
            <line x1="14" y1="11" x2="14" y2="17" />
        </Icon>
    }
}

#[component]
pub fn ViewAllIcon() -> impl IntoView {
    view! {
        <Icon class="w-5 h-5 text-gray-400 group-hover:text-white transition-colors" label="Show All" filled=true>
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
            <circle cx="6.5" cy="6.5" r="0.5" fill="#0c0b1a" />
            <circle cx="17.5" cy="6.5" r="0.5" fill="#0c0b1a" />
            <circle cx="6.5" cy="17.5" r="0.5" fill="#0c0b1a" />
            <circle cx="17.5" cy="17.5" r="0.5" fill="#0c0b1a" />
        </Icon>
    }
}

#[component]
pub fn RetryIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6 text-gray-400 hover:text-white transition-colors" label="Retry">
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1.03 6.36 2.36L21 8" />
            <polyline points="21 3 21 8 16 8" />
        </Icon>
    }
}

#[component]
pub fn EmptyStateIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16 text-white/20" viewBox="0 0 24 24" fill="currentColor" role="img" aria-label="Empty folder">
            <path d="M4 6h5l1 2h10v10H4z" />
            <path d="M8 12h8" stroke="#0c0b1a" stroke-width="2" stroke-linecap="round" fill="none" />
            <circle cx="16" cy="12" r="0.5" fill="#0c0b1a" />
        </svg>
    }
}

#[component]
pub fn ClockIcon() -> impl IntoView {
    view! {
        <Icon class="h-4 w-4" label="Clock">
            <circle cx="12" cy="12" r="9" />
            <polyline points="12 7 12 12 15 15" />
            <circle cx="12" cy="12" r="1" fill="currentColor" />
        </Icon>
    }
}
