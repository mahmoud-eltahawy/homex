//! Re-exports the outcome glyphs from the shared `icons` crate.
//!
//! These used to be defined here. They now live in `icons::outcome`.
//! The leading `::` distinguishes the external crate from this module,
//! which is itself called `icons`.

pub(crate) use ::icons::outcome::{ErrorIcon, LoadingIcon};
