use leptos::prelude::*;

fn tooltip(children: impl IntoView, text: &'static str) -> impl IntoView {
    view! {
        <div class="relative group inline-flex">
            {children}
            <span class="
                absolute top-full left-1/2 -translate-x-1/2 mt-2
                px-2 py-1 rounded-md bg-gray-900 text-white text-xs font-medium
                opacity-0 group-hover:opacity-100 transition-opacity duration-200
                pointer-events-none whitespace-nowrap
                border border-white/10 shadow-lg
                z-50
            ">
                {text}
            </span>
        </div>
    }
}

fn stroked_icon(children: impl IntoView, class: &'static str, text: &'static str) -> impl IntoView {
    tooltip(
        view! {
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class=format!("{} fill-none stroke-current", class)
                viewBox="0 0 24 24"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                {children}
            </svg>
        },
        text,
    )
}

fn filled_icon(children: impl IntoView, class: &'static str, text: &'static str) -> impl IntoView {
    tooltip(
        view! {
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class=class
                viewBox="0 0 24 24"
                fill="currentColor"
            >
                {children}
            </svg>
        },
        text,
    )
}

// ─── Optional: Keep your existing SearchIcon as an example ────────────────
#[component]
pub fn SearchIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/> },
        "h-5 w-5",
        "Search",
    )
}

#[component]
pub fn DownloadIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/> },
        "h-5 w-5",
        "Download",
    )
}

#[component]
pub fn PlayIcon() -> impl IntoView {
    stroked_icon(
        view! { <polygon points="5,3 19,12 5,21"/> },
        "h-6 w-6",
        "Play",
    )
}

#[component]
pub fn PauseIcon() -> impl IntoView {
    stroked_icon(
        view! { <rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/> },
        "h-6 w-6",
        "Pause",
    )
}

#[component]
pub fn ClockIcon() -> impl IntoView {
    stroked_icon(
        view! { <circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/> },
        "h-4 w-4",
        "Clock",
    )
}

#[component]
pub fn DeleteIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/> },
        "h-5 w-5",
        "Delete",
    )
}

#[component]
pub fn UpArrow() -> impl IntoView {
    stroked_icon(
        view! { <polyline points="18,15 12,9 6,15"/> },
        "h-4 w-4",
        "Go Up",
    )
}

#[component]
pub fn DownArrow() -> impl IntoView {
    stroked_icon(
        view! { <polyline points="6,9 12,15 18,9"/> },
        "h-4 w-4",
        "Go Down",
    )
}

#[component]
pub fn SortIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M3 4h13M3 8h9m-9 4h9m5-4v12m0 0l-4-4m4 4l4-4"/> },
        "h-5 w-5",
        "Sort Items",
    )
}

#[component]
pub fn VolumeIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/> },
        "h-5 w-5",
        "Volume",
    )
}

#[component]
pub fn MuteIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"/><path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/> },
        "h-5 w-5",
        "Mute",
    )
}

#[component]
pub fn FullscreenIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5"/> },
        "h-5 w-5",
        "Full Screen",
    )
}

#[component]
pub fn FullscreenExitIcon() -> impl IntoView {
    stroked_icon(
        view! { <path d="M9 9V4M9 4H4M9 4l5 5M15 15V20M15 20h5M15 20l-5-5M9 15v5M9 15H4M9 15l5 5M15 9V4M15 4h5M15 4l-5 5"/> },
        "h-5 w-5",
        "Full Screen Exit",
    )
}

