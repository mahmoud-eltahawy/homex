//! Icon components for HomeX.
//!
//! Every icon is a plain Leptos component rendering inline SVG, so they
//! participate in hydration correctly and style via `class` like any other
//! element. The crate has no knowledge of the app's domain types — the
//! media-kind glyphs are selected through [`MediaIconKind`].

pub mod brand;
pub mod generic;
pub mod media;
pub mod posters;
pub mod ui;

pub use brand::MediaCubeLogo;
pub use generic::Icon;
pub use media::{AudioGroupIcon, AudioIcon, MediaIconKind, MovieIcon, SeriesIcon, icon_for};
pub use posters::{MoviePosterSvg, MusicPosterSvg, SeriesPosterSvg};
pub use ui::{
    ClockIcon, DeleteIcon, DownloadIcon, EditIcon, EmptyStateIcon, MenuIcon, NextPageIcon,
    PrevPageIcon, RetryIcon, SearchIcon, UploadIcon, ViewAllIcon, XIcon,
};

/// Everything you normally want in one `use`.
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
