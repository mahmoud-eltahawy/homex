//! Icon components for HomeX.
//!
//! Every icon is a plain Leptos component rendering inline SVG, so they
//! participate in hydration correctly and style via `class` like any other
//! element. The crate has no knowledge of the app's domain types — the
//! media-kind glyphs are selected through [`MediaIconKind`].
//!
//! # Module layout
//!
//! | Module    | Contents                                                    |
//! |-----------|-------------------------------------------------------------|
//! | `generic` | The `<svg>` wrapper every icon is built on.                 |
//! | `brand`   | The HomeX brand mark.                                       |
//! | `ui`      | HomeX chrome: search, nav, edit, upload, …                  |
//! | `media`   | The four media-kind glyphs + `MediaIconKind`.               |
//! | `posters` | Full-bleed placeholder artwork for missing posters.         |
//! | `player`  | Transport / volume / fullscreen glyphs for the media player.|
//! | `outcome` | Feedback glyphs for error and loading states.               |
//!
//! `player` and `outcome` deliberately overlap names with `ui` (several
//! glyphs — page arrows, download, edit, delete — are shared), but they
//! carry different `class` attributes because the two consumers size them
//! differently. They are therefore *not* re-exported at the crate root;
//! use them via their module path.

pub mod brand;
pub mod generic;
pub mod media;
pub mod outcome;
pub mod player;
pub mod posters;
pub mod ui;

// ─── Root re-exports ──────────────────────────────────────────────────────────
//
// Only `brand` / `generic` / `media` / `posters` / `ui` are promoted to the
// root. `player` and `outcome` are namespaced because several of their
// components share names with `ui`'s and differ only in class.

pub use brand::MediaCubeLogo;
pub use generic::Icon;
pub use media::{AudioGroupIcon, AudioIcon, MediaIconKind, MovieIcon, SeriesIcon, icon_for};
pub use posters::{MoviePosterSvg, MusicPosterSvg, SeriesPosterSvg};
pub use ui::{
    ClockIcon, DeleteIcon, DownloadIcon, EditIcon, EmptyStateIcon, MenuIcon, NextPageIcon,
    PrevPageIcon, RetryIcon, SearchIcon, UploadIcon, ViewAllIcon, XIcon,
};

/// Everything you normally want in one `use`.
///
/// Covers the HomeX-facing glyphs only — `player` and `outcome` are
/// intentionally excluded so a downstream `use icons::prelude::*;` never
/// pulls in a second `PrevPageIcon` shadowing the first.
pub mod prelude {
    pub use crate::{
        brand::MediaCubeLogo,
        generic::Icon,
        media::{AudioGroupIcon, AudioIcon, MediaIconKind, MovieIcon, SeriesIcon, icon_for},
        posters::{MoviePosterSvg, MusicPosterSvg, SeriesPosterSvg},
        ui::{
            ClockIcon, DeleteIcon, DownloadIcon, EditIcon, EmptyStateIcon, MenuIcon, NextPageIcon,
            PrevPageIcon, RetryIcon, SearchIcon, UploadIcon, ViewAllIcon, XIcon,
        },
    };
}
