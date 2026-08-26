use leptos::{either::Either, prelude::*};

// ─── Tooltip & Icon Wrappers (unchanged from your polished version) ───────

#[component]
fn Tooltip(
    children: Children,
    #[prop(optional)] text: Option<&'static str>,
    #[prop(default = TooltipPosition::Bottom)] position: TooltipPosition,
) -> impl IntoView {
    let position_class = match position {
        TooltipPosition::Top => "bottom-full left-1/2 -translate-x-1/2 mb-2",
        TooltipPosition::Bottom => "top-full left-1/2 -translate-x-1/2 mt-2",
        TooltipPosition::Left => "right-full top-1/2 -translate-y-1/2 mr-2",
        TooltipPosition::Right => "left-full top-1/2 -translate-y-1/2 ml-2",
    };

    view! {
        <div class="relative inline-flex group">
            {children()}
            {text.map(|txt| view! {
                <span class=format!(
                    "absolute {} px-2 py-1 rounded-md bg-gray-900 text-white text-xs font-medium \
                     opacity-0 group-hover:opacity-100 transition-opacity duration-200 \
                     pointer-events-none whitespace-nowrap border border-white/10 shadow-lg z-50",
                    position_class
                )>
                    {txt}
                </span>
            })}
        </div>
    }
}

#[derive(Clone, Copy)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[component]
fn Icon(
    children: Children,
    #[prop(into)] class: String,
    #[prop(into)] label: String,
    #[prop(default = false)] filled: bool,
    #[prop(default = "1.5")] stroke_width: &'static str,
    #[prop(optional)] tooltip: Option<&'static str>,
) -> impl IntoView {
    let svg = view! {
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
    };

    if let Some(txt) = tooltip {
        Either::Left(view! {
            <Tooltip text=txt>
                {svg}
            </Tooltip>
        })
    } else {
        Either::Right(svg)
    }
}

// ─── Icon Components ───────────────────────────────────────────────────────

#[component]
pub fn SearchIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Search" tooltip="Search">
            <circle cx="11" cy="11" r="7" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
            <circle cx="8.5" cy="8.5" r="0.5" fill="currentColor" />
        </Icon>
    }
}

#[component]
pub fn DownloadIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Download" tooltip="Download">
            <path d="M12 3v12" />
            <polyline points="7 10 12 15 17 10" />
            <path d="M5 21h14" />
            <path d="M7 18h10" /> // subtle tray lines
        </Icon>
    }
}

#[component]
pub fn PlayIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6" label="Play" tooltip="Play">
            <polygon points="5 3 19 12 5 21 5 3" fill="currentColor" stroke="none" />
            <polygon points="7 6 17 12 7 18" fill="none" stroke="rgba(255,255,255,0.3)" />
        </Icon>
    }
}

#[component]
pub fn PauseIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6" label="Pause" tooltip="Pause">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
            <line x1="6" y1="12" x2="10" y2="12" stroke="currentColor" opacity="0.5" />
            <line x1="14" y1="12" x2="18" y2="12" stroke="currentColor" opacity="0.5" />
        </Icon>
    }
}

#[component]
pub fn ClockIcon() -> impl IntoView {
    view! {
        <Icon class="h-4 w-4" label="Clock" tooltip="Clock">
            <circle cx="12" cy="12" r="9" />
            <polyline points="12 7 12 12 15 15" />
            <circle cx="12" cy="12" r="1" fill="currentColor" />
        </Icon>
    }
}

#[component]
pub fn DeleteIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Delete" tooltip="Delete">
            <path d="M3 6h18" />
            <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
            <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
            <line x1="10" y1="11" x2="10" y2="17" />
            <line x1="14" y1="11" x2="14" y2="17" />
        </Icon>
    }
}

#[component]
pub fn UpArrow() -> impl IntoView {
    view! {
        <Icon class="h-4 w-4" label="Go Up" tooltip="Go Up">
            <polyline points="6 15 12 9 18 15" />
        </Icon>
    }
}

#[component]
pub fn DownArrow() -> impl IntoView {
    view! {
        <Icon class="h-4 w-4" label="Go Down" tooltip="Go Down">
            <polyline points="6 9 12 15 18 9" />
        </Icon>
    }
}

#[component]
pub fn SortIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Sort Items" tooltip="Sort Items">
            <path d="M3 6h18M3 12h12M3 18h6" />
            <path d="M17 15v6M17 21l-3-3M17 21l3-3" />
            <path d="M7 9v6M7 15l-3-3M7 15l3-3" />
        </Icon>
    }
}

#[component]
pub fn VolumeIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Volume" tooltip="Volume">
            <path d="M11 5L6 9H2v6h4l5 4V5z" />
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
        </Icon>
    }
}

