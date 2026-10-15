//! [`ResourceView`], the loading/success/error dispatcher for
//! [`Resource<Result<T, E>>`](leptos::prelude::Resource).

use std::sync::Arc;

use leptos::either::Either;
use leptos::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{ErrorView, UserFacingError};
use icons::outcome::LoadingIcon;

/// Render a [`Resource<Result<T, E>>`](leptos::prelude::Resource) as loading,
/// success, or error.
///
/// On success it calls `adapter(value)` and feeds the result into `view_fn` —
/// this split exists so the underlying view can be a normal `#[component]` with
/// explicit props, while `ResourceView` supplies those props from whatever the
/// resource resolved to.
///
/// On error it renders [`ErrorView`] in banner layout, with a retry that
/// refetches the resource.
///
/// While loading it renders `fallback`, or a spinner if none is given. The
/// loading branch is always wrapped in a [`Suspense`], so re-fetches blank the
/// subtree by default. A future version may make this opt-out.
///
/// # Example
///
/// ```rust,ignore
/// #[component]
/// fn ItemsList(items: Vec<Item>) -> impl IntoView { /* … */ }
///
/// let items: Resource<Result<Vec<Item>, AppError>> =
///     Resource::new(id, fetch_items);
///
/// view! {
///     <ResourceView
///         resource=items
///         view_fn=ItemsList
///         adapter=|v: Vec<Item>| v
///     />
/// }
/// ```
#[component]
pub fn ResourceView<ResourceViewFn, ResourceValue, ViewValue, Props, Adapter, E>(
    /// The resource to render.
    resource: Resource<Result<ResourceValue, E>>,
    /// The component that renders the success value.
    view_fn: ResourceViewFn,
    /// Maps the resource's success value into the props for `view_fn`.
    adapter: Adapter,
    /// Optional loading fallback. Defaults to a spinner.
    #[prop(optional, into)]
    fallback: Option<ViewFn>,
    #[prop(default = false)] suspense: bool,
) -> impl IntoView
where
    ResourceValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    ViewValue: IntoView + Send + 'static,
    ResourceViewFn: Fn(Props) -> ViewValue + Send + Sync + 'static,
    Props: Send + 'static,
    Adapter: Fn(ResourceValue) -> Props + Send + Sync + 'static,
    E: UserFacingError + Serialize + DeserializeOwned,
{
    let view_fn = Arc::new(view_fn);
    let adapter = Arc::new(adapter);

    let core = move || {
        let res = resource.get()?;
        Some(match res {
            Ok(val) => view_fn(adapter(val)).into_any(),
            Err(e) => {
                let err_signal = Signal::derive(move || Some(e.clone()));
                let retry = Callback::new(move |_| resource.refetch());
                view! {
                    <div class="py-8 text-center">
                        <ErrorView
                            error=err_signal
                            on_retry=retry
                            banner=true
                        />
                    </div>
                }
                .into_any()
            }
        })
    };

    let fallback_closure = {
        let fallback = fallback.clone();
        move || match fallback.clone() {
            Some(f) => f.run(),
            None => view! { <LoadingIcon/> }.into_any(),
        }
    };

    if suspense {
        Either::Left(view! {
            <Suspense fallback=fallback_closure>
                {core}
            </Suspense>
        })
    } else {
        Either::Right(view! {
            <Transition fallback=fallback_closure>
                {core}
            </Transition>
        })
    }
}
