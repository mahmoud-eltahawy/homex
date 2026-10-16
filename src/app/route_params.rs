//! Route parameter readers.
//!
//! The generic readers live in `leptos_page`. This module re-exports them
//! and adds the one reader that needs a HomeX domain type — `use_slug_param`,
//! which parses a URL segment into a [`Slug`].
//!
//! Every reader returns `Result`. A parse failure is not swallowed; it
//! propagates as a `RouteError` and short-circuits the page to its
//! `render_error`.

use crate::app::ids::Slug;

pub use leptos_page::{RouteError, use_optional_uuid_param, use_uuid_param};

/// Read the named URL path parameter and parse it as a [`Slug`].
///
/// `Slug::new` does the validation — non-empty, lowercase alphanumeric plus
/// hyphens, no leading or trailing hyphen, no `--`. A `SlugError` becomes a
/// `RouteError::InvalidValue` with the reason preserved.
pub fn use_slug_param(name: &'static str) -> Result<Slug, RouteError> {
    leptos_page::use_parsed_param(name)
}