#[component]
pub fn MuteIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Mute" tooltip="Mute">
            <path d="M11 5L6 9H2v6h4l5 4V5z" />
            <line x1="23" y1="9" x2="17" y2="15" />
            <line x1="17" y1="9" x2="23" y2="15" />
        </Icon>
    }
}

#[component]
pub fn FullscreenIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Full Screen" tooltip="Full Screen">
            <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M12 8v8M8 12h8" opacity="0.5" />
        </Icon>
    }
}

#[component]
pub fn FullscreenExitIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Full Screen Exit" tooltip="Full Screen Exit">
            <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M12 8v8M8 12h8" opacity="0.5" />
        </Icon>
    }
}

#[component]
pub fn XIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5" label="Cancel" tooltip="Cancel">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
        </Icon>
    }
}

#[component]
pub fn MovieIcon() -> impl IntoView {
    // Clapperboard – universally recognized symbol for film
    view! {
        <Icon class="w-6 h-6 text-amber-400" label="Movie" tooltip="Movie" filled=true>
            <path d="M4 5h16a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z" />
            <path d="M3 10h18" stroke="#0c0b1a" stroke-width="2" />
            <path d="M6 5v5M10 5v5M14 5v5M18 5v5" stroke="#0c0b1a" stroke-width="2" />
            <path d="M8 10l4-5M12 10l4-5" stroke="#0c0b1a" stroke-width="2" fill="none" />
        </Icon>
    }
}

#[component]
pub fn SeriesIcon() -> impl IntoView {
    // Retro TV – universally recognized for television series
    view! {
        <Icon class="w-6 h-6 text-purple-400" label="Series" tooltip="Series" filled=true>
            <rect x="3" y="4" width="18" height="13" rx="2" />
            <path d="M8 20h8l-1-3H9z" fill="currentColor" />
            <path d="M10 8h4v3h-4z" fill="#0c0b1a" />
            <path d="M6 4l3 3M18 4l-3 3" stroke="#0c0b1a" stroke-width="2" fill="none" />
        </Icon>
    }
}

#[component]
pub fn AudioIcon() -> impl IntoView {
    // Headphones – global audio symbol
    view! {
        <Icon class="w-6 h-6 text-cyan-400" label="Audio" tooltip="Audio" filled=true>
            <path d="M4 13v-1a8 8 0 1 1 16 0v1" />
            <path d="M4 13a2 2 0 0 0-2 2v1a2 2 0 0 0 2 2h2v-5H4zM20 13a2 2 0 0 1 2 2v1a2 2 0 0 1-2 2h-2v-5h2z" />
            <rect x="6" y="14" width="12" height="4" rx="1" fill="#0c0b1a" />
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
pub fn ViewAllIcon() -> impl IntoView {
    view! {
        <Icon class="w-5 h-5 text-gray-400 group-hover:text-white transition-colors" label="Show All" tooltip="Show All" filled=true>
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
pub fn PrevPageIcon() -> impl IntoView {
    view! {
        <Icon class="w-5 h-5" label="Previous" tooltip="Previous">
            <polyline points="9 6 15 12 9 18" />
        </Icon>
    }
}

#[component]
pub fn NextPageIcon() -> impl IntoView {
    view! {
        <Icon class="w-5 h-5" label="Next" tooltip="Next">
            <polyline points="15 6 9 12 15 18" />
        </Icon>
    }
}

#[component]
pub fn LoadingIcon() -> impl IntoView {
    view! {
        <Icon class="h-8 w-8 animate-spin text-cyan-400" label="Loading" tooltip="Loading">
            <circle cx="12" cy="12" r="10" stroke-dasharray="70 200" stroke-dashoffset="0" />
            <circle cx="12" cy="12" r="2" fill="currentColor" />
        </Icon>
    }
}

#[component]
pub fn RetryIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6 text-gray-400 hover:text-white transition-colors" label="Retry" tooltip="Retry">
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1.03 6.36 2.36L21 8" />
            <polyline points="21 3 21 8 16 8" />
        </Icon>
    }
}

#[component]
pub fn ErrorIcon() -> impl IntoView {
    view! {
        <Icon class="h-6 w-6 text-red-400" label="Something Went Wrong" tooltip="Something Went Wrong">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <circle cx="12" cy="16" r="0.5" fill="currentColor" />
        </Icon>
    }
}

#[component]
pub fn MediaCubeLogo() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="w-8 h-8" role="img" aria-label="MediaCube">
            <defs>
                <linearGradient id="logoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#22d3ee" />
                    <stop offset="100%" stop-color="#3b82f6" />
                </linearGradient>
            </defs>
            <rect x="2" y="2" width="20" height="20" rx="5" fill="url(#logoGrad)" />
            <path d="M10 7 L10 17 L17 12 Z" fill="#0a0a0f" />
            <circle cx="12" cy="12" r="1.5" fill="#22d3ee" />
        </svg>
    }
}

