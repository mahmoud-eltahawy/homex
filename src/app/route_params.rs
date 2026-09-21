use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

pub fn use_u64_param(name: &'static str) -> impl Fn() -> u64 + Copy + 'static {
    let params = use_params_map();
    move || params.with(|p| p.get(name).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0))
}

pub fn use_string_param(name: &'static str) -> impl Fn() -> String + Copy + 'static {
    let params = use_params_map();
    move || params.with(|p| p.get(name).unwrap_or_default())
}
