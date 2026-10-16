//! A page whose URL parameters are parsed eagerly.
//!
//! Every routed page parses its path parameters in `data()`, before any
//! resource is constructed. If a parameter fails to parse — missing, bad
//! UUID, bad slug — the page short-circuits to a redirect; otherwise the
//! loaded body is rendered.
//!
//! [`Page<T, E>`] carries that policy. `T` supplies the content; `E` is the
//! type `load()` returns on failure.
//!
//! * Pages that read path parameters use [`RouteError`] as `E`. Every such
//!   page is a `Page<T, RouteError>`.
//! * Pages whose `load()` cannot fail use [`InfalliblePage<T>`], which is
//!   `Page<T, std::convert::Infallible>`. The `Bad` variant is unreachable
//!   and `load()` always produces `Good`.

use leptos::{either::Either, prelude::*};
use leptos_router::{LazyRoute, components::Redirect};

pub enum Page<T, E> {
    Good(T),
    Bad(E),
}

/// A page whose `load()` cannot fail.
///
/// Same shape as [`Page<T, RouteError>`], but with
/// [`std::convert::Infallible`] as the error type — the `Bad` variant is
/// structurally unreachable.
pub type InfalliblePage<T> = Page<T, std::convert::Infallible>;

pub trait PageContent<E>: Sized + Send + Sync + 'static {
    fn load() -> Result<Self, E>;
    fn render(self) -> impl IntoView;
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

    pub fn into_view(self) -> impl IntoView {
        match self {
            Page::Good(body) => Either::Left(body.render()),
            Page::Bad(_) => Either::Right(view! { <Redirect path="/error"/> }),
        }
    }
}

impl<T, E> LazyRoute for Page<T, E>
where
    T: PageContent<E>,
    E: Send + Sync + 'static,
{
    fn data() -> Self {
        Page::load()
    }
    async fn view(this: Self) -> AnyView {
        this.into_view().into_any()
    }
}
