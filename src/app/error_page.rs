//! The `/error` route.
//!
//! The card itself lives in `leptos_page::ErrorPage`; this module wires it
//! into the app's router as an infallible page.

use std::convert::Infallible;

use leptos::prelude::*;
use leptos_page::{InfalliblePage, PageContent};

pub struct ErrorContent;

impl PageContent<Infallible> for ErrorContent {
    fn load() -> Result<Self, Infallible> {
        Ok(Self)
    }

    fn render(self) -> impl IntoView {
        view! { <leptos_page::ErrorPage/> }
    }
}

pub type ErrorPage = InfalliblePage<ErrorContent>;