#[component]
pub fn SettingsIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5 text-gray-400 hover:text-white transition-colors" label="Settings" tooltip="Settings">
            <circle cx="12" cy="12" r="3" />
            <path d="M12 1v3M12 20v3M4.22 4.22l2.12 2.12M17.66 17.66l2.12 2.12M1 12h3M20 12h3M4.22 19.78l2.12-2.12M17.66 6.34l2.12-2.12" />
            <circle cx="12" cy="12" r="0.5" fill="currentColor" />
        </Icon>
    }
}

#[component]
pub fn UploadIcon() -> impl IntoView {
    view! {
        <Icon class="h-5 w-5 text-gray-400 hover:text-white transition-colors" label="Upload" tooltip="Upload">
            <path d="M4 14.9A7 7 0 0 1 7 4.1a7 7 0 0 1 12.7 2.1A5 5 0 0 1 19 16h-5" />
            <polyline points="12 12 12 20 9 17 12 12 15 17" />
        </Icon>
    }
}

// ─── Poster SVGs (Artistic Overhaul) ─────────────────────────────────────

#[component]
pub fn MoviePosterSvg() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 900" width="100%" height="100%" role="img" aria-label="Movie poster placeholder">
            <defs>
                <linearGradient id="bg-movie" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#0f172a" />
                    <stop offset="100%" stop-color="#1e293b" />
                </linearGradient>
                <radialGradient id="spotlight" cx="50%" cy="35%" r="60%">
                    <stop offset="0%" stop-color="#38bdf8" stop-opacity="0.35" />
                    <stop offset="60%" stop-color="#38bdf8" stop-opacity="0.05" />
                    <stop offset="100%" stop-color="#38bdf8" stop-opacity="0" />
                </radialGradient>
                <linearGradient id="film" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stop-color="#64748b" />
                    <stop offset="50%" stop-color="#94a3b8" />
                    <stop offset="100%" stop-color="#64748b" />
                </linearGradient>
                <filter id="grain">
                    <feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" result="noise" />
                    <feColorMatrix type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.06 0" in="noise" result="coloredNoise" />
                    <feBlend in="SourceGraphic" in2="coloredNoise" mode="multiply" />
                </filter>
            </defs>

            <rect width="600" height="900" fill="url(#bg-movie)" />
            <rect width="600" height="900" fill="url(#spotlight)" />

            <g transform="translate(-50, 650) rotate(-15)" filter="url(#grain)">
                <rect x="0" y="0" width="800" height="60" fill="url(#film)" opacity="0.9" />
                {[20, 80, 140, 200, 260, 320, 380, 440, 500, 560, 620, 680, 740].into_iter().map(|x| {
                    view! {
                        <rect x=x y="10" width="16" height="16" rx="2" fill="#0f172a" />
                        <rect x=x+2 y="34" width="16" height="16" rx="2" fill="#0f172a" />
                    }
                }).collect_view()}
                <rect x="0" y="45" width="800" height="10" fill="#0f172a" opacity="0.3" />
            </g>

            <g transform="translate(300, 320)">
                <circle r="120" fill="none" stroke="#e2e8f0" stroke-width="8" />
                <circle r="100" fill="none" stroke="#e2e8f0" stroke-width="2" />
                <circle r="80" fill="none" stroke="#e2e8f0" stroke-width="1" stroke-dasharray="5 5" />
                <circle r="20" fill="#e2e8f0" />
                <circle r="10" fill="#38bdf8" opacity="0.6" />
                <line x1="-70" y1="0" x2="70" y2="0" stroke="#e2e8f0" stroke-width="4" />
                <line x1="0" y1="-70" x2="0" y2="70" stroke="#e2e8f0" stroke-width="4" />
                <line x1="-50" y1="-50" x2="50" y2="50" stroke="#e2e8f0" stroke-width="4" />
                <line x1="50" y1="-50" x2="-50" y2="50" stroke="#e2e8f0" stroke-width="4" />
            </g>

            <text x="300" y="600" font-family="Arial, Helvetica, sans-serif" font-size="42" font-weight="900" letter-spacing="8" fill="#ffffff" text-anchor="middle">MOVIE</text>
            <text x="300" y="640" font-family="Arial, Helvetica, sans-serif" font-size="16" letter-spacing="4" fill="#94a3b8" text-anchor="middle">NO POSTER AVAILABLE</text>
        </svg>
    }
}

