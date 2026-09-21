use crate::app::{
    icons::{AudioIcon, MediaCubeLogo, MenuIcon, MovieIcon, XIcon},
    model::{MediaKind, Section},
    sections::fetch_sections,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component(transparent)]
pub fn Layout() -> impl IntoView {
    view! {
        <div class="flex flex-col min-h-screen bg-[#0a0a0f] text-white font-sans antialiased" dir="rtl">
            <Navbar/>
            <main class="flex-1 bg-gradient-to-b from-[#0a0a0f] via-[#12121a] to-[#0a0a0f] pt-20 md:pt-24 lg:pt-28 pb-8 md:pb-12">
                <Outlet/>
            </main>
            <Footer/>
        </div>
    }
}

fn section_icon(kind: MediaKind) -> Either<impl IntoView, impl IntoView> {
    match kind {
        MediaKind::Video => Either::Left(MovieIcon()),
        MediaKind::Audio => Either::Right(AudioIcon()),
    }
}

#[component]
fn Navbar() -> impl IntoView {
    let mobile_open = RwSignal::new(false);
    view! {
        <nav class="fixed top-0 start-0 end-0 z-50 backdrop-blur-xl bg-black/60 border-b border-white/[0.06] shadow-2xl shadow-black/50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16 md:h-20">
                    <Brand/>
                    <DesktopNavLinks/>
                    <MobileMenuButton open=mobile_open/>
                </div>
            </div>
            <MobileMenu open=mobile_open/>
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
    let sections = Resource::new(|| (), |_| fetch_sections());
    view! {
        <div class="hidden md:flex items-center gap-2">
            <Transition fallback=|| ()>
                {move || sections.get().map(|r| {
                    r.unwrap_or_default().into_iter().map(|s: Section| {
                        let icon = section_icon(s.media_kind);
                        view! { <NavLink href=s.href() label=s.title icon=icon/> }
                    }).collect_view()
                })}
            </Transition>
        </div>
    }
}

#[component]
pub fn NavLink(
    #[prop(into)] href: String,
    icon: impl IntoView + 'static,
    #[prop(into)] label: String,
) -> impl IntoView {
    view! {
        <a href=href
            class="px-3 py-2 rounded-xl text-gray-400 hover:text-white hover:bg-white/10 transition-all duration-300 inline-flex items-center gap-2"
            aria-label="Navigate">
            {icon}
            <span class="text-sm font-medium">{label}</span>
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
    let sections = Resource::new(|| (), |_| fetch_sections());
    view! {
        <div class="flex flex-col sm:flex-row items-center justify-between gap-8 md:gap-12">
            <Brand/>
            <div class="flex items-center gap-6">
                <Transition fallback=|| ()>
                    {move || sections.get().map(|r| {
                        r.unwrap_or_default().into_iter().map(|s: Section| {
                            let icon = section_icon(s.media_kind);
                            view! { <NavLink href=s.href() label=s.title icon=icon/> }
                        }).collect_view()
                    })}
                </Transition>
            </div>
            <div class="flex items-center gap-6">
                <span class="text-gray-500 text-xs font-mono">"v1.0.0"</span>
            </div>
        </div>
    }
}

#[component]
fn MobileMenuButton(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="md:hidden p-2 rounded-xl text-gray-300 hover:text-white hover:bg-white/10 transition"
            on:click=move |_| open.update(|o| *o = !*o)
            aria-label=move || if open.get() { "إغلاق القائمة" } else { "فتح القائمة" }
            aria-expanded=move || if open.get() { "true" } else { "false" }
        >
            {move || if open.get() { Either::Left(XIcon()) } else { Either::Right(MenuIcon()) }}
        </button>
    }
}

#[component]
fn MobileMenu(open: RwSignal<bool>) -> impl IntoView {
    let sections = Resource::new(|| (), |_| fetch_sections());
    view! {
        <Show when=move || open.get()>
            <div class="md:hidden fixed inset-0 top-16 z-40 bg-black/85 backdrop-blur-xl"
                on:click=move |_| open.set(false)>
                <div class="flex flex-col p-4 gap-1" on:click=|ev| ev.stop_propagation()>
                    <Transition fallback=|| ()>
                        {move || sections.get().map(|r| {
                            r.unwrap_or_default().into_iter().map(|s: Section| {
                                let icon = section_icon(s.media_kind);
                                view! {
                                    <MobileMenuLink href=s.href() icon=icon label=s.title open=open/>
                                }
                            }).collect_view()
                        })}
                    </Transition>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn MobileMenuLink(
    #[prop(into)] href: String,
    icon: impl IntoView + 'static,
    #[prop(into)] label: String,
    open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <a href=href on:click=move |_| open.set(false)
            class="flex items-center gap-3 px-4 py-3 rounded-xl text-gray-300 hover:text-white hover:bg-white/10 transition">
            {icon}
            <span class="font-medium text-base">{label}</span>
        </a>
    }
}
