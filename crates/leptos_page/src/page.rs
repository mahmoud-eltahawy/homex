//! The [`Page`] / [`PageContent`] / [`InfalliblePage`] machinery.
//!
//! `Page<T, E>` is the only type the router ever sees. It implements
//! [`LazyRoute`], so every routed page is spelled `Lazy::<Page<T, E>>::new()`
//! — the router never needs to know about `T` or `E`.

use std::fmt::Display;

use leptos::{either::Either, prelude::*};
use leptos_router::{LazyRoute, components::Redirect};

/// A routed page: a body, or the failure of that body's `load()`.
pub enum Page<T, E> {
    Good(T),
    Bad(E),
}

/// A page whose `load()` cannot fail.
///
/// Same shape as [`Page<T, E>`], but with [`std::convert::Infallible`] as the
/// error type — the `Bad` variant is structurally unreachable.
pub type InfalliblePage<T> = Page<T, std::convert::Infallible>;

/// A page body.
///
/// `load()` runs in [`LazyRoute::data`], inside the route's reactive owner
/// and before any resource is constructed. It is the only place a URL
/// parameter is turned into a domain type. It returns `Err` the moment any
/// parameter fails validation; no resource is built in that case.
///
/// `render()` runs in [`LazyRoute::view`], only ever called on a body that
/// loaded successfully.
///
/// `render_error()` is called in place of `render()` when `load()` fails.
pub trait PageContent<E>: Sized + Send + Sync + 'static
where
    E: Display + Send + Sync + 'static,
{
    fn load() -> Result<Self, E>;
    fn render(self) -> impl IntoView;

    /// Rendered in place of the page when `load()` returns `Err`.
    ///
    /// The default logs the error and redirects to `/error`, carrying the
    /// error's `Display` output in a `reason` query parameter. The
    /// [`ErrorPage`](crate::ErrorPage) at that route reads it back.
    ///
    /// Override to do anything else with the value: render the failure in
    /// place, attach a different status, report to an error tracker, or
    /// route somewhere else entirely.
    fn render_error(err: E) -> impl IntoView {
        leptos::logging::warn!("[page] load failed: {err}");
        let reason = urlencode(&err.to_string());
        view! { <Redirect path=format!("/error?reason={reason}")/> }
    }
}

impl<T, E: std::fmt::Display + Send + Sync + 'static> Page<T, E>
where
    T: PageContent<E>,
{
    /// Run `T::load()`, packaging the result as a `Page`.
    pub fn load() -> Self {
        match T::load() {
            Ok(body) => Page::Good(body),
            Err(e) => Page::Bad(e),
        }
    }

    /// Consume `self`, producing the view the router will render.
    pub fn into_view(self) -> impl IntoView {
        match self {
            Page::Good(body) => Either::Left(body.render()),
            Page::Bad(e) => Either::Right(T::render_error(e)),
        }
    }
}

impl<T, E: std::fmt::Display> LazyRoute for Page<T, E>
where
    T: PageContent<E>,
    E: Send + Sync + 'static,
{
    fn data() -> Self {
        Page::load()
    }

    async fn view(this: Self) -> AnyView {
        // The only `.into_any()` in the whole pattern. `LazyRoute::view` is
        // forced by the framework to return `AnyView`; everything above this
        // line stays concrete.
        this.into_view().into_any()
    }
}

/// Percent-encode a string for use as a query-string value. Leaves the RFC
/// 3986 unreserved set (`A–Z a–z 0–9 - _ . ~`) alone; everything else
/// becomes `%XX`.
fn urlencode(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}
