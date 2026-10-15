use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use crate::app::ids::{Slug, SlugError};

pub fn use_uuid_param(name: &'static str) -> impl Fn() -> Uuid + Copy + 'static {
    let params = use_params_map();
    move || {
        params.with(|p| {
            p.get(name)
                .and_then(|s| Uuid::parse_str(&s).ok())
                .unwrap_or(Uuid::nil())
        })
    }
}

pub fn use_string_param(name: &'static str) -> impl Fn() -> String + Copy + 'static {
    let params = use_params_map();
    move || params.with(|p| p.get(name).unwrap_or_default())
}

pub fn use_optional_uuid_param(name: &'static str) -> impl Fn() -> Option<Uuid> + Copy + 'static {
    let params = use_params_map();
    move || params.with(|p| p.get(name).and_then(|s| Uuid::parse_str(&s).ok()))
}

/// Read the named URL path parameter and parse it as a [`Slug`] **now**.
///
/// This is the only place a URL string becomes a domain `Slug`. Called from
/// a page's [`PageContent::load`](crate::app::page::PageContent::load),
/// which runs in `data()` before any resource is constructed.
pub fn use_slug_param(name: &'static str) -> Result<Slug, SlugError> {
    Slug::new(use_string_param(name)())
}
