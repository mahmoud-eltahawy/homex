//! The HomeX brand mark.

use leptos::prelude::*;

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
