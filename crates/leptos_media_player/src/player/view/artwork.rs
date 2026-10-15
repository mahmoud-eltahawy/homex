//! The audio-mode artwork overlay.

use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub(crate) fn AudioArtworkOverlay(
    artwork: Signal<Option<String>>,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="lmp-artwork" on:click=on_click>
            {move || artwork.get().map(|url| view! {
                <div
                    class="lmp-artwork-bg"
                    style=format!("background-image: url('{url}');")
                ></div>
                <img src=url class="lmp-artwork-img" alt=""/>
            })}
        </div>
    }
}
