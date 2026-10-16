//! The audio-mode artwork overlay.

use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub(crate) fn AudioArtworkOverlay(
    artwork: Signal<Option<String>>,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div
            class="relative w-full aspect-video flex items-center justify-center \
                   cursor-pointer overflow-hidden \
                   bg-gradient-to-br from-[#1e1e2e] via-[#14141e] to-[#0a0a0f]"
            on:click=on_click
        >
            {move || artwork.get().map(|url| view! {
                <div
                    class="absolute inset-0 opacity-50 bg-cover bg-center \
                           blur-[50px] saturate-150 scale-[1.3]"
                    style=format!("background-image: url('{url}');")
                ></div>
                <img
                    src=url
                    class="relative max-h-[70%] max-w-[70%] object-contain \
                           rounded-2xl shadow-2xl border border-white/10"
                    alt=""
                />
            })}
        </div>
    }
}
