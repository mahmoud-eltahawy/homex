//! Keyboard shortcut dispatch.
//!
//! Kept separate from the component so the mapping from key to action is
//! readable in one place and testable in isolation (the pure decision
//! function `should_ignore_hotkey` has no DOM side effects).

use leptos::{callback::Callable, wasm_bindgen::JsCast};
use web_sys::{HtmlElement, KeyboardEvent};

use super::state::PlayerHandlers;

/// Map a keydown event to a handler call. Returns `true` if the key was
/// recognized and consumed; the caller can use that to decide whether to
/// stop propagation further up the tree.
pub(crate) fn handle_keydown(ev: &KeyboardEvent, handlers: PlayerHandlers) -> bool {
    if should_ignore_hotkey(ev) {
        return false;
    }

    match ev.key().as_str() {
        " " | "k" | "K" => {
            ev.prevent_default();
            handlers.toggle_play.run(());
            true
        }
        "m" | "M" => {
            ev.prevent_default();
            handlers.toggle_mute.run(());
            true
        }
        "f" | "F" => {
            ev.prevent_default();
            handlers.toggle_fullscreen.run(());
            true
        }
        "ArrowLeft" => {
            ev.prevent_default();
            handlers.seek_relative.run(-5.0);
            true
        }
        "ArrowRight" => {
            ev.prevent_default();
            handlers.seek_relative.run(5.0);
            true
        }
        "ArrowUp" => {
            ev.prevent_default();
            handlers.nudge_volume.run(0.05);
            true
        }
        "ArrowDown" => {
            ev.prevent_default();
            handlers.nudge_volume.run(-0.05);
            true
        }
        "Home" => {
            ev.prevent_default();
            handlers.seek_to_fraction.run(0.0);
            true
        }
        "End" => {
            ev.prevent_default();
            handlers.seek_to_fraction.run(1.0);
            true
        }
        k if k.len() == 1 && k.as_bytes()[0].is_ascii_digit() => {
            ev.prevent_default();
            let pct = (k.as_bytes()[0] - b'0') as f64 / 10.0;
            handlers.seek_to_fraction.run(pct);
            true
        }
        _ => false,
    }
}

/// Hotkeys should not fire when the user is holding a browser/OS modifier,
/// nor when the event target is a form field the user is typing in or
/// nudging. Note: we deliberately do *not* skip `<button>` targets — if we
/// did, `f` on a focused fullscreen button wouldn't work.
fn should_ignore_hotkey(ev: &KeyboardEvent) -> bool {
    if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
        return true;
    }
    let Some(target) = ev.target() else {
        return false;
    };
    let Ok(el) = target.dyn_into::<HtmlElement>() else {
        return false;
    };
    matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT") || el.is_content_editable()
}
