use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

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