#[component]
pub fn MoviePosterSvg() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 900" width="100%" height="100%">
          <defs>
            <linearGradient id="bg-movie" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#0f172a" />
              <stop offset="100%" stop-color="#1e293b" />
            </linearGradient>
            <radialGradient id="spotlight" cx="50%" cy="35%" r="50%">
              <stop offset="0%" stop-color="#38bdf8" stop-opacity="0.4" />
              <stop offset="100%" stop-color="#38bdf8" stop-opacity="0" />
            </radialGradient>
            <linearGradient id="film" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stop-color="#64748b" />
              <stop offset="50%" stop-color="#94a3b8" />
              <stop offset="100%" stop-color="#64748b" />
            </linearGradient>
          </defs>

          <rect width="600" height="900" fill="url(#bg-movie)" />
          <rect width="600" height="900" fill="url(#spotlight)" />

          <g transform="translate(-50, 650) rotate(-15)">
            <rect x="0" y="0" width="800" height="60" fill="url(#film)" opacity="0.8" />
            <rect x="20" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="80" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="140" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="200" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="260" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="320" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="380" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="440" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="500" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="560" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="620" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="680" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="740" y="10" width="20" height="20" fill="#0f172a" />
            <rect x="0" y="45" width="800" height="10" fill="#0f172a" opacity="0.3" />
          </g>

          <g transform="translate(300, 320)">
            <circle r="100" fill="none" stroke="#e2e8f0" stroke-width="6" />
            <circle r="20" fill="#e2e8f0" />
            <circle cx="0" cy="-70" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="70" cy="0" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="0" cy="70" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="-70" cy="0" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="-50" cy="-50" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="50" cy="-50" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="50" cy="50" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
            <circle cx="-50" cy="50" r="25" fill="none" stroke="#e2e8f0" stroke-width="4" />
          </g>

          <text x="300" y="600" font-family="Arial, Helvetica, sans-serif" font-size="42" font-weight="900" letter-spacing="8" fill="#ffffff" text-anchor="middle">MOVIE</text>
          <text x="300" y="640" font-family="Arial, Helvetica, sans-serif" font-size="16" letter-spacing="4" fill="#94a3b8" text-anchor="middle">NO POSTER AVAILABLE</text>
        </svg>
    }
}

#[component]
pub fn SeriesPosterSvg() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 900" width="100%" height="100%">
          <defs>
            <linearGradient id="bg-series" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#111827" />
              <stop offset="100%" stop-color="#312e81" />
            </linearGradient>
            <radialGradient id="glow" cx="50%" cy="40%" r="40%">
              <stop offset="0%" stop-color="#8b5cf6" stop-opacity="0.4" />
              <stop offset="100%" stop-color="#8b5cf6" stop-opacity="0" />
            </radialGradient>
            <linearGradient id="screen" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" stop-color="#e0e7ff" />
              <stop offset="100%" stop-color="#a5b4fc" />
            </linearGradient>
          </defs>

          <rect width="600" height="900" fill="url(#bg-series)" />
          <rect width="600" height="900" fill="url(#glow)" />

          <g transform="translate(150, 250) skewY(5)">
            <rect width="300" height="180" rx="10" fill="#1e1b4b" stroke="#6366f1" stroke-width="4" opacity="0.6" />
          </g>
          <g transform="translate(180, 280) skewY(-5)">
            <rect width="300" height="180" rx="10" fill="#312e81" stroke="#818cf8" stroke-width="5" opacity="0.8" />
          </g>
          <g transform="translate(210, 310)">
            <rect width="300" height="180" rx="12" fill="url(#screen)" />
            <polygon points="350,370 350,430 410,400" fill="#1e1b4b" />
            <rect x="210" y="470" width="300" height="4" fill="#6366f1" opacity="0.5" />
            <rect x="220" y="480" width="80" height="10" rx="2" fill="#a5b4fc" opacity="0.8" />
            <rect x="310" y="480" width="80" height="10" rx="2" fill="#a5b4fc" opacity="0.5" />
            <rect x="400" y="480" width="80" height="10" rx="2" fill="#a5b4fc" opacity="0.3" />
          </g>

          <path d="M 120 180 Q 150 150 180 180" fill="none" stroke="#c4b5fd" stroke-width="6" stroke-linecap="round" />
          <path d="M 110 160 Q 150 120 190 160" fill="none" stroke="#c4b5fd" stroke-width="6" stroke-linecap="round" opacity="0.6" />

          <text x="300" y="600" font-family="Arial, Helvetica, sans-serif" font-size="42" font-weight="900" letter-spacing="8" fill="#ffffff" text-anchor="middle">SERIES</text>
          <text x="300" y="640" font-family="Arial, Helvetica, sans-serif" font-size="16" letter-spacing="4" fill="#a5b4fc" text-anchor="middle">NO POSTER AVAILABLE</text>
        </svg>
    }
}

