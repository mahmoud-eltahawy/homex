use crate::app::{
    icons::DownloadIcon,
    inline_edit::{EditModeToggle, use_edit_mode},
};
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
pub fn DetailShell(
    #[prop(into)] poster: Option<String>,
    #[prop(default = true)] editable: bool,
    children: Children,
) -> impl IntoView {
    let edit_on = use_edit_mode();

    view! {
        <div class="relative min-h-screen bg-black text-white overflow-hidden">
            <div class="absolute inset-0">
                {poster.map(|src| view! {
                    <img src=src class="w-full h-full object-cover opacity-40" alt=""/>
                })}
                <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
            </div>
            <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                <Show when=move || editable>
                    <EditModeToggle
                        edit_on=edit_on
                        wrap_class="absolute top-4 end-4 md:top-6 md:end-6 z-20".to_string()
                    />
                </Show>
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn DetailHero(poster: impl IntoView + 'static, children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
                {poster}
            </div>
            <div class="flex-1 w-full">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn HeroBadge(#[prop(into)] label: String, icon: impl IntoView + 'static) -> impl IntoView {
    view! {
        <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
            {icon}
            {label}
        </div>
    }
}

#[component]
pub fn HeroTitle(#[prop(into)] title: String) -> impl IntoView {
    view! {
        <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">
            {title}
        </h1>
    }
}

#[component]
pub fn HeroMeta(children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
            {children()}
        </div>
    }
}

#[component]
pub fn HeroDescription(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">
            {text}
        </p>
    }
}

#[component]
pub fn DownloadButton(
    #[prop(into)] href: String,
    #[prop(into)] download_name: String,
) -> impl IntoView {
    view! {
        <a
            href=href
            download=download_name
            class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm"
        >
            <DownloadIcon/> "تحميل"
        </a>
    }
}
