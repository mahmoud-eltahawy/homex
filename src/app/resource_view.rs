use std::sync::Arc;

use leptos::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::app::error::{self, Disposition};
use crate::app::icons::{ErrorIcon, LoadingIcon, RetryIcon};

#[component]
pub fn ResourceView<ResourceViewFn, ResourceValue, ViewValue, Props, Adapter>(
    resource: Resource<Result<ResourceValue, ServerFnError>>,
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
                let retryable = matches!(error::disposition(&e), Disposition::Transient);
                let msg = error::user_message(&e);
                view! {
                    <div class="py-8 text-center">
                        <div class="text-red-400 text-sm font-bold mb-2"><ErrorIcon/></div>
                        <p class="text-gray-500 text-xs mb-3">{msg}</p>
                        <Show when=move || retryable>
                            <button
                                on:click=move |_| resource.refetch()
                                class="px-3 py-1 rounded-lg bg-white/10 hover:bg-white/20 \
                                       text-white text-xs transition"
                            >
                                <RetryIcon/>
                            </button>
                        </Show>
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
