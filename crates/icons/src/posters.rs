//! Large placeholder artwork used when a collection has no poster yet.
//!
//! These are full illustrations rather than icons — they don't go through
//! [`crate::generic::Icon`] and are not sized by the caller's font.

use leptos::prelude::*;

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
