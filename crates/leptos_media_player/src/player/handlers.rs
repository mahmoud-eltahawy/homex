//! Handler factories and lifecycle effects.
//!
//! Everything here is "glue": closures that read the DOM element behind a
//! `NodeRef` and push to/from the reactive signals in
//! [`state`](super::state). Nothing in this module renders.

use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use super::state::PlayerSignals;
use crate::MediaItem;

// ─── Effects ──────────────────────────────────────────────────────────────

/// Keep `current_idx` inside the bounds of `items` when the list shrinks.
pub(crate) fn install_clamp_effect(items: Signal<Vec<MediaItem>>, current_idx: RwSignal<usize>) {
    Effect::new(move |_| {
        let n = items.get().len();
        if n == 0 {
            current_idx.set(0);
        } else if current_idx.get() >= n {
            current_idx.set(n - 1);
        }
    });
}

/// Reload the `<video>` whenever the source URL changes, resetting playback
/// state and honouring the "play after load" flag set by the `ended` handler.
pub(crate) fn install_src_reload_effect(
    video_ref: NodeRef<html::Video>,
    signals: PlayerSignals,
    current_src: Signal<String>,
) {
    Effect::new(move || {
        let src = current_src.get();
        if let Some(video) = video_ref.get() {
            video.set_src(&src);
            video.load();
            signals.playing.set(false);
            signals.current_time.set(0.0);
            signals.duration.set(0.0);
            if signals.play_after_load.get_untracked() {
                signals.play_after_load.set(false);
                let _ = video.play();
            }
        }
    });
}

// ─── Action factories ─────────────────────────────────────────────────────
// All factories return `impl Fn(..) + Copy` so they can be wrapped in
// `Callback::new(..)` and shared between click handlers and the keydown
// dispatcher.

pub(crate) fn make_toggle_play(
    video_ref: NodeRef<html::Video>,
    playing: RwSignal<bool>,
) -> impl Fn(()) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if playing.get() {
                video.pause().ok();
            } else {
                let _ = video.play();
            }
        }
    }
}

pub(crate) fn make_toggle_mute(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    last_volume: RwSignal<f64>,
    muted: RwSignal<bool>,
) -> impl Fn(()) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if muted.get() {
                video.set_muted(false);
                let restore = last_volume.get().max(0.1);
                video.set_volume(restore);
                volume.set(restore);
                muted.set(false);
            } else {
                last_volume.set(volume.get().max(0.1));
                video.set_muted(true);
                muted.set(true);
            }
        }
    }
}

/// Toggle fullscreen on the player *container*, not on the video element.
/// This is what keeps the controls overlay visible in fullscreen — the
/// overlay lives outside the video but inside the container.
pub(crate) fn make_toggle_fullscreen(player_ref: NodeRef<html::Div>) -> impl Fn(()) + Copy {
    move |_| {
        if let Some(el) = player_ref.get() {
            if document().fullscreen_element().is_none() {
                let _ = el.request_fullscreen();
            } else {
                document().exit_fullscreen();
            }
        }
    }
}

pub(crate) fn make_seek_relative(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(f64) + Copy {
    move |delta: f64| {
        if let Some(video) = video_ref.get() {
            let dur = video.duration();
            if !dur.is_finite() || dur <= 0.0 {
                return;
            }
            let new_time = (video.current_time() + delta).clamp(0.0, dur);
            video.set_current_time(new_time);
            current_time.set(new_time);
        }
    }
}

pub(crate) fn make_nudge_volume(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    last_volume: RwSignal<f64>,
) -> impl Fn(f64) + Copy {
    move |delta: f64| {
        if let Some(video) = video_ref.get() {
            let new_vol = (video.volume() + delta).clamp(0.0, 1.0);
            video.set_volume(new_vol);
            video.set_muted(new_vol == 0.0);
            volume.set(new_vol);
            muted.set(new_vol == 0.0);
            if new_vol > 0.0 {
                last_volume.set(new_vol);
            }
        }
    }
}

pub(crate) fn make_seek_to_fraction(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(f64) + Copy {
    move |frac: f64| {
        if let Some(video) = video_ref.get() {
            let dur = video.duration();
            if !dur.is_finite() || dur <= 0.0 {
                return;
            }
            let new_time = (dur * frac.clamp(0.0, 1.0)).clamp(0.0, dur);
            video.set_current_time(new_time);
            current_time.set(new_time);
        }
    }
}

// ─── Input-element handler factories ──────────────────────────────────────

pub(crate) fn make_handle_loaded_metadata(
    video_ref: NodeRef<html::Video>,
    duration: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            duration.set(video.duration());
        }
    }
}

pub(crate) fn make_handle_time_update(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            current_time.set(video.current_time());
        }
    }
}

pub(crate) fn make_handle_seek(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_current_time(val);
            current_time.set(val);
        }
    }
}

pub(crate) fn make_handle_volume(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    last_volume: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_volume(val);
            video.set_muted(val == 0.0);
            volume.set(val);
            muted.set(val == 0.0);
            if val > 0.0 {
                last_volume.set(val);
            }
        }
    }
}
