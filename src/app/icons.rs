use leptos::prelude::*;

fn icon(children: impl IntoView, class: &str) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" class=format!("{} fill-none stroke-current", class) viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{children}</svg> }.into_any()
}

#[component]
pub fn SearchIcon() -> impl IntoView {
    icon(
        view! { <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn MovieIcon() -> impl IntoView {
    icon(
        view! { <path d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn SeriesIcon() -> impl IntoView {
    icon(
        view! { <path d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn DownloadIcon() -> impl IntoView {
    icon(
        view! { <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn PlayIcon() -> impl IntoView {
    icon(view! { <polygon points="5,3 19,12 5,21"/> }, "h-6 w-6")
}

#[component]
pub fn PauseIcon() -> impl IntoView {
    icon(
        view! { <rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/> },
        "h-6 w-6",
    )
}

#[component]
pub fn ClockIcon() -> impl IntoView {
    icon(
        view! { <circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/> },
        "h-4 w-4",
    )
}

#[component]
pub fn UploadIcon() -> impl IntoView {
    icon(
        view! { <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/> },
        "h-6 w-6",
    )
}

#[component]
pub fn DeleteIcon() -> impl IntoView {
    icon(
        view! { <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn UpArrow() -> impl IntoView {
    icon(view! { <polyline points="18,15 12,9 6,15"/> }, "h-4 w-4")
}

#[component]
pub fn DownArrow() -> impl IntoView {
    icon(view! { <polyline points="6,9 12,15 18,9"/> }, "h-4 w-4")
}

#[component]
pub fn SortIcon() -> impl IntoView {
    icon(
        view! { <path d="M3 4h13M3 8h9m-9 4h9m5-4v12m0 0l-4-4m4 4l4-4"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn VolumeIcon() -> impl IntoView {
    icon(
        view! { <path d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn MuteIcon() -> impl IntoView {
    icon(
        view! { <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"/><path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn FullscreenIcon() -> impl IntoView {
    icon(
        view! { <path d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5"/> },
        "h-5 w-5",
    )
}

#[component]
pub fn FullscreenExitIcon() -> impl IntoView {
    icon(
        view! { <path d="M9 9V4M9 4H4M9 4l5 5M15 15V20M15 20h5M15 20l-5-5M9 15v5M9 15H4M9 15l5 5M15 9V4M15 4h5M15 4l-5 5"/> },
        "h-5 w-5",
    )
}
