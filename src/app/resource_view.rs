use leptos::either::Either;
use leptos::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[component]
pub fn ResourceView<ResourceViewFn, ResourceValue, ViewValue, Props, Adapter>(
    resource: Resource<Result<ResourceValue, ServerFnError>>,
    view_fn: ResourceViewFn,
    adapter: Adapter,
    context: &'static str,
    #[prop(optional, into)] fallback: Option<ViewFn>,
) -> impl IntoView
where
    ResourceValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    ViewValue: IntoView + Send + 'static,
    ResourceViewFn: Send + Sync + 'static + Fn(Props) -> ViewValue,
    Props: Send + Sync + 'static,
    Adapter: Fn(ResourceValue) -> Props + Send + 'static,
{
    let fallback = {
        let fallback = fallback.clone();
        move || match fallback.clone() {
            Some(f) => Either::Left(f.run()),
            None => Either::Right(Fallback(FallbackProps { context })),
        }
    };

    let core = {
        let fallback = fallback.clone();
        move || match resource.get() {
            None => Either::Left(Either::Right(fallback())),
            Some(Err(e)) => Either::Left(Either::Left(ServerFnErrorView(ServerFnErrorViewProps {
                e,
                refetch: move || resource.refetch(),
                context,
            }))),
            Some(Ok(val)) => Either::Right(view_fn(adapter(val))),
        }
    };

    view! {
        <Suspense fallback=fallback>
            {core}
        </Suspense>
    }
}

#[component]
fn ServerFnErrorView<F>(e: ServerFnError, refetch: F, context: &'static str) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <div class="py-8 text-center">
            <div class="text-red-400 text-sm font-bold mb-2">"حدث خطأ اثناء " {context}</div>
            <p class="text-gray-500 text-xs mb-3">{e.to_string()}</p>
            <button
                on:click=move |_| refetch()
                class="px-3 py-1 rounded-lg bg-white/10 hover:bg-white/20 text-white text-xs transition"
            >
                "إعادة المحاولة"
            </button>
        </div>
    }
}

#[component]
fn Fallback(context: &'static str) -> impl IntoView {
    view! { <p class="text-gray-400">"جارٍ ..." {context}</p> };
}
