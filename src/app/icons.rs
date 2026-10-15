//! Thin adapter over `homex_icons`.
//!
//! Keeps the `crate::app::icons::*` path stable for existing call sites
//! while delegating all rendering to the extracted crate.

pub use icons::*;

use leptos::prelude::*;

use crate::app::model::MediaKind;

/// Map HomeX's domain `MediaKind` onto the icon crate's enum and render.
///
/// This is the only piece that knows both types; the icon crate itself
/// stays domain-agnostic.
pub fn icon_for(kind: MediaKind, nested: bool) -> AnyView {
    icons::icon_for(icons::MediaIconKind::from_parts(
        matches!(kind, MediaKind::Video),
        nested,
    ))
}
