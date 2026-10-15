//! Signal, derived-signal, and handler bundles owned by the player.
//!
//! These are the "shape" of the player's reactive state, provided to the
//! subtree via [`ContextBundle`](crate::ContextBundle) so that every control
//! and the shortcut dispatcher can reach them without prop drilling.

use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::MediaItem;

// ─── Signals ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) struct PlayerSignals {
    pub playing: RwSignal<bool>,
    pub current_time: RwSignal<f64>,
    pub duration: RwSignal<f64>,
    pub volume: RwSignal<f64>,
    pub last_volume: RwSignal<f64>,
    pub muted: RwSignal<bool>,
    pub fullscreen: RwSignal<bool>,
    pub controls_visible: RwSignal<bool>,
    pub play_after_load: RwSignal<bool>,
}

impl PlayerSignals {
    pub(crate) fn new() -> Self {
        Self {
            playing: RwSignal::new(false),
            current_time: RwSignal::new(0.0),
            duration: RwSignal::new(0.0),
            volume: RwSignal::new(1.0),
            last_volume: RwSignal::new(1.0),
            muted: RwSignal::new(false),
            fullscreen: RwSignal::new(false),
            controls_visible: RwSignal::new(true),
            play_after_load: RwSignal::new(false),
        }
    }
}

// ─── Derived signals ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) struct PlayerDerived {
    pub current_src: Signal<String>,
    pub current_title: Signal<String>,
    pub current_artwork: Signal<Option<String>>,
    pub has_prev: Signal<bool>,
    pub has_next: Signal<bool>,
    pub can_navigate: Signal<bool>,
}

impl PlayerDerived {
    pub(crate) fn build(
        items: Signal<Vec<MediaItem>>,
        current_idx: RwSignal<usize>,
        artwork_fallback: Option<String>,
    ) -> Self {
        let current_item = Memo::new(move |_| items.get().get(current_idx.get()).cloned());

        let current_src =
            Signal::derive(move || current_item.get().map(|i| i.src).unwrap_or_default());
        let current_title =
            Signal::derive(move || current_item.get().map(|i| i.title).unwrap_or_default());
        let current_artwork = {
            let fallback = artwork_fallback;
            Signal::derive(move || {
                current_item
                    .get()
                    .and_then(|i| i.artwork)
                    .or_else(|| fallback.clone())
            })
        };

        let has_prev = Signal::derive(move || current_idx.get() > 0);
        let has_next = Signal::derive(move || current_idx.get() + 1 < items.get().len());
        let can_navigate = Signal::derive(move || items.get().len() > 1);

        Self {
            current_src,
            current_title,
            current_artwork,
            has_prev,
            has_next,
            can_navigate,
        }
    }
}

// ─── Handler bundles ──────────────────────────────────────────────────────

/// Actions the player exposes. Buttons and keyboard shortcuts both call
/// these — the only thing that distinguishes a click from a hotkey is
/// *which* event triggers the callback, not the callback itself.
#[derive(Clone, Copy)]
pub(crate) struct PlayerHandlers {
    pub toggle_play: Callback<()>,
    pub toggle_mute: Callback<()>,
    pub toggle_fullscreen: Callback<()>,
    /// Seek by `delta` seconds (positive forward, negative backward).
    pub seek_relative: Callback<f64>,
    /// Nudge volume by `delta` (positive up, negative down), 0.0..=1.0.
    pub nudge_volume: Callback<f64>,
    /// Seek to `fraction` of duration (0.0..=1.0).
    pub seek_to_fraction: Callback<f64>,

    // Input-element callbacks — these need the concrete DOM event.
    pub handle_seek: Callback<web_sys::Event>,
    pub handle_volume: Callback<web_sys::Event>,
}

#[derive(Clone, Copy)]
pub(crate) struct PlayerNav {
    pub on_prev: Callback<MouseEvent>,
    pub on_next: Callback<MouseEvent>,
}
