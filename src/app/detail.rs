use leptos::{either::Either, prelude::*};

#[component]
pub fn Poster(
    src: Option<String>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional)] alt: Option<String>,
    #[prop(optional)] placeholder: Option<AnyView>,
) -> impl IntoView {
    let class =
        class.unwrap_or_else(|| "w-full rounded-2xl shadow-2xl border border-white/10".into());
    view! {
        {match src {
            Some(src) => Either::Left(view! {
                <img src=src class=class alt=alt.unwrap_or_default()/>
            }),
            None => Either::Right(placeholder.unwrap_or_else(|| view! {
                <div class=format!("{class} aspect-[2/3] bg-white/5")></div>
            }.into_any())),
        }}
    }
}

#[component]
pub fn DetailShell(#[prop(into)] poster: Option<String>, children: Children) -> impl IntoView {
    view! {
        <div class="relative min-h-screen bg-black text-white overflow-hidden">
            <div class="absolute inset-0">
                {poster.map(|src| view! {
                    <img src=src class="w-full h-full object-cover opacity-40" alt=""/>
                })}
                <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
            </div>
            <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                {children()}
            </div>
        </div>
    }
}
