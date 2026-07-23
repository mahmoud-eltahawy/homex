use crate::app::AppLink;
use crate::app::SearchIcon;

use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_navigate;

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
                <NavbarTop/>
                <MobileNav/>
            </div>
        </nav>
    }
}

#[component]
fn NavbarTop() -> impl IntoView {
    let search_term = RwSignal::new(String::new());
    let search_open = RwSignal::new(false);
    view! {
        <div class="flex items-center justify-between h-16 md:h-20">
            <NavbarBrand/>
            <DesktopNavLinks search_term=search_term search_open=search_open/>
            <MobileSearch search_term=search_term/>
        </div>
    }
}

#[component]
fn NavbarBrand() -> impl IntoView {
    view! {
        <AppLink
            href="/"
            class="flex items-center gap-2 text-2xl sm:text-3xl md:text-4xl font-black tracking-tighter"
        >
            <span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
        </AppLink>
    }
}

#[component]
fn DesktopNavLinks(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="hidden md:flex items-center gap-2">
            <NavLink href="/movies" label="أفلام"/>
            <NavLink href="/series" label="مسلسلات"/>
            <SearchBox search_term=search_term search_open=search_open/>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <AppLink
            href=href
            class="px-4 py-2 rounded-2xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-300 backdrop-blur-sm"
        >
            {label}
        </AppLink>
    }
}

#[component]
fn SearchBox(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    let navigate = use_navigate();
    let on_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let term = search_term.get().trim().to_string();
        if !term.is_empty() {
            navigate(
                &format!("/search?q={}", encode_uri_component(&term)),
                Default::default(),
            );
            search_open.set(false);
        }
    };
    let class = move || {
        format!(
            "relative me-2 transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] {}",
            if search_open.get() { "w-64" } else { "w-10" }
        )
    };
    view! {
        <div class=class>
            <form on:submit=on_search class="flex items-center">
                <SearchToggle search_open=search_open/>
                <SearchInput search_term=search_term search_open=search_open/>
            </form>
        </div>
    }
}

#[component]
fn SearchToggle(search_open: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| search_open.set(!search_open.get());
    view! {
        <button type="button" on:click=on_click
            class="absolute start-1 top-1/2 -translate-y-1/2 p-1.5 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors">
            <SearchIcon/>
        </button>
    }
}

#[component]
fn SearchInput(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    let class = move || {
        format!("w-full bg-white/5 backdrop-blur-xl text-white placeholder-gray-500 rounded-full py-2.5 pe-4 ps-12 text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/10 transition-all duration-300 {}",
            if search_open.get() { "opacity-100 scale-100" } else { "opacity-0 scale-95 pointer-events-none" })
    };
    view! {
        <input type="text"
            prop:value=search_term
            on:input=move |ev| search_term.set(event_target_value(&ev))
            on:focus=move |_| search_open.set(true)
            on:blur=move |_| search_open.set(!search_term.get().is_empty())
            placeholder="ابحث..."
            class=class
        />
    }
}

#[component]
fn MobileSearch(search_term: RwSignal<String>) -> impl IntoView {
    let navigate = use_navigate();
    let on_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let term = search_term.get().trim().to_string();
        if !term.is_empty() {
            navigate(
                &format!("/search?q={}", encode_uri_component(&term)),
                Default::default(),
            );
        }
    };
    let on_input = move |ev| search_term.set(event_target_value(&ev));
    view! {
        <div class="md:hidden flex items-center gap-2">
            <form on:submit=on_search class="relative flex items-center">
                <input type="text"
                    prop:value=search_term
                    on:input=on_input
                    placeholder="ابحث..."
                    class="w-28 sm:w-36 bg-white/10 backdrop-blur-xl text-white placeholder-gray-400 rounded-full py-1.5 pe-3 ps-3 text-xs focus:outline-none focus:ring-1 focus:ring-cyan-400/50"
                />
                <button type="submit" class="absolute start-1.5 top-1/2 -translate-y-1/2 text-gray-400">
                    <SearchIcon/>
                </button>
            </form>
        </div>
    }
}

#[component]
fn MobileNav() -> impl IntoView {
    view! {
        <div class="md:hidden flex gap-1 pb-2">
            <NavLink href="/movies" label="أفلام"/>
            <NavLink href="/series" label="مسلسلات"/>
        </div>
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
            <AppLink
                href="/"
                class="text-2xl font-black tracking-tighter"
            >
                <span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
            </AppLink>
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
                <li><NavLink href="/search" label="بحث"/></li>
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

// ── PERCENT ENCODING ──────────────────────────────────────────────────────
fn encode_uri_component(s: &str) -> String {
    s.chars().fold(String::new(), |mut acc, c| {
        match c {
            'A'..='Z'
            | 'a'..='z'
            | '0'..='9'
            | '-'
            | '_'
            | '.'
            | '!'
            | '~'
            | '*'
            | '\''
            | '('
            | ')' => acc.push(c),
            _ => {
                for b in c.to_string().into_bytes() {
                    acc.push_str(&format!("%{:02X}", b));
                }
            }
        }
        acc
    })
}
