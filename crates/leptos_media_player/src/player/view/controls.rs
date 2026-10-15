//! The controls overlay, button row, and sliders.

use leptos::prelude::*;

use crate::ContextBundle;
use crate::player::state::{PlayerDerived, PlayerHandlers, PlayerSignals};
use crate::utils::format_time;

use super::buttons::{FullscreenButton, MuteButton, NavButton, NavDirection, PlayPauseButton};

// ─── Controls overlay ─────────────────────────────────────────────────────

#[component]
pub(crate) fn MediaControls(
    show_controls: impl Fn() + Clone + 'static,
    start_hide_timer: impl Fn() + Clone + 'static,
    #[prop(default = true)] show_fullscreen: bool,
) -> impl IntoView {
    let signals = PlayerSignals::expect();
    let derived = PlayerDerived::expect();

    let class = move || {
        if signals.controls_visible.get() {
            "lmp-controls lmp-controls--visible"
        } else {
            "lmp-controls lmp-controls--hidden"
        }
    };
    let on_mouse_leave = {
        let start = start_hide_timer.clone();
        move |_| start()
    };

    view! {
        <div
            class=class
            on:mouseenter={let show = show_controls.clone(); move |_| show()}
            on:mouseleave=on_mouse_leave
            on:touchstart={let show = show_controls.clone(); move |_| show()}
        >
            <div class="lmp-controls-inner">
                <div class="lmp-controls-title">
                    {move || derived.current_title.get()}
                </div>
                <SeekBar/>
                <ControlButtons show_fullscreen=show_fullscreen/>
            </div>
        </div>
    }
}

// ─── Button row ───────────────────────────────────────────────────────────

#[component]
pub(crate) fn ControlButtons(#[prop(default = true)] show_fullscreen: bool) -> impl IntoView {
    let fullscreen_btn = show_fullscreen.then(|| view! { <FullscreenButton/> });

    view! {
        <div class="lmp-controls-row">
            <NavButton direction=NavDirection::Prev/>
            <PlayPauseButton/>
            <NavButton direction=NavDirection::Next/>
            <div class="lmp-volume">
                <MuteButton/>
                <VolumeSlider/>
            </div>
            <div class="lmp-btn--spacer"></div>
            {fullscreen_btn}
        </div>
    }
}

// ─── Volume slider ────────────────────────────────────────────────────────

#[component]
pub(crate) fn VolumeSlider() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    let vol_value = move || {
        if signals.muted.get() {
            0.0
        } else {
            signals.volume.get()
        }
    };

    view! {
        <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            class="lmp-slider lmp-slider--volume"
            prop:value=vol_value
            on:input=move |ev| handlers.handle_volume.run(ev)
        />
    }
}

// ─── Seek bar ─────────────────────────────────────────────────────────────

#[component]
pub(crate) fn SeekBar() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <div class="lmp-seek">
            <span class="lmp-time">
                {move || format_time(signals.current_time.get())}
            </span>
            <input
                type="range"
                min="0"
                class="lmp-slider lmp-slider--seek"
                prop:max=signals.duration
                prop:value=signals.current_time
                on:input=move |ev| handlers.handle_seek.run(ev)
            />
            <span class="lmp-time">
                {move || format_time(signals.duration.get())}
            </span>
        </div>
    }
}
