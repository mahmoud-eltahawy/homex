use crate::app::{
    icons::{AudioIcon, MediaCubeLogo, MovieIcon, SeriesIcon, SettingsIcon, UploadIcon},
    model::MediaType,
};
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <div
            class="flex flex-col min-h-screen bg-[#0a0a0f] text-white font-sans antialiased"
            dir="rtl"
        >
            <Navbar/>
            <main
                class="flex-1 bg-gradient-to-b from-[#0a0a0f] via-[#12121a] to-[#0a0a0f] pt-20 md:pt-24 lg:pt-28 pb-8 md:pb-12"
            >
                <Outlet/>
            </main>
            <Footer/>
        </div>
    }
}

#[component]
fn Navbar() -> impl IntoView {
    view! {
        <nav class="fixed top-0 start-0 end-0 z-50 backdrop-blur-xl bg-black/60 border-b border-white/[0.06] shadow-2xl shadow-black/50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16 md:h-20">
                    <Brand/>
                    <DesktopNavLinks/>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn Brand() -> impl IntoView {
    view! {
        <a href="/".to_string() class="flex items-center">
            <MediaCubeLogo />
        </a>
    }
}

#[component]
fn DesktopNavLinks() -> impl IntoView {
    view! {
        <div class="hidden md:flex items-center gap-2">
            <NavLink href=MediaType::Movie.listing_href() icon=MovieIcon />
            <NavLink href=MediaType::Series.listing_href() icon=SeriesIcon />
            <NavLink href=MediaType::AudioGroup.listing_href() icon=AudioIcon />
        </div>
    }
}

#[component]
pub fn NavLink(href: String, icon: impl IntoView) -> impl IntoView {
    view! {
        <a
            href=href
            class="p-2 rounded-xl text-gray-400 hover:text-white hover:bg-white/10 transition-all duration-300"
            aria-label="Navigate"
        >
            {icon}
        </a>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-[#0a0a0f]/90 backdrop-blur-xl border-t border-white/5 mt-auto">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 md:py-16">
                <FooterGrid/>
            </div>
        </footer>
    }
}

#[component]
fn FooterGrid() -> impl IntoView {
    view! {
        <div class="flex flex-col sm:flex-row items-center justify-between gap-8 md:gap-12">
            <Brand/>
            <div class="flex items-center gap-6">
                <NavLink href=MediaType::Movie.listing_href() icon={MovieIcon()} />
                <NavLink href=MediaType::Series.listing_href() icon={SeriesIcon()} />
                <NavLink href=MediaType::AudioGroup.listing_href() icon={AudioIcon()} />
            </div>
            <div class="flex items-center gap-6">
                <NavLink href="/upload".to_string() icon={UploadIcon()} />
                <NavLink href="/settings".to_string() icon={SettingsIcon()} />
                <span class="text-gray-500 text-xs font-mono">v1.0.0</span>
            </div>
        </div>
    }
}
