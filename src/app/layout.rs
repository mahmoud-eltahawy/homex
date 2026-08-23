use crate::app::layout::search::SearchBox;
use leptos::prelude::*;
use leptos_router::components::Outlet;

mod search;

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
    let search_term = RwSignal::new(String::new());
    let search_open = RwSignal::new(false);
    view! {
        <nav class="fixed top-0 start-0 end-0 z-50 backdrop-blur-xl bg-black/60 border-b border-white/[0.06] shadow-2xl shadow-black/50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16 md:h-20">
                    <NavbarBrand/>
                    <DesktopNavLinks search_term=search_term search_open=search_open/>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn NavbarBrand() -> impl IntoView {
    view! {
        <a
            href="/".to_string()
            class="flex items-center gap-2 text-2xl sm:text-3xl md:text-4xl font-black tracking-tighter"
        >
            <span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
        </a>
    }
}

#[component]
fn DesktopNavLinks(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="hidden md:flex items-center gap-2">
            <SearchBox search_term=search_term search_open=search_open/>
            <NavLink href="/movie" label="أفلام"/>
            <NavLink href="/series" label="مسلسلات"/>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a
            href=href.to_string()
            class="px-4 py-2 rounded-2xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-300 backdrop-blur-sm"
        >
            {label}
        </a>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-[#0a0a0f]/90 backdrop-blur-xl border-t border-white/5 mt-auto">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 md:py-16">
                <FooterGrid/>
                <FooterCopyright/>
            </div>
        </footer>
    }
}

#[component]
fn FooterGrid() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-8 md:gap-12">
            <FooterBrand/>
            <FooterLinks/>
            <FooterLibrary/>
        </div>
    }
}

#[component]
fn FooterBrand() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <a
                href="/".to_string()
                class="text-2xl font-black tracking-tighter"
            >
                <span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
            </a>
            <p
                class="text-gray-400 text-sm max-w-xs leading-relaxed"
            >
                "خادم السينما الشخصي الخاص بك — شاهد، حمّل، واستمتع بمجموعتك في أي وقت."
            </p>
        </div>
    }
}

#[component]
fn FooterLinks() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-white font-semibold text-sm mb-4 tracking-wide">تصفح</h3>
            <ul class="space-y-2 text-sm">
                <li><NavLink href="/movies" label="أفلام"/></li>
                <li><NavLink href="/series" label="مسلسلات"/></li>
            </ul>
        </div>
    }
}

#[component]
fn FooterLibrary() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-white font-semibold text-sm mb-4 tracking-wide">المكتبة</h3>
            <ul class="space-y-2 text-sm">
                <li><NavLink href="/upload" label="رفع وسائط"/></li>
                <li><NavLink href="/settings" label="الإعدادات"/></li>
                <li><span class="text-gray-500 cursor-default">v1.0.0</span></li>
            </ul>
        </div>
    }
}

#[component]
fn FooterCopyright() -> impl IntoView {
    view! {
        <div class="mt-10 pt-6 border-t border-white/5 text-center text-gray-500 text-xs tracking-wide">
            <p>"© 2025 وسائطي. صُنع بكل ❤️ لشبكتك المنزلية."</p>
        </div>
    }
}
