use crate::app::icons::DownloadIcon;
use leptos::prelude::*;

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
pub fn HeroMeta(children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
            {children()}
        </div>
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
            <DownloadIcon/> "Download"
        </a>
    }
}
