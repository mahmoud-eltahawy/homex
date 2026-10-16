//! The default error page.
//!
//! Rendered at `/error` by the app's error route. Reads `?reason=` from the
//! query string and displays it when present — the default
//! [`PageContent::render_error`](crate::PageContent::render_error) puts the
//! failing error's `Display` output there.
//!
//! A normal component. An app can render it directly, use it as the body of
//! its `/error` route, or compose it into its own error UI.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn ErrorPage() -> impl IntoView {
    let query = use_query_map();
    let reason = move || query.with(|q| q.get("reason")).unwrap_or_default();

    view! {
        <div class="min-h-[70vh] flex items-center justify-center px-4">
            <div class="max-w-md text-center space-y-4">
                <div class="text-6xl font-black text-cyan-400">"404"</div>
                <h1 class="text-xl font-bold text-white">"Page not found"</h1>
                <p class="text-gray-400 text-sm break-all">
                    {move || {
                        let r = reason();
                        if r.is_empty() {
                            "The URL you tried to open is malformed.".to_string()
                        } else {
                            r
                        }
                    }}
                </p>
                <a
                    href="/"
                    class="inline-block px-4 py-2 rounded-lg \
                               bg-gradient-to-r from-cyan-500 to-blue-500 \
                               hover:from-cyan-400 hover:to-blue-400 \
                               text-white font-bold text-sm transition"
                >
                    "Go home"
                </a>
            </div>
        </div>
    }
}
