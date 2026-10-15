//! The crate's embedded stylesheet.
//!
//! Every component renders [`stylesheet()`] near the top of its view. The
//! first call in a given subtree emits the `<style>` tag and marks the
//! subtree via Leptos context; subsequent calls (nested players, sibling
//! playlists under a shared parent) become no-ops.
//!
//! When you compose primitives yourself — e.g. `MediaPlayer` and `Playlist`
//! as siblings without a `WatchPage` above them — call `stylesheet()` once
//! at the top of the common parent's view so both see the scope and only
//! one `<style>` tag is emitted. See the crate-level docs for the pattern.

use leptos::prelude::*;

const CSS: &str = include_str!("../style.css");

/// Marker placed in context by whichever component first renders the
/// stylesheet. Descendants that find it in scope skip emitting a duplicate
/// `<style>` tag.
#[derive(Clone, Copy)]
struct StylesheetScope;

/// Render the crate's embedded stylesheet.
///
/// Emits `<style>…</style>` if no ancestor has already provided the scope,
/// and provides the scope for descendants. Safe to call from every
/// component body — it is idempotent per subtree.
pub fn stylesheet() -> impl IntoView {
    if use_context::<StylesheetScope>().is_some() {
        None
    } else {
        provide_context(StylesheetScope);
        Some(view! { <style>{CSS}</style> })
    }
}
