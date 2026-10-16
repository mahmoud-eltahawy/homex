use leptos::prelude::*;
use std::convert::Infallible;

use crate::app::page::{InfalliblePage, PageContent};

pub struct ErrorContent;

impl PageContent<Infallible> for ErrorContent {
    fn load() -> Result<Self, Infallible> {
        Ok(Self)
    }

    fn render(self) -> impl IntoView {
        view! {
            <div class="min-h-[70vh] flex items-center justify-center px-4">
                <div class="max-w-md text-center space-y-4">
                    <div class="text-6xl font-black text-cyan-400">"404"</div>
                    <h1 class="text-xl font-bold text-white">"Page not found"</h1>
                    <p class="text-gray-400 text-sm">
                        "The URL you tried to open is malformed."
                    </p>
                    <a
                        href="/"
                        class="inline-block px-4 py-2 rounded-lg \
                               bg-gradient-to-r from-cyan-500 to-blue-500 \
                               hover:from-cyan-400 hover:to-blue-400 \
                               text-white font-bold text-sm transition"
                    >
                        "Go home"
                    </a>
                </div>
            </div>
        }
    }
}

pub type ErrorPage = InfalliblePage<ErrorContent>;
