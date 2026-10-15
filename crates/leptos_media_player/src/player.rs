//! The [`MediaPlayer`] component and its supporting modules.

use leptos::ev::fullscreenchange;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_use::{UseTimeoutFnReturn, use_document, use_event_listener, use_timeout_fn};
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent};

use crate::MediaItem;
use crate::style::stylesheet;

mod handlers;
mod shortcuts;
mod state;
mod view;

use handlers::{
    install_clamp_effect, install_src_reload_effect, make_handle_loaded_metadata, make_handle_seek,
    make_handle_time_update, make_handle_volume, make_nudge_volume, make_seek_relative,
    make_seek_to_fraction, make_toggle_fullscreen, make_toggle_mute, make_toggle_play,
};
use state::{PlayerDerived, PlayerHandlers, PlayerNav, PlayerSignals};
use view::{AudioArtworkOverlay, MediaControls};

// ─── Context bundles ──────────────────────────────────────────────────────

use crate::ContextBundle;

impl ContextBundle for PlayerSignals {}
impl ContextBundle for PlayerDerived {}
impl ContextBundle for PlayerHandlers {}
impl ContextBundle for PlayerNav {}

// ─── Public component ─────────────────────────────────────────────────────

/// The video (or audio) surface with its controls.
///
/// Pair it with a [`Playlist`](crate::Playlist) that shares the same
/// `items` and `current_idx` signals — or use it standalone.
///
/// ## Keyboard shortcuts
///
/// Click anywhere on the player (or Tab to it) to focus it, then:
///
/// | Key            | Action                    |
/// |----------------|---------------------------|
/// | `Space`, `k`   | Toggle play / pause       |
/// | `m`            | Toggle mute               |
/// | `f`            | Toggle fullscreen         |
/// | `←` / `→`      | Seek back / forward 5 s   |
/// | `↑` / `↓`      | Volume up / down 5 %      |
/// | `0` – `9`      | Jump to 0 % – 90 %        |
/// | `Home` / `End` | Jump to start / end       |
#[component]
pub fn MediaPlayer(
    items: Signal<Vec<MediaItem>>,
    current_idx: RwSignal<usize>,
    #[prop(default = false)] audio: bool,
    #[prop(default = None)] artwork: Option<String>,
) -> impl IntoView {
    let player_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let signals = PlayerSignals::new();
    let derived = PlayerDerived::build(items, current_idx, artwork.clone());

    install_clamp_effect(items, current_idx);
    install_src_reload_effect(video_ref, signals, derived.current_src);

    // ── Controls visibility timeout ───────────────────────────────────
    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_i: i8| signals.controls_visible.set(false), 3000.);
    let start_hide_timer = {
        let stop = stop.clone();
        move || {
            stop();
            start(3);
        }
    };
    let show_controls = {
        let start_hide_timer = start_hide_timer.clone();
        move || {
            signals.controls_visible.set(true);
            start_hide_timer();
        }
    };
    let toggle_controls = {
        let show_controls = show_controls.clone();
        let stop = stop.clone();
        move || {
            if signals.controls_visible.get() {
                signals.controls_visible.set(false);
                stop();
            } else {
                show_controls();
            }
        }
    };

    // ── Fullscreen listener ───────────────────────────────────────────
    let u_document = use_document();
    let _guard = use_event_listener(u_document.clone(), fullscreenchange, move |_| {
        signals
            .fullscreen
            .set(u_document.fullscreen().is_some_and(|x| x));
    });

    // ── Navigation ────────────────────────────────────────────────────
    let on_next: Callback<MouseEvent> = Callback::new(move |_| {
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            current_idx.update(|i| *i += 1);
        }
    });
    let on_prev: Callback<MouseEvent> = Callback::new(move |_| {
        current_idx.update(|i| *i = i.saturating_sub(1));
    });
    let nav = PlayerNav { on_prev, on_next };

    let handle_ended = move |_| {
        signals.playing.set(false);
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            signals.play_after_load.set(true);
            current_idx.update(|i| *i += 1);
        }
    };

    // ── Build handler bundle ──────────────────────────────────────────
    let handlers = PlayerHandlers {
        toggle_play: Callback::new(make_toggle_play(video_ref, signals.playing)),
        toggle_mute: Callback::new(make_toggle_mute(
            video_ref,
            signals.volume,
            signals.last_volume,
            signals.muted,
        )),
        toggle_fullscreen: Callback::new(make_toggle_fullscreen(player_ref)),
        seek_relative: Callback::new(make_seek_relative(video_ref, signals.current_time)),
        nudge_volume: Callback::new(make_nudge_volume(
            video_ref,
            signals.volume,
            signals.muted,
            signals.last_volume,
        )),
        seek_to_fraction: Callback::new(make_seek_to_fraction(video_ref, signals.current_time)),
        handle_seek: Callback::new(make_handle_seek(video_ref, signals.current_time)),
        handle_volume: Callback::new(make_handle_volume(
            video_ref,
            signals.volume,
            signals.muted,
            signals.last_volume,
        )),
    };

    let handle_loaded_metadata = make_handle_loaded_metadata(video_ref, signals.duration);
    let handle_time_update = make_handle_time_update(video_ref, signals.current_time);

    // ── Hotkey dispatcher ─────────────────────────────────────────────
    let handle_keydown = move |ev: KeyboardEvent| {
        shortcuts::handle_keydown(&ev, handlers);
    };

    // ── Click-to-focus ────────────────────────────────────────────────
    // Safari doesn't focus `tabindex="0"` elements on click; Chrome/Firefox
    // do. We do it explicitly for consistency, but skip interactive children
    // so clicking the volume slider doesn't steal its focus.
    let handle_player_click = move |ev: MouseEvent| {
        let Some(target) = ev.target() else {
            return;
        };
        let Ok(el) = target.dyn_into::<HtmlElement>() else {
            return;
        };
        let interactive = el
            .closest("button, input, select, textarea, a, [contenteditable]")
            .ok()
            .flatten()
            .is_some();
        if !interactive && let Some(container) = player_ref.get() {
            let _ = container.focus();
        }
    };

    // ── Install contexts (must precede view!) ─────────────────────────
    PlayerSignals::provide(signals);
    PlayerDerived::provide(derived);
    PlayerHandlers::provide(handlers);
    PlayerNav::provide(nav);

    // ── View ──────────────────────────────────────────────────────────
    let video_class = if audio {
        "lmp-video lmp-video--audio"
    } else {
        "lmp-video"
    };

    let artwork_view = if audio {
        let tc = toggle_controls.clone();
        Some(view! {
            <AudioArtworkOverlay
                artwork=derived.current_artwork
                on_click=move |_| tc()
            />
        })
    } else {
        None
    };

    view! {
        {stylesheet()}
        <div
            node_ref=player_ref
            class="lmp-player"
            tabindex="0"
            on:keydown=handle_keydown
            on:click=handle_player_click
            on:mousemove={let show = show_controls.clone(); move |_| show()}
        >
            <video
                node_ref=video_ref
                title=move || derived.current_title.get()
                class=video_class
                on:loadedmetadata=handle_loaded_metadata
                on:timeupdate=handle_time_update
                on:play=move |_| signals.playing.set(true)
                on:pause=move |_| signals.playing.set(false)
                on:ended=handle_ended
                on:click={let tc = toggle_controls.clone(); move |_| tc()}
                playsinline
            />
            {artwork_view}
            <MediaControls
                show_controls=show_controls.clone()
                start_hide_timer=start_hide_timer
                show_fullscreen=!audio
            />
        </div>
    }
}