#[component]
pub fn SeriesPosterSvg() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 900" width="100%" height="100%" role="img" aria-label="Series poster placeholder">
            <defs>
                <linearGradient id="bg-series" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#111827" />
                    <stop offset="100%" stop-color="#312e81" />
                </linearGradient>
                <radialGradient id="glow" cx="50%" cy="40%" r="50%">
                    <stop offset="0%" stop-color="#8b5cf6" stop-opacity="0.5" />
                    <stop offset="100%" stop-color="#8b5cf6" stop-opacity="0" />
                </radialGradient>
                <linearGradient id="screen" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stop-color="#e0e7ff" />
                    <stop offset="100%" stop-color="#a5b4fc" />
                </linearGradient>
                <filter id="shadow">
                    <feDropShadow dx="0" dy="15" stdDeviation="20" flood-color="#000" flood-opacity="0.6" />
                </filter>
                <filter id="scanlines">
                    <feComponentTransfer>
                        <feFuncA type="linear" slope="0.2" />
                    </feComponentTransfer>
                </filter>
            </defs>

            <rect width="600" height="900" fill="url(#bg-series)" />
            <rect width="600" height="900" fill="url(#glow)" />

            <g transform="translate(150, 220)" filter="url(#shadow)">
                <rect width="300" height="200" rx="15" fill="#1e1b4b" stroke="#818cf8" stroke-width="5" />
                <path d="M120 -10 L150 -40 L180 -10" fill="none" stroke="#818cf8" stroke-width="5" stroke-linecap="round" />
                <circle cx="150" cy="-40" r="5" fill="#c4b5fd" />
                <rect x="10" y="10" width="280" height="150" rx="8" fill="url(#screen)" />
                <polygon points="140,65 140,115 170,90" fill="#1e1b4b" />
                <rect x="10" y="10" width="280" height="150" rx="8" fill="url(#screen)" filter="url(#scanlines)" opacity="0.15" />
                <rect x="20" y="170" width="260" height="4" fill="#6366f1" opacity="0.5" />
            </g>

            <text x="300" y="600" font-family="Arial, Helvetica, sans-serif" font-size="42" font-weight="900" letter-spacing="8" fill="#ffffff" text-anchor="middle">SERIES</text>
            <text x="300" y="640" font-family="Arial, Helvetica, sans-serif" font-size="16" letter-spacing="4" fill="#a5b4fc" text-anchor="middle">NO POSTER AVAILABLE</text>
        </svg>
    }
}

#[component]
pub fn MusicPosterSvg() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 600" width="100%" height="100%" role="img" aria-label="Music artwork placeholder">
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
                <filter id="glow">
                    <feGaussianBlur stdDeviation="8" result="coloredBlur"/>
                    <feMerge>
                        <feMergeNode in="coloredBlur"/>
                        <feMergeNode in="SourceGraphic"/>
                    </feMerge>
                </filter>
            </defs>

            <rect width="600" height="600" fill="url(#bg-music)" />

            <circle cx="300" cy="250" r="180" fill="#ffffff" opacity="0.05" />
            <circle cx="300" cy="250" r="140" fill="#ffffff" opacity="0.08" />
            <circle cx="300" cy="250" r="120" fill="url(#vinyl)" filter="url(#glow)" />
            <circle cx="300" cy="250" r="110" fill="none" stroke="#1e1b4b" stroke-width="1" opacity="0.4" />
            <circle cx="300" cy="250" r="95" fill="none" stroke="#1e1b4b" stroke-width="1" opacity="0.3" />
            <circle cx="300" cy="250" r="80" fill="none" stroke="#1e1b4b" stroke-width="1" opacity="0.2" />
            <circle cx="300" cy="250" r="30" fill="#1e1b4b" />
            <circle cx="300" cy="250" r="12" fill="#fce7f3" />

            <g transform="translate(420, 180) rotate(25)">
                <rect x="0" y="0" width="8" height="120" rx="4" fill="#e2e8f0" />
                <circle cx="0" cy="0" r="15" fill="#64748b" />
                <circle cx="0" cy="0" r="8" fill="#94a3b8" />
                <circle cx="30" cy="120" r="5" fill="#cbd5e1" />
            </g>

            <g transform="translate(0, 430)">
                {[0, 25, 50, 75, 100, 125, 150, 175, 200, 225, 250, 275, 300, 325, 350, 375, 400, 425, 450, 475, 500].into_iter().map(|x| {
                    let height = (x % 70 + 30).min(110);
                    let y = 40 - (height / 10);
                    view! {
                        <rect x=x y=y width="12" height=height rx="2" fill="url(#wave)" />
                    }
                }).collect_view()}
            </g>

            <text x="300" y="550" font-family="Arial, Helvetica, sans-serif" font-size="36" font-weight="900" letter-spacing="6" fill="#ffffff" text-anchor="middle">MUSIC</text>
            <text x="300" y="580" font-family="Arial, Helvetica, sans-serif" font-size="14" letter-spacing="3" fill="#fbcfe8" text-anchor="middle">NO ARTWORK AVAILABLE</text>
        </svg>
    }
}
