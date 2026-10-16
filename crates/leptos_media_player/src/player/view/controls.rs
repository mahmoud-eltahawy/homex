//! The controls overlay, button row, and sliders.

use leptos::prelude::*;

use crate::ContextBundle;
use crate::player::state::{PlayerDerived, PlayerHandlers, PlayerSignals};
use crate::utils::format_time;

use super::buttons::{FullscreenButton, MuteButton, NavButton, NavDirection, PlayPauseButton};

// ─── Shared class fragments ───────────────────────────────────────────────

/// The range-input surface. The thumb is styled via Tailwind arbitrary
/// pseudo-element variants — Tailwind has no first-class variant for
/// `::-webkit-slider-thumb` / `::-moz-range-thumb`, but `[&::-webkit-slider-thumb]:`
/// reaches them.
const SLIDER: &str = "appearance-none h-1.5 bg-white/20 rounded-full cursor-pointer outline-none border-none \
     [&::-webkit-slider-thumb]:appearance-none \
     [&::-webkit-slider-thumb]:w-3 \
     [&::-webkit-slider-thumb]:h-3 \
     [&::-webkit-slider-thumb]:bg-cyan-400 \
     [&::-webkit-slider-thumb]:rounded-full \
     [&::-webkit-slider-thumb]:cursor-pointer \
     [&::-webkit-slider-thumb]:shadow-lg \
     [&::-webkit-slider-thumb]:shadow-cyan-400/30 \
     [&::-moz-range-thumb]:w-3 \
     [&::-moz-range-thumb]:h-3 \
     [&::-moz-range-thumb]:bg-cyan-400 \
     [&::-moz-range-thumb]:rounded-full \
     [&::-moz-range-thumb]:border-none \
     [&::-moz-range-thumb]:cursor-pointer \
     [&::-moz-range-thumb]:shadow-lg \
     [&::-moz-range-thumb]:shadow-cyan-400/30";

/// The seek/volume time readouts.
const TIME: &str = "text-[clamp(0.6875rem,0.65rem+0.25vw,0.75rem)] font-mono";

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
        let visibility = if signals.controls_visible.get() {
            "opacity-100"
        } else {
            "opacity-0"
        };
        let padding = if signals.fullscreen.get() {
            "p-6"
        } else {
            "p-3 sm:p-5"
        };
        format!(
            "absolute bottom-0 left-0 right-0 \
             bg-gradient-to-t from-black/90 via-black/50 to-transparent \
             transition-opacity duration-300 motion-reduce:transition-none \
             {visibility} {padding}"
        )
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
            <div class="flex flex-col gap-2">
                <div class="text-[clamp(0.75rem,0.7rem+0.25vw,1rem)] font-medium \
                            whitespace-nowrap overflow-hidden text-ellipsis px-1 \
                            drop-shadow-md">
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
        <div class="flex items-center gap-2 sm:gap-3">
            <NavButton direction=NavDirection::Prev/>
            <PlayPauseButton/>
            <NavButton direction=NavDirection::Next/>
            <div class="flex items-center gap-2">
                <MuteButton/>
                <VolumeSlider/>
            </div>
            <div class="flex-1"></div>
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
            class=format!("{SLIDER} w-16 sm:w-20 max-[479px]:hidden")
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
        <div class="flex items-center gap-2">
            <span class=TIME>
                {move || format_time(signals.current_time.get())}
            </span>
            <input
                type="range"
                min="0"
                class=format!("{SLIDER} flex-1")
                prop:max=signals.duration
                prop:value=signals.current_time
                on:input=move |ev| handlers.handle_seek.run(ev)
            />
            <span class=TIME>
                {move || format_time(signals.duration.get())}
            </span>
        </div>
    }
}
