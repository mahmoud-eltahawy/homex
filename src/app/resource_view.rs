use std::sync::Arc;

use leptos::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::app::error::AppError;
use crate::app::error_view::{ErrorStyle, ErrorView};
use crate::app::icons::LoadingIcon;

#[component]
pub fn ResourceView<ResourceViewFn, ResourceValue, ViewValue, Props, Adapter>(
    resource: Resource<Result<ResourceValue, AppError>>,
    view_fn: ResourceViewFn,
    adapter: Adapter,
    #[prop(optional, into)] fallback: Option<ViewFn>,
) -> impl IntoView
where
    ResourceValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    ViewValue: IntoView + Send + 'static,
    ResourceViewFn: Fn(Props) -> ViewValue + Send + Sync + 'static,
    Props: Send + 'static,
    Adapter: Fn(ResourceValue) -> Props + Send + Sync + 'static,
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
                            style=ErrorStyle::Banner
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

    view! {
        <Suspense fallback=fallback_closure>
            {core}
        </Suspense>
    }
}