#[component]
pub fn MusicPosterSvg() -> impl IntoView {
    view! {

        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 600" width="100%" height="100%">
          <defs>
            <linearGradient id="bg-music" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#4c1d95" />
              <stop offset="50%" stop-color="#a21caf" />
              <stop offset="100%" stop-color="#be185d" />
            </linearGradient>
            <radialGradient id="vinyl" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stop-color="#fce7f3" />
              <stop offset="20%" stop-color="#fbcfe8" />
              <stop offset="40%" stop-color="#f9a8d4" />
              <stop offset="100%" stop-color="#1e1b4b" />
            </radialGradient>
            <linearGradient id="wave" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stop-color="#fbcfe8" />
              <stop offset="100%" stop-color="#e9d5ff" />
            </linearGradient>
          </defs>

          <rect width="600" height="600" fill="url(#bg-music)" />

          <circle cx="300" cy="250" r="180" fill="#ffffff" opacity="0.05" />
          <circle cx="300" cy="250" r="140" fill="#ffffff" opacity="0.08" />

          <circle cx="300" cy="250" r="120" fill="url(#vinyl)" />
          <circle cx="300" cy="250" r="110" fill="none" stroke="#1e1b4b" stroke-width="1" opacity="0.3" />
          <circle cx="300" cy="250" r="90" fill="none" stroke="#1e1b4b" stroke-width="1" opacity="0.3" />
          <circle cx="300" cy="250" r="70" fill="none" stroke="#1e1b4b" stroke-width="1" opacity="0.3" />
          <circle cx="300" cy="250" r="30" fill="#1e1b4b" />
          <circle cx="300" cy="250" r="12" fill="#fce7f3" />

          <g transform="translate(420, 180)">
            <rect x="0" y="0" width="8" height="120" rx="4" fill="#e2e8f0" transform="rotate(25)" />
            <circle cx="0" cy="0" r="15" fill="#64748b" />
            <circle cx="0" cy="0" r="8" fill="#94a3b8" />
            <circle cx="30" cy="120" r="5" fill="#cbd5e1" />
          </g>

          <g transform="translate(0, 430)">
            <rect x="50" y="40" width="12" height="30" rx="2" fill="url(#wave)" />
            <rect x="75" y="20" width="12" height="70" rx="2" fill="url(#wave)" />
            <rect x="100" y="10" width="12" height="90" rx="2" fill="url(#wave)" />
            <rect x="125" y="30" width="12" height="50" rx="2" fill="url(#wave)" />
            <rect x="150" y="0" width="12" height="110" rx="2" fill="url(#wave)" />
            <rect x="175" y="40" width="12" height="30" rx="2" fill="url(#wave)" />

            <rect x="200" y="20" width="12" height="70" rx="2" fill="url(#wave)" />
            <rect x="225" y="10" width="12" height="90" rx="2" fill="url(#wave)" />
            <rect x="250" y="30" width="12" height="50" rx="2" fill="url(#wave)" />
            <rect x="275" y="5" width="12" height="100" rx="2" fill="url(#wave)" />
            <rect x="300" y="40" width="12" height="30" rx="2" fill="url(#wave)" />

            <rect x="325" y="20" width="12" height="70" rx="2" fill="url(#wave)" />
            <rect x="350" y="10" width="12" height="90" rx="2" fill="url(#wave)" />
            <rect x="375" y="35" width="12" height="40" rx="2" fill="url(#wave)" />
            <rect x="400" y="0" width="12" height="110" rx="2" fill="url(#wave)" />
            <rect x="425" y="25" width="12" height="60" rx="2" fill="url(#wave)" />
            <rect x="450" y="15" width="12" height="80" rx="2" fill="url(#wave)" />
            <rect x="475" y="45" width="12" height="20" rx="2" fill="url(#wave)" />
            <rect x="500" y="30" width="12" height="50" rx="2" fill="url(#wave)" />
          </g>

          <text x="300" y="550" font-family="Arial, Helvetica, sans-serif" font-size="36" font-weight="900" letter-spacing="6" fill="#ffffff" text-anchor="middle">MUSIC</text>
          <text x="300" y="580" font-family="Arial, Helvetica, sans-serif" font-size="14" letter-spacing="3" fill="#fbcfe8" text-anchor="middle">NO ARTWORK AVAILABLE</text>
        </svg>
    }
}

#[component]
pub fn XIcon() -> impl IntoView {
    stroked_icon(
        view! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /> },
        "h-5 w-5",
        "Cancel",
    )
}

#[component]
pub fn MovieIcon() -> impl IntoView {
    filled_icon(
        view! {
            <rect x="3" y="4" width="18" height="5" rx="1" />
            <rect x="3" y="9" width="18" height="11" rx="1" />
            <rect x="6" y="11" width="12" height="2" fill="#0c0b1a" />
            <rect x="6" y="14" width="12" height="2" fill="#0c0b1a" />
            <rect x="6" y="17" width="12" height="2" fill="#0c0b1a" />
            <circle cx="6" cy="4" r="1.5" fill="#0c0b1a" />
        },
        "w-6 h-6 text-amber-400",
        "Movie",
    )
}

