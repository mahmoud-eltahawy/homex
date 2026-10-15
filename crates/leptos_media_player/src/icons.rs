//! Re-exports the media-player glyphs from the shared `icons` crate.
//!
//! The crate's own SVG bodies used to live here. They are now part of
//! `icons::player`, so styling (and any future tweak to a glyph) is a
//! single-crate concern. The leading `::` distinguishes the external
//! crate from this module, which is itself called `icons`.

pub(crate) use ::icons::player::{
    DeleteIcon, DownloadIcon, EditIcon, FullscreenExitIcon, FullscreenIcon, MuteIcon, NextPageIcon,
    PauseIcon, PlayIcon, PrevPageIcon, VolumeIcon,
};
