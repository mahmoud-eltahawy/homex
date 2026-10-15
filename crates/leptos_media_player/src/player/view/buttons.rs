//! Individual player buttons.

use leptos::either::Either;
use leptos::prelude::*;

use crate::ContextBundle;
use crate::icons::{
    FullscreenExitIcon, FullscreenIcon, MuteIcon, NextPageIcon, PauseIcon, PlayIcon, PrevPageIcon,
    VolumeIcon,
};
use crate::player::state::{PlayerDerived, PlayerHandlers, PlayerNav, PlayerSignals};

// ─── Nav direction ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) enum NavDirection {
    Prev,
    Next,
}

impl NavDirection {
    fn aria_label(self) -> &'static str {
        match self {
            Self::Prev => "Previous",
            Self::Next => "Next",
        }
    }
}

// ─── Nav button ───────────────────────────────────────────────────────────

#[component]
pub(crate) fn NavButton(direction: NavDirection) -> impl IntoView {
    let derived = PlayerDerived::expect();
    let nav = PlayerNav::expect();

    let show = derived.can_navigate;
    let disabled = Signal::derive(move || match direction {
        NavDirection::Prev => !derived.has_prev.get(),
        NavDirection::Next => !derived.has_next.get(),
    });
    let on_click = match direction {
        NavDirection::Prev => nav.on_prev,
        NavDirection::Next => nav.on_next,
    };
    let label = direction.aria_label();

    view! {
        <Show when=move || show.get()>
            <button
                class="lmp-btn"
                on:click=move |ev| on_click.run(ev)
                disabled=move || disabled.get()
                aria-label=label
            >
                {move || match direction {
                    NavDirection::Prev => Either::Left(view! { <PrevPageIcon/> }),
                    NavDirection::Next => Either::Right(view! { <NextPageIcon/> }),
                }}
            </button>
        </Show>
    }
}

// ─── Play / pause ─────────────────────────────────────────────────────────

#[component]
pub(crate) fn PlayPauseButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            class="lmp-btn"
            on:click=move |_| handlers.toggle_play.run(())
            aria-label="Play or pause"
        >
            {move || if signals.playing.get() {
                Either::Left(PauseIcon())
            } else {
                Either::Right(PlayIcon())
            }}
        </button>
    }
}

// ─── Mute ─────────────────────────────────────────────────────────────────

#[component]
pub(crate) fn MuteButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            class="lmp-btn"
            on:click=move |_| handlers.toggle_mute.run(())
            aria-label="Mute or unmute"
        >
            {move || if signals.muted.get() || signals.volume.get() == 0.0 {
                Either::Left(MuteIcon())
            } else {
                Either::Right(VolumeIcon())
            }}
        </button>
    }
}

// ─── Fullscreen ───────────────────────────────────────────────────────────

#[component]
pub(crate) fn FullscreenButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            class="lmp-btn"
            on:click=move |_| handlers.toggle_fullscreen.run(())
            aria-label="Toggle fullscreen"
        >
            {move || if signals.fullscreen.get() {
                Either::Left(FullscreenExitIcon())
            } else {
                Either::Right(FullscreenIcon())
            }}
        </button>
    }
}
