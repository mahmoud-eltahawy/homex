//! Page-as-a-value routing for [Leptos](https://leptos.dev).
//!
//! The pattern this crate packages: a routed page is one type that knows how
//! to *load* itself from the URL and how to *render* itself. Loading runs
//! before any reactive resource is constructed, so a malformed URL fails
//! once, at the boundary — never a silent `unwrap_or_default()` deep inside
//! a component tree.
//!
//! # The three pieces
//!
//! * [`PageContent`] — the trait a page body implements. `load()` parses the
//!   URL and builds the reactive state; `render()` consumes `self` and
//!   produces the view.
//! * [`Page<T, E>`] — the routing wrapper. `Page::<T, E>::load()` calls
//!   `T::load()` and packages the result; [`LazyRoute`] is implemented for
//!   it, so the router only ever sees one type.
//! * [`RouteError`] — the default `E`. Returned by every parameter reader in
//!   this crate when the URL segment is missing or fails to parse.
//!
//! [`InfalliblePage<T>`] is `Page<T, Infallible>`, for pages with no fallible
//! parameters.
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_page::{Page, PageContent, RouteError, use_uuid_param};
//! use uuid::Uuid;
//!
//! struct UserContent {
//!     id: Uuid,
//!     user: Resource<Result<User, AppError>>,
//! }
//!
//! impl PageContent<RouteError> for UserContent {
//!     fn load() -> Result<Self, RouteError> {
//!         let id = use_uuid_param("id")?;
//!         let user = Resource::new(move || id, |id| fetch_user(id));
//!         Ok(Self { id, user })
//!     }
//!
//!     fn render(self) -> impl IntoView {
//!         view! { /* loading / error / success */ }
//!     }
//! }
//!
//! pub type UserPage = Page<UserContent, RouteError>;
//!
//! // In the router:
//! // <Route path=path!("/users/:id") view={Lazy::<UserPage>::new()}/>
//! ```
//!
//! # Failure handling
//!
//! A page's `load()` can fail — a `:id` segment that isn't a UUID, a slug
//! that doesn't validate. When that happens the router renders
//! [`PageContent::render_error`] instead of the page body.
//!
//! The default implementation logs the error and redirects to `/error`,
//! carrying the error's `Display` output in a `reason` query parameter. The
//! [`ErrorPage`] component at that route reads it back and shows it. Every
//! failure is observed — even the default doesn't discard the value.
//!
//! [`LazyRoute`]: leptos_router::LazyRoute

mod error_page;
mod page;
mod route;

pub use error_page::ErrorPage;
pub use page::{InfalliblePage, Page, PageContent};
pub use route::{
    RouteError, use_optional_uuid_param, use_parsed_param, use_string_param, use_uuid_param,
};