#[component]
pub fn SeriesIcon() -> impl IntoView {
    filled_icon(
        view! {
            <rect x="4" y="4" width="16" height="11" rx="2" />
            <path d="M8 17h8l1 2H7z" />
            <path d="M10 8.5v5l5-2.5z" fill="#0c0b1a" />
        },
        "w-6 h-6 text-purple-400",
        "Series",
    )
}

#[component]
pub fn AudioIcon() -> impl IntoView {
    filled_icon(
        view! {
            <path d="M7 10v4a5 5 0 0 0 10 0v-4" />
            <rect x="4" y="9" width="5" height="8" rx="1.5" />
            <rect x="15" y="9" width="5" height="8" rx="1.5" />
            <circle cx="6.5" cy="17" r="0.8" fill="#0c0b1a" />
            <circle cx="17.5" cy="17" r="0.8" fill="#0c0b1a" />
        },
        "w-6 h-6 text-cyan-400",
        "Audio",
    )
}

#[component]
pub fn EmptyStateIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="w-16 h-16 text-white/20"
            viewBox="0 0 24 24"
            fill="currentColor"
        >
            <path d="M4 6h5l1 2h10v10H4z" />
            <path d="M8 12h8" stroke="#0c0b1a" stroke-width="2" stroke-linecap="round" fill="none" />
        </svg>
    }
}

#[component]
pub fn ViewAllIcon() -> impl IntoView {
    filled_icon(
        view! {
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
        },
        "w-5 h-5 text-gray-400 group-hover:text-white transition-colors",
        "Show All",
    )
}

#[component]
pub fn PrevPageIcon() -> impl IntoView {
    stroked_icon(
        view! { <polyline points="9 6 15 12 9 18" /> },
        "w-5 h-5",
        "Previous",
    )
}

#[component]
pub fn NextPageIcon() -> impl IntoView {
    stroked_icon(
        view! { <polyline points="15 6 9 12 15 18" /> },
        "w-5 h-5",
        "Next",
    )
}

#[component]
pub fn LoadingIcon() -> impl IntoView {
    stroked_icon(
        view! {
            <circle
                cx="12"
                cy="12"
                r="10"
                stroke-dasharray="70 200"
                stroke-dashoffset="0"
                stroke-linecap="round"
            />
        },
        "h-8 w-8 animate-spin text-cyan-400",
        "Loading",
    )
}

#[component]
pub fn RetryIcon() -> impl IntoView {
    stroked_icon(
        view! {
            <path d="M21 12a9 9 0 0 1-9 9m9-9a9 9 0 0 0-9-9m9 9h-4m4 0v4" />
        },
        "h-6 w-6 text-gray-400 hover:text-white transition-colors",
        "Retry",
    )
}

#[component]
pub fn ErrorIcon() -> impl IntoView {
    stroked_icon(
        view! {
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <circle cx="12" cy="16" r="0.5" fill="currentColor" />
        },
        "h-6 w-6 text-red-400",
        "Something Went Wrong",
    )
}

#[component]
pub fn MediaCubeLogo() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="w-8 h-8">
            <defs>
                <linearGradient id="logoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#22d3ee" /> // cyan-400
                    <stop offset="100%" stop-color="#3b82f6" /> // blue-500
                </linearGradient>
            </defs>
            <rect x="2" y="2" width="20" height="20" rx="5" fill="url(#logoGrad)" />
            <path d="M10 7 L10 17 L17 12 Z" fill="#0a0a0f" />
        </svg>
    }
}

#[component]
pub fn SettingsIcon() -> impl IntoView {
    stroked_icon(
        view! {
            <>
                <circle cx="12" cy="12" r="3" />
                <path d="M12 1v3" />
                <path d="M12 20v3" />
                <path d="M4.22 4.22l2.12 2.12" />
                <path d="M17.66 17.66l2.12 2.12" />
                <path d="M1 12h3" />
                <path d="M20 12h3" />
                <path d="M4.22 19.78l2.12-2.12" />
                <path d="M17.66 6.34l2.12-2.12" />
            </>
        },
        "h-5 w-5 text-gray-400 hover:text-white transition-colors",
        "Settings",
    )
}

#[component]
pub fn UploadIcon() -> impl IntoView {
    stroked_icon(
        view! {
            <path d="M4 14.9A7 7 0 0 1 7 4.1a7 7 0 0 1 12.7 2.1A5 5 0 0 1 19 16h-5" />
            <path d="M12 12v9" />
            <path d="M9 15l3-3 3 3" />
        },
        "h-5 w-5 text-gray-400 hover:text-white transition-colors",
        "Upload",
    )
}
