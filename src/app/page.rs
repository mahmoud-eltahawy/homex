//! A page whose URL parameters are parsed eagerly.
//!
//! Every routed page parses its path parameters in `data()`, before any
//! resource is constructed. If a parameter fails to parse, the page
//! short-circuits to a redirect; otherwise, the loaded body is rendered.
//!
//! [`Page<T, E>`] carries that policy. `T` supplies the content; `E` is the
//! type `load()` returns on failure. See [`PageContent`].

use leptos::prelude::*;
use leptos_router::LazyRoute;

use crate::app::common::RedirectToError;

/// A page render decision.
///
/// * `Good(T)` — parameters parsed, body loaded, render it.
/// * `Bad(E)` — a parameter failed to parse, redirect to `/error`.
pub enum Page<T, E> {
    Good(T),
    Bad(E),
}

/// The content of a page: how to load it, and how to render it.
///
/// `load()` runs in `data()` inside the route's reactive owner. It is the
/// only place a URL parameter is turned into a domain type. It returns
/// `Err` the moment any parameter fails validation; no resource is
/// constructed in that case.
///
/// `render()` runs in `view()`, only ever called on a body that loaded
/// successfully.
pub trait PageContent<E>: Sized + Send + Sync + 'static {
    fn load() -> Result<Self, E>;
    fn render(self) -> AnyView;
}

impl<T, E> Page<T, E>
where
    T: PageContent<E>,
{
    pub fn load() -> Self {
        match T::load() {
            Ok(body) => Page::Good(body),
            Err(e) => Page::Bad(e),
        }
    }

    pub fn into_view(self) -> AnyView {
        match self {
            Page::Good(body) => body.render(),
            Page::Bad(_) => view! { <RedirectToError/> }.into_any(),
        }
    }
}

// ─── LazyRoute plumbing ───────────────────────────────────────────────────
//
// `LazyRoute` requires `Self: Send + Sync + 'static`, and `Self` here is
// `Page<T, E>`. `T` is already covered by `PageContent`. `E` needs the
// same bounds spelled out because nothing else constrains it.

impl<T, E> LazyRoute for Page<T, E>
where
    T: PageContent<E>,
    E: Send + Sync + 'static,
{
    fn data() -> Self {
        Page::load()
    }
    async fn view(this: Self) -> AnyView {
        this.into_view()
    }
}
