use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    hooks::{use_navigate, use_params_map, use_query_map},
    path,
};
use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, MouseEvent};

// ---------- MOCK DATA ----------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Episode {
    id: i64,
    season: u32,
    episode: u32,
    title: String,
    file_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum MediaType {
    Movie,
    Series,
}

impl TryFrom<&str> for MediaType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let res = match value.to_lowercase().as_str() {
            "movie" => Self::Movie,
            "series" => Self::Series,
            _ => return Err("Media type either movie or series only"),
        };
        Ok(res)
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            MediaType::Movie => "movie",
            MediaType::Series => "series",
        };
        write!(f, "{}", res)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Media {
    id: i64,
    title: String,
    media_type: MediaType,
    poster: String,
    file_path: String,
    size: String,
    description: Option<String>,
    year: Option<u32>,
    duration: Option<String>,
    episodes: Vec<Episode>,
}

const TEST_VIDEO: &str = "https://www.w3schools.com/html/mov_bbb.mp4";

fn mock_movies() -> Vec<Media> {
    vec![
        Media {
            id: 1,
            title: "Inception".into(),
            media_type: MediaType::Movie,
            poster: "https://picsum.photos/seed/inception/300/450".into(),
            file_path: TEST_VIDEO.into(),
            size: "2.1 جيجابايت".into(),
            description: Some("لص يسرق أسرار الشركات من خلال تقنية مشاركة الأحلام.".into()),
            year: Some(2010),
            duration: Some("ساعتان و28 دقيقة".into()),
            episodes: vec![],
        },
        Media {
            id: 2,
            title: "The Matrix".into(),
            media_type: MediaType::Movie,
            poster: "https://picsum.photos/seed/matrix/300/450".into(),
            file_path: TEST_VIDEO.into(),
            size: "1.8 جيجابايت".into(),
            description: Some("هاكر كمبيوتر يكتشف حقيقة الواقع.".into()),
            year: Some(1999),
            duration: Some("ساعتان و16 دقيقة".into()),
            episodes: vec![],
        },
        Media {
            id: 3,
            title: "Interstellar".into(),
            media_type: MediaType::Movie,
            poster: "https://picsum.photos/seed/interstellar/300/450".into(),
            file_path: TEST_VIDEO.into(),
            size: "3.1 جيجابايت".into(),
            description: Some("فريق من المستكشفين يسافرون عبر ثقب دودي في الفضاء.".into()),
            year: Some(2014),
            duration: Some("ساعتان و49 دقيقة".into()),
            episodes: vec![],
        },
        Media {
            id: 4,
            title: "The Dark Knight".into(),
            media_type: MediaType::Movie,
            poster: "https://picsum.photos/seed/darkknight/300/450".into(),
            file_path: TEST_VIDEO.into(),
            size: "2.5 جيجابايت".into(),
            description: Some("عندما يهدد الجوكر مدينة غوثام بالدمار.".into()),
            year: Some(2008),
            duration: Some("ساعتان و32 دقيقة".into()),
            episodes: vec![],
        },
        Media {
            id: 5,
            title: "Pulp Fiction".into(),
            media_type: MediaType::Movie,
            poster: "https://picsum.photos/seed/pulpfiction/300/450".into(),
            file_path: TEST_VIDEO.into(),
            size: "1.9 جيجابايت".into(),
            description: Some("تتشابك حياة اثنين من القتلة وملاكم وزوجين من اللصوص.".into()),
            year: Some(1994),
            duration: Some("ساعتان و34 دقيقة".into()),
            episodes: vec![],
        },
    ]
}

fn mock_series() -> Vec<Media> {
    vec![
        Media {
            id: 101,
            title: "Breaking Bad".into(),
            media_type: MediaType::Series,
            poster: "https://picsum.photos/seed/breakingbad/300/450".into(),
            file_path: "/media/series/breakingbad/".into(),
            size: "45 جيجابايت (5 مواسم)".into(),
            description: Some("مدرس كيمياء يتحول إلى تاجر مخدرات.".into()),
            year: Some(2008),
            duration: Some("5 مواسم".into()),
            episodes: vec![
                Episode {
                    id: 1011,
                    season: 1,
                    episode: 1,
                    title: "Pilot".into(),
                    file_path: TEST_VIDEO.into(),
                },
                Episode {
                    id: 1012,
                    season: 1,
                    episode: 2,
                    title: "Cat's in the Bag...".into(),
                    file_path: TEST_VIDEO.into(),
                },
                Episode {
                    id: 1013,
                    season: 1,
                    episode: 3,
                    title: "...And the Bag's in the River".into(),
                    file_path: TEST_VIDEO.into(),
                },
                Episode {
                    id: 1014,
                    season: 2,
                    episode: 1,
                    title: "Seven Thirty-Seven".into(),
                    file_path: TEST_VIDEO.into(),
                },
                Episode {
                    id: 1015,
                    season: 2,
                    episode: 2,
                    title: "Grilled".into(),
                    file_path: TEST_VIDEO.into(),
                },
            ],
        },
        Media {
            id: 102,
            title: "Stranger Things".into(),
            media_type: MediaType::Series,
            poster: "https://picsum.photos/seed/strangerthings/300/450".into(),
            file_path: "/media/series/strangerthings/".into(),
            size: "32 جيجابايت (4 مواسم)".into(),
            description: Some("مجموعة من الأطفال يكشفون أسرارًا خارقة في بلدتهم.".into()),
            year: Some(2016),
            duration: Some("4 مواسم".into()),
            episodes: vec![
                Episode {
                    id: 1021,
                    season: 1,
                    episode: 1,
                    title: "Chapter One: Will Byers".into(),
                    file_path: TEST_VIDEO.into(),
                },
                Episode {
                    id: 1022,
                    season: 1,
                    episode: 2,
                    title: "Chapter Two: The Weirdo on Maple Street".into(),
                    file_path: TEST_VIDEO.into(),
                },
            ],
        },
        Media {
            id: 103,
            title: "The Crown".into(),
            media_type: MediaType::Series,
            poster: "https://picsum.photos/seed/thecrown/300/450".into(),
            file_path: "/media/series/thecrown/".into(),
            size: "28 جيجابايت (4 مواسم)".into(),
            description: Some("عهد الملكة إليزابيث الثانية.".into()),
            year: Some(2016),
            duration: Some("4 مواسم".into()),
            episodes: vec![Episode {
                id: 1031,
                season: 1,
                episode: 1,
                title: "Wolferton Splash".into(),
                file_path: TEST_VIDEO.into(),
            }],
        },
        Media {
            id: 104,
            title: "Game of Thrones".into(),
            media_type: MediaType::Series,
            poster: "https://picsum.photos/seed/got/300/450".into(),
            file_path: "/media/series/got/".into(),
            size: "68 جيجابايت (8 مواسم)".into(),
            description: Some("عائلات نبيلة تتصارع على السيطرة على ويستروس.".into()),
            year: Some(2011),
            duration: Some("8 مواسم".into()),
            episodes: vec![
                Episode {
                    id: 1041,
                    season: 1,
                    episode: 1,
                    title: "Winter Is Coming".into(),
                    file_path: TEST_VIDEO.into(),
                },
                Episode {
                    id: 1042,
                    season: 1,
                    episode: 2,
                    title: "The Kingsroad".into(),
                    file_path: TEST_VIDEO.into(),
                },
            ],
        },
    ]
}

#[cfg(feature = "ssr")]
async fn delay(ms: i32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

#[server]
async fn fetch_movies() -> Result<Vec<Media>, ServerFnError> {
    delay(300).await;
    Ok(mock_movies())
}
#[server]
async fn fetch_series() -> Result<Vec<Media>, ServerFnError> {
    delay(300).await;
    Ok(mock_series())
}
#[server]
async fn fetch_all_media() -> Result<Vec<Media>, ServerFnError> {
    delay(300).await;
    let mut all = mock_movies();
    all.extend(mock_series());
    Ok(all)
}
#[server]
async fn fetch_media_detail(media_type: String, id: i64) -> Result<Media, ServerFnError> {
    delay(200).await;
    let list = match media_type.as_str() {
        "movie" => mock_movies(),
        "series" => mock_series(),
        _ => return Err(ServerFnError::new("not found")),
    };
    list.into_iter()
        .find(|m| m.id == id)
        .ok_or(ServerFnError::new("not found"))
}

// ---------- ICONS ----------
fn icon(children: impl IntoView, class: &str) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" class=format!("{} fill-none stroke-current", class) viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{children}</svg> }.into_any()
}
#[component]
fn SearchIcon() -> impl IntoView {
    icon(
        view! { <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/> },
        "h-5 w-5",
    )
}
#[component]
fn MovieIcon() -> impl IntoView {
    icon(
        view! { <path d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"/> },
        "h-5 w-5",
    )
}
#[component]
fn SeriesIcon() -> impl IntoView {
    icon(
        view! { <path d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/> },
        "h-5 w-5",
    )
}
#[component]
fn DownloadIcon() -> impl IntoView {
    icon(
        view! { <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/> },
        "h-5 w-5",
    )
}
#[component]
fn PlayIcon() -> impl IntoView {
    icon(view! { <polygon points="5,3 19,12 5,21"/> }, "h-6 w-6")
}
#[component]
fn PauseIcon() -> impl IntoView {
    icon(
        view! { <rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/> },
        "h-6 w-6",
    )
}
#[component]
fn ClockIcon() -> impl IntoView {
    icon(
        view! { <circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/> },
        "h-4 w-4",
    )
}
#[component]
fn UploadIcon() -> impl IntoView {
    icon(
        view! { <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/> },
        "h-6 w-6",
    )
}
#[component]
fn DeleteIcon() -> impl IntoView {
    icon(
        view! { <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/> },
        "h-5 w-5",
    )
}
#[component]
fn UpArrow() -> impl IntoView {
    icon(view! { <polyline points="18,15 12,9 6,15"/> }, "h-4 w-4")
}
#[component]
fn DownArrow() -> impl IntoView {
    icon(view! { <polyline points="6,9 12,15 18,9"/> }, "h-4 w-4")
}
#[component]
fn SortIcon() -> impl IntoView {
    icon(
        view! { <path d="M3 4h13M3 8h9m-9 4h9m5-4v12m0 0l-4-4m4 4l4-4"/> },
        "h-5 w-5",
    )
}
#[component]
fn VolumeIcon() -> impl IntoView {
    icon(
        view! { <path d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/> },
        "h-5 w-5",
    )
}
#[component]
fn MuteIcon() -> impl IntoView {
    icon(
        view! { <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"/><path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/> },
        "h-5 w-5",
    )
}
#[component]
fn FullscreenIcon() -> impl IntoView {
    icon(
        view! { <path d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5"/> },
        "h-5 w-5",
    )
}
#[component]
fn FullscreenExitIcon() -> impl IntoView {
    icon(
        view! { <path d="M9 9V4M9 4H4M9 4l5 5M15 15V20M15 20h5M15 20l-5-5M9 15v5M9 15H4M9 15l5 5M15 9V4M15 4h5M15 4l-5 5"/> },
        "h-5 w-5",
    )
}

// ---------- PERCENT ENCODING ----------
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

// ---------- LAYOUT ----------
#[component]
fn Layout() -> impl IntoView {
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
        <nav
            class="fixed top-0 start-0 end-0 z-50 backdrop-blur-xl bg-black/60 border-b border-white/[0.06] shadow-2xl shadow-black/50"
            >
                <div
                class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8"
            >
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
    <div
        class="flex items-center justify-between h-16 md:h-20"
    >
        <NavbarBrand/>
        <DesktopNavLinks
            search_term=search_term
            search_open=search_open
        />
        <MobileSearch
            search_term=search_term
        />
    </div>
    }
}
#[component]
fn NavbarBrand() -> impl IntoView {
    let navigate = use_navigate();
    view! { <a href="/" class="flex items-center gap-2 text-2xl sm:text-3xl md:text-4xl font-black tracking-tighter" on:click=move |ev: MouseEvent| { ev.prevent_default(); navigate("/", Default::default()); }><span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span></a> }
}
#[component]
fn DesktopNavLinks(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    view! { <div class="hidden md:flex items-center gap-2"><NavLink href="/movies" label="أفلام"/><NavLink href="/series" label="مسلسلات"/><SearchBox search_term=search_term search_open=search_open/></div> }
}
#[component]
fn NavLink(href: &'static str, label: &'static str) -> impl IntoView {
    let navigate = use_navigate();
    view! { <a href=href class="px-4 py-2 rounded-2xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-300 backdrop-blur-sm" on:click=move |ev: MouseEvent| { ev.prevent_default(); navigate(href, Default::default()); }>{label}</a> }
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
    view! { <div class=move || format!("relative me-2 transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] {}", if search_open.get() { "w-64" } else { "w-10" })><form on:submit=on_search class="flex items-center"><SearchToggle search_open=search_open/><SearchInput search_term=search_term search_open=search_open/></form></div> }
}
#[component]
fn SearchToggle(search_open: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| search_open.set(!search_open.get());
    view! {
        <button
            type="button"
            on:click=on_click
            class="absolute start-1 top-1/2 -translate-y-1/2 p-1.5 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors"
        >
            <SearchIcon/>
        </button>
    }
}
#[component]
fn SearchInput(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    view! { <input type="text" prop:value=search_term on:input=move |ev| set_search_target(ev, search_term) on:focus=move |_| search_open.set(true) on:blur=move |_| if search_term.get().is_empty() { search_open.set(false); } placeholder="ابحث..." class=move || format!("w-full bg-white/5 backdrop-blur-xl text-white placeholder-gray-500 rounded-full py-2.5 pe-4 ps-12 text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/10 transition-all duration-300 {}", if search_open.get() { "opacity-100 scale-100" } else { "opacity-0 scale-95 pointer-events-none" }) /> }
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
    let on_input = move |ev| set_search_target(ev, search_term);
    view! {
    <div
        class="md:hidden flex items-center gap-2"
    >
        <form
            on:submit=on_search
            class="relative flex items-center"
        >
            <input
                type="text"
                prop:value=search_term
                on:input=on_input
                placeholder="ابحث..."
                class="w-28 sm:w-36 bg-white/10 backdrop-blur-xl text-white placeholder-gray-400 rounded-full py-1.5 pe-3 ps-3 text-xs focus:outline-none focus:ring-1 focus:ring-cyan-400/50"
            />
            <button
                type="submit"
                class="absolute start-1.5 top-1/2 -translate-y-1/2 text-gray-400"
            >
                <SearchIcon/>
            </button>
        </form>
    </div>
    }
}
#[component]
fn MobileNav() -> impl IntoView {
    view! { <div class="md:hidden flex gap-1 pb-2"><NavLink href="/movies" label="أفلام"/><NavLink href="/series" label="مسلسلات"/></div> }
}
fn set_search_target(ev: web_sys::Event, setter: RwSignal<String>) {
    if let Some(input) = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
    {
        setter.set(input.value());
    }
}
#[component]
fn Footer() -> impl IntoView {
    view! { <footer class="bg-[#0a0a0f]/90 backdrop-blur-xl border-t border-white/5 mt-auto"><div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 md:py-16"><FooterGrid/><FooterCopyright/></div></footer> }
}
#[component]
fn FooterGrid() -> impl IntoView {
    view! { <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-8 md:gap-12"><FooterBrand/><FooterLinks/><FooterLibrary/></div> }
}
#[component]
fn FooterBrand() -> impl IntoView {
    let navigate = use_navigate();
    view! { <div class="space-y-4"><a href="/" class="text-2xl font-black tracking-tighter" on:click=move |ev: MouseEvent| { ev.prevent_default(); navigate("/", Default::default()); }><span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span></a><p class="text-gray-400 text-sm max-w-xs leading-relaxed">"خادم السينما الشخصي الخاص بك — شاهد، حمّل، واستمتع بمجموعتك في أي وقت."</p></div> }
}
#[component]
fn FooterLinks() -> impl IntoView {
    view! { <div><h3 class="text-white font-semibold text-sm mb-4 tracking-wide">تصفح</h3><ul class="space-y-2 text-sm"><li><NavLink href="/movies" label="أفلام"/></li><li><NavLink href="/series" label="مسلسلات"/></li><li><NavLink href="/search" label="بحث"/></li></ul></div> }
}
#[component]
fn FooterLibrary() -> impl IntoView {
    view! { <div><h3 class="text-white font-semibold text-sm mb-4 tracking-wide">المكتبة</h3><ul class="space-y-2 text-sm"><li><NavLink href="/upload" label="رفع وسائط"/></li><li><NavLink href="/settings" label="الإعدادات"/></li><li><span class="text-gray-500 cursor-default">v1.0.0</span></li></ul></div> }
}
#[component]
fn FooterCopyright() -> impl IntoView {
    view! {
        <div
            class="mt-10 pt-6 border-t border-white/5 text-center text-gray-500 text-xs tracking-wide"
        >
            <p>"© 2025 وسائطي. صُنع بكل ❤️ لشبكتك المنزلية."</p>
        </div>
    }
}

// ---------- MEDIA CARD ----------
#[component]
fn MediaCard(item: Media, kind: MediaType) -> impl IntoView {
    let navigate = use_navigate();
    let href = format!("/detail/{}/{}", kind, item.id);

    let href1 = href.clone();
    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        navigate(&href1, Default::default());
    };

    view! {
        <a
            href=href.clone()
            class="group relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03] hover:-translate-y-2"
            on:click=on_click
        >
            <MediaCardImage
                poster=item.poster.clone()
                title=item.title.clone()
                year=item.year
                duration=item.duration.clone()
                media_type=item.media_type
            />
            <MediaCardInfo
                title=item.title
                year=item.year
                size=item.size
            />
        </a>
    }
}
#[component]
fn MediaCardImage(
    poster: String,
    title: String,
    year: Option<u32>,
    duration: Option<String>,
    media_type: MediaType,
) -> impl IntoView {
    view! { <div class="aspect-[2/3] relative overflow-hidden">
        <img src=poster alt=title.clone() class="w-full h-full object-cover transition-transform duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] group-hover:scale-110" loading="lazy" on:error=|_| {} />
        <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
            <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                <MediaCardMeta year=year duration=duration/>
            </div>
        </div>
        <MediaTypeBadge media_type=media_type/>
    </div> }
}
#[component]
fn MediaCardMeta(year: Option<u32>, duration: Option<String>) -> impl IntoView {
    let year = year.map(|y| view! { <span>{y}</span> });
    let duration = duration.map(|y| {
        view! {
            <span class="flex items-center">
                <ClockIcon/>
                {y}
            </span>
        }
    });
    view! {
    <div
        class="flex items-center gap-2 mt-1 text-gray-300 text-sm"
    >
        {year}
        {duration}
    </div>
    }
}

#[component]
fn MediaTypeBadge(media_type: MediaType) -> impl IntoView {
    let icon = match media_type {
        MediaType::Movie => Either::Left(MovieIcon()),
        MediaType::Series => Either::Right(SeriesIcon()),
    };
    let name = match media_type {
        MediaType::Movie => "فيلم",
        MediaType::Series => "مسلسل",
    };
    view! {
    <div
        class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10"
    >
        {icon}
        {name}
    </div>
    }
}
#[component]
fn MediaCardInfo(title: String, year: Option<u32>, size: String) -> impl IntoView {
    view! { <div class="p-4 flex flex-col gap-1"><h3 class="text-white font-semibold truncate text-sm">{title}</h3><div class="flex items-center justify-between text-gray-500 text-xs"><span class="flex items-center gap-1">{year.map(|y| format!("{} · ", y))}{size}</span><span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">"← التفاصيل"</span></div></div> }
}
#[component]
fn CardSkeleton() -> impl IntoView {
    view! { <div class="animate-pulse rounded-2xl bg-[#1a1a24]/60 border border-white/5 overflow-hidden shadow-xl"><div class="aspect-[2/3] bg-gradient-to-b from-[#2a2a3a] to-[#1a1a24]"></div><div class="p-4 space-y-2"><div class="h-3 bg-[#2a2a3a] rounded w-3/4"></div><div class="h-2 bg-[#2a2a3a] rounded w-1/2"></div></div></div> }
}

// ---------- VIDEO PLAYER ----------
#[component]
fn VideoPlayer(src: Signal<String>, #[prop(optional)] title: Option<String>) -> impl IntoView {
    let video_ref = NodeRef::<leptos::html::Video>::new();
    let playing = RwSignal::new(false);
    let current_time = RwSignal::new(0.0);
    let duration = RwSignal::new(0.0);
    let volume = RwSignal::new(1.0);
    let last_volume = RwSignal::new(1.0);
    let muted = RwSignal::new(false);
    let fullscreen = RwSignal::new(false);
    let controls_visible = RwSignal::new(true);
    let controls_timeout = RwSignal::new(None::<i32>);

    let start_hide_timer = {
        move || {
            if let Some(id) = controls_timeout.get() {
                let _ = web_sys::window().unwrap().clear_timeout_with_handle(id);
            }
            let window = web_sys::window().unwrap();
            let cb = Closure::once_into_js(move || controls_visible.set(false));
            let handle = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    3000,
                )
                .unwrap();
            controls_timeout.set(Some(handle));
        }
    };
    let show_controls = {
        let start_hide_timer = start_hide_timer.clone();
        move || {
            controls_visible.set(true);
            start_hide_timer();
        }
    };
    let toggle_controls = {
        let show_controls = show_controls.clone();
        let controls_timeout = controls_timeout.clone();
        move || {
            if controls_visible.get() {
                controls_visible.set(false);
                if let Some(id) = controls_timeout.get() {
                    let _ = web_sys::window().unwrap().clear_timeout_with_handle(id);
                }
            } else {
                show_controls();
            }
        }
    };
    let handle_loaded_metadata = {
        let video_ref = video_ref.clone();
        move |_| {
            if let Some(video) = video_ref.get() {
                duration.set(video.duration());
            }
        }
    };
    let handle_time_update = {
        let video_ref = video_ref.clone();
        move |_| {
            if let Some(video) = video_ref.get() {
                current_time.set(video.current_time());
            }
        }
    };
    let toggle_play = {
        let video_ref = video_ref.clone();
        move |_| {
            if let Some(video) = video_ref.get() {
                if playing.get() {
                    video.pause().ok();
                } else {
                    let _ = video.play();
                }
            }
        }
    };
    let handle_seek = {
        let video_ref = video_ref.clone();
        move |ev: web_sys::Event| {
            if let Some(input) = ev
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                if let Ok(val) = input.value().parse::<f64>() {
                    if let Some(video) = video_ref.get() {
                        video.set_current_time(val);
                        current_time.set(val);
                    }
                }
            }
        }
    };
    let handle_volume = {
        let video_ref = video_ref.clone();
        move |ev: web_sys::Event| {
            if let Some(input) = ev
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                if let Ok(val) = input.value().parse::<f64>() {
                    if let Some(video) = video_ref.get() {
                        video.set_volume(val);
                        video.set_muted(val == 0.0);
                        volume.set(val);
                        muted.set(val == 0.0);
                        if val > 0.0 {
                            last_volume.set(val);
                        }
                    }
                }
            }
        }
    };
    let toggle_mute = {
        let video_ref = video_ref.clone();
        move |_| {
            if let Some(video) = video_ref.get() {
                if muted.get() {
                    video.set_muted(false);
                    let restore = last_volume.get().max(0.1);
                    video.set_volume(restore);
                    volume.set(restore);
                    muted.set(false);
                } else {
                    last_volume.set(volume.get().max(0.1));
                    video.set_muted(true);
                    muted.set(true);
                }
            }
        }
    };
    let toggle_fullscreen = {
        let video_ref = video_ref.clone();
        move |_| {
            if let Some(video) = video_ref.get() {
                if document().fullscreen_element().is_none() {
                    let _ = video.request_fullscreen();
                    fullscreen.set(true);
                } else {
                    let _ = document().exit_fullscreen();
                    fullscreen.set(false);
                }
            }
        }
    };
    on_cleanup(move || {
        if let Some(id) = controls_timeout.get() {
            let _ = web_sys::window().unwrap().clear_timeout_with_handle(id);
        }
    });
    Effect::new(move || {
        if let Some(video) = video_ref.get() {
            video.set_src(&src.get());
            let _ = video.load();
            playing.set(false);
            current_time.set(0.0);
            duration.set(0.0);
        }
    });
    let format_time = |time: f64| -> String {
        if time.is_nan() {
            "00:00".into()
        } else {
            format!("{:02}:{:02}", (time / 60.0) as u32, (time % 60.0) as u32)
        }
    };

    view! { <div dir="ltr" class="relative bg-black rounded-2xl overflow-hidden shadow-2xl shadow-black/50 group">
        <VideoElement video_ref=video_ref title=title playing=playing handle_loaded_metadata=handle_loaded_metadata handle_time_update=handle_time_update toggle_controls=toggle_controls />
        <VideoControls controls_visible=controls_visible show_controls=show_controls controls_timeout=controls_timeout format_time=format_time
            current_time=current_time duration=duration playing=playing muted=muted volume=volume fullscreen=fullscreen
            toggle_play=toggle_play toggle_mute=toggle_mute toggle_fullscreen=toggle_fullscreen
            handle_seek=handle_seek handle_volume=handle_volume />
    </div> }
}

#[component]
fn VideoElement(
    video_ref: NodeRef<leptos::html::Video>,
    title: Option<String>,
    playing: RwSignal<bool>,
    handle_loaded_metadata: impl Fn(web_sys::Event) + 'static,
    handle_time_update: impl Fn(web_sys::Event) + 'static,
    toggle_controls: impl Fn() + Clone + 'static,
) -> impl IntoView {
    view! { <video node_ref=video_ref title=title class="w-full h-auto max-h-[60vh] md:max-h-[70vh] object-contain cursor-pointer"
    on:loadedmetadata=handle_loaded_metadata on:timeupdate=handle_time_update
    on:play=move |_| playing.set(true) on:pause=move |_| playing.set(false)
    on:ended=move |_| playing.set(false) on:click=move |_| toggle_controls() playsinline /> }
}

#[component]
fn VideoControls(
    controls_visible: RwSignal<bool>,
    show_controls: impl Fn() + Clone + 'static,
    controls_timeout: RwSignal<Option<i32>>,
    format_time: impl Fn(f64) -> String + 'static,
    current_time: RwSignal<f64>,
    duration: RwSignal<f64>,
    playing: RwSignal<bool>,
    muted: RwSignal<bool>,
    volume: RwSignal<f64>,
    fullscreen: RwSignal<bool>,
    toggle_play: impl Fn(MouseEvent) + 'static,
    toggle_mute: impl Fn(MouseEvent) + 'static,
    toggle_fullscreen: impl Fn(MouseEvent) + 'static,
    handle_seek: impl Fn(web_sys::Event) + 'static,
    handle_volume: impl Fn(web_sys::Event) + 'static,
) -> impl IntoView {
    view! { <div class=move || format!("absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3 sm:p-5 transition-opacity duration-300 {}", if controls_visible.get() { "opacity-100" } else { "opacity-0" })
        on:mouseenter={let show = show_controls.clone(); move |_| show()}
        on:mouseleave={ let controls_timeout = controls_timeout.clone(); move |_| { if let Some(id) = controls_timeout.get() { let _ = web_sys::window().unwrap().clear_timeout_with_handle(id); } let window = web_sys::window().unwrap(); let cb = Closure::once_into_js(move || controls_visible.set(false)); let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 1500).unwrap(); controls_timeout.set(Some(handle)); }}
        on:touchstart={let show = show_controls.clone(); move |_| show()}>
        <div class="flex flex-col gap-2">
            <SeekBar current_time=current_time duration=duration format_time=format_time handle_seek=handle_seek/>
            <ControlButtons playing=playing muted=muted volume=volume fullscreen=fullscreen
                toggle_play=toggle_play toggle_mute=toggle_mute toggle_fullscreen=toggle_fullscreen handle_volume=handle_volume/>
        </div>
    </div> }
}

#[component]
fn SeekBar(
    current_time: RwSignal<f64>,
    duration: RwSignal<f64>,
    format_time: impl Fn(f64) -> String + 'static,
    handle_seek: impl Fn(web_sys::Event) + 'static,
) -> impl IntoView {
    view! { <div class="flex items-center gap-2">
        <span class="text-white text-xs font-mono">{format_time(current_time.get())}</span>
        <input type="range" min="0" max={duration.get()} value={current_time.get()} on:input=handle_seek class="flex-1 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-cyan-400/30"/>
        <span class="text-white text-xs font-mono">{format_time(duration.get())}</span>
    </div> }
}

#[component]
fn ControlButtons(
    playing: RwSignal<bool>,
    muted: RwSignal<bool>,
    volume: RwSignal<f64>,
    fullscreen: RwSignal<bool>,
    toggle_play: impl Fn(MouseEvent) + 'static,
    toggle_mute: impl Fn(MouseEvent) + 'static,
    toggle_fullscreen: impl Fn(MouseEvent) + 'static,
    handle_volume: impl Fn(web_sys::Event) + 'static,
) -> impl IntoView {
    let play_icon = if playing.get() {
        Either::Left(PauseIcon())
    } else {
        Either::Right(PlayIcon())
    };
    let mute_icon = if muted.get() || volume.get() == 0.0 {
        Either::Left(MuteIcon())
    } else {
        Either::Right(VolumeIcon())
    };
    let volume = if muted.get() { 0.0 } else { volume.get() };
    let full_screen = if fullscreen.get() {
        Either::Left(FullscreenExitIcon())
    } else {
        Either::Right(FullscreenIcon())
    };
    view! {
    <div class="flex items-center gap-4 text-white">
        <button
            on:click=toggle_play
            class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
        >
            {play_icon}
        </button>
        <div class="flex items-center gap-2">
            <button
                on:click=toggle_mute
                class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
            >
                {mute_icon}
            </button>
            <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={volume}
                on:input=handle_volume
                class="w-16 sm:w-20 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400"
            />
        </div>
        <div class="flex-1"></div>
        <button
            on:click=toggle_fullscreen
            class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
        >
            {full_screen}
        </button>
    </div> }
}

// ---------- DETAIL PAGE ----------
#[component]
fn DetailPoster(poster: String, title: String) -> impl IntoView {
    view! {
    <div
        class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0"><img src=poster class="w-full rounded-2xl shadow-2xl border border-white/10"
        alt=title
    />
    </div>
    }
}
#[component]
fn DetailMetaBadge(media_type: MediaType) -> impl IntoView {
    let media_icon = match media_type {
        MediaType::Movie => Either::Left(MovieIcon()),
        MediaType::Series => Either::Right(SeriesIcon()),
    };
    let name = match media_type {
        MediaType::Movie => "فيلم",
        MediaType::Series => "مسلسل",
    };
    view! {
    <div
        class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5"
    >
        {media_icon}
        {name}
    </div>
    }
}
#[component]
fn DetailInfo(data: Media) -> impl IntoView {
    let year = data.year.map(|y| view! { <span>{y}</span> });
    let duration = data.duration.map(|y| {
        view! {
            <span
                class="flex items-center gap-1"
            >
                <ClockIcon/>
                {y}
            </span>
        }
    });
    let download = matches!(data.media_type,MediaType::Movie).then_some(
        view! {
        <a
            href=data.file_path.clone()
            class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm"
            ><DownloadIcon/>
             "تحميل"
         </a>
         });
    view! {
        <h1
            class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2"
        >
            {data.title.clone()}
        </h1>
        <div
            class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base"
        >
            {year}
            {duration}
            <span>{data.size.clone()}</span>
        </div>
        <p
            class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg"
        >
            {data.description.clone().unwrap_or_else(|| "لا يوجد وصف متاح.".into())}
        </p>
        <div class="mt-6 flex gap-3">
            {download}
        </div>
    }
}
#[component]
fn EpisodeSelector(
    episodes: Vec<Episode>,
    selected_episode: RwSignal<Option<Episode>>,
) -> impl IntoView {
    view! {
    <div
        class="mt-10"
    >
        <h2
            class="text-xl sm:text-2xl font-bold text-white mb-4 flex items-center gap-2"
        >
            <SeriesIcon/>
            " الحلقات"
        </h2>
        <div
            class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
            <For
                each={move || episodes.clone()}
                key=|ep| ep.id
                let:ep
            >
                <EpisodeCard ep=ep selected_episode=selected_episode/>
            </For>
        </div>
    </div>
    }
}
#[component]
fn EpisodeCard(ep: Episode, selected_episode: RwSignal<Option<Episode>>) -> impl IntoView {
    let class = move || {
        format!(
            "p-3 rounded-xl border transition-all cursor-pointer backdrop-blur-sm {}",
            if selected_episode
                .get()
                .as_ref()
                .map_or(false, |s| s.id == ep.id)
            {
                "border-cyan-400 bg-cyan-400/10 shadow-lg shadow-cyan-400/10"
            } else {
                "border-white/10 bg-white/5 hover:bg-white/10 hover:border-white/20"
            }
        )
    };
    let title = ep.title.clone();
    let episode = ep.episode.clone();
    let season = ep.season.clone();
    let on_click = move |_| selected_episode.set(Some(ep.clone()));
    view! {
    <div
        class=class
        on:click=on_click
    >
        <div
            class="flex items-center gap-3"
        >
            <span
                class="text-sm font-mono text-gray-400"
            >
                "S"
                {format!("{:02}", season)}
                "E"
                {format!("{:02}", episode)}
            </span>
            <span
                class="text-sm text-white truncate"
            >
                {title}
            </span>
        </div>
    </div>
    }
}

#[component]
fn Detail() -> impl IntoView {
    let params = use_params_map();
    let media_type =
        move || params.with(|p| p.get("kind").map(|s| s.to_string()).unwrap_or_default());
    let id = move || params.with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0));
    let detail = Resource::new(
        move || (media_type(), id()),
        |(t, i): (String, i64)| async move { fetch_media_detail(t, i).await },
    );
    let selected_episode = RwSignal::new(None::<Episode>);
    Effect::new(move || {
        if let Some(Ok(data)) = detail.get() {
            if matches!(data.media_type, MediaType::Series) && !data.episodes.is_empty() {
                selected_episode.set(Some(data.episodes[0].clone()));
            } else {
                selected_episode.set(None);
            }
        }
    });
    let video_src = Memo::new(move |_| {
        if let Some(Ok(data)) = detail.get() {
            match data.media_type {
                MediaType::Movie => data.file_path.clone(),
                MediaType::Series => selected_episode
                    .get()
                    .map(|ep| ep.file_path)
                    .unwrap_or_default(),
            }
        } else {
            String::new()
        }
    });
    let fallback = || {
        view! {
            <div
                class="min-h-screen flex items-center justify-center text-white text-lg"
            >"جارٍ التحميل..."
            </div>
        }
    };
    view! {
    <Suspense
        fallback=fallback>
        {move || detail.get().and_then(|x| x.ok()).map(|data| view! {
            <div
                class="relative min-h-screen bg-black text-white overflow-hidden"
            >
                <div
                    class="absolute inset-0"
                >
                    <img
                        src=data.poster.clone()
                        class="w-full h-full object-cover scale-110 blur-3xl opacity-20"
                        alt=""
                    />
                    <div
                        class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"
                    ></div>
                </div>
                <div
                    class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32"
                >
                    <DetailBody
                         data=data
                         video_src=video_src
                         selected_episode=selected_episode
                     />
                </div>
            </div>
        })}
    </Suspense> }
}

#[component]
fn DetailBody(
    data: Media,
    video_src: Memo<String>,
    selected_episode: RwSignal<Option<Episode>>,
) -> impl IntoView {
    let title = data.title.clone();
    let video_player = move || {
        (!video_src.get().is_empty()).then_some(view! {
            <div
                class="mt-10"
            >
                <VideoPlayer
                    src=Signal::derive({move || video_src.get()})
                    title=title.clone()
                />
            </div>
        })
    };
    let available_series =
        matches!(data.media_type, MediaType::Series) && !data.episodes.is_empty();
    let episodes = data.episodes.clone();
    let selector = move || {
        available_series.then_some(view! {
            <EpisodeSelector
                episodes=episodes.clone()
                selected_episode=selected_episode
            />
        })
    };
    view! {
        <div
            class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start"
        >
            <DetailPoster
                poster=data.poster.clone()
                title=data.title.clone()
            />
            <div
                class="flex-1 w-full"
            >
                <DetailMetaBadge
                    media_type=data.media_type.clone()
                />
                <DetailInfo
                    data=data.clone()
                />
            </div>
        </div>
        {video_player}
        {selector}
    }
}

// ---------- HOME PAGE ----------
#[component]
fn HomeHero() -> impl IntoView {
    view! {
    <div
        class="py-12 sm:py-16 md:py-20 lg:py-24 text-center"
    >
        <h1
            class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-black tracking-tight leading-[1.1]"
        >
            <span
                class="bg-gradient-to-r from-cyan-200 via-blue-300 to-indigo-400 bg-clip-text text-transparent"
            >
                "سينماك"
            </span>
            <br
                class="sm:hidden"
            />
            <span
                class="text-white"
            >
                " الشخصية"
            </span>
        </h1>
        <p
            class="text-gray-400 text-base sm:text-lg md:text-xl max-w-2xl mx-auto mt-4 leading-relaxed"
        >
            "شاهد وحمّل مجموعتك من الأفلام والمسلسلات من أي مكان في منزلك."
        </p>
    </div>
    }
}
#[component]
fn MediaSection(
    title: String,
    icon: impl IntoView,
    items: Signal<Vec<Media>>,
    kind: MediaType,
) -> impl IntoView {
    let navigate = use_navigate();
    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        navigate(&kind.to_string(), Default::default());
    };
    view! {
    <section
        class="mb-12 md:mb-16"
    >
        <div
            class="flex items-center justify-between mb-6"
        >
            <h2
                class="text-2xl sm:text-3xl md:text-4xl font-black text-white flex items-center gap-3"
            >
                <span class="text-cyan-400">
                    {icon}
                </span>
                {title.clone()}
            </h2>
            <a
                class="text-cyan-400 hover:text-cyan-300 text-sm font-medium transition-all flex items-center gap-1 group"
                on:click=on_click
            >
                <span
                    class="text-lg group-hover:translate-x-1 transition-transform"
                >
                    "←"
                </span>
                " عرض الكل"
            </a>
        </div>
        <div
            class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6"
        >
            <For
                each={move || items.get().into_iter().take(5).collect::<Vec<_>>()}
                key=|m| m.id
                let:item
            >
                <MediaCard item=item.clone() kind=kind.clone()/>
            </For>
        </div>
    </section>
    }
}
#[component]
fn Home() -> impl IntoView {
    let media = Resource::new(|| (), |_| async move { fetch_all_media().await });

    let fallback = || {
        view! {
            <div
                class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6"
            >
                <For
                    each={move || (0..5).collect::<Vec<_>>()}
                    key=|i| *i
                    let:_
                >{CardSkeleton}
                </For>
            </div>
        }
    };
    view! {
    <div
        class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8"
    >
        <HomeHero/>
        <Suspense fallback=fallback>
            {move || {
                let (movies,series) : (Vec<_>,Vec<_>) = media
                    .get()
                    .and_then(|x| x.ok())
                    .unwrap_or_default()
                    .into_iter()
                    .partition(|m| matches!(m.media_type, MediaType::Movie));
                view! {
                    <MediaSection
                        title="أفلام".to_string()
                        icon=MovieIcon()
                        items=Signal::derive(move || movies.clone())
                        kind=MediaType::Movie
                    />
                    <MediaSection
                        title="مسلسلات".to_string()
                        icon=SeriesIcon()
                        items=Signal::derive(move || series.clone())
                        kind=MediaType::Series
                    />
                }
            }}
        </Suspense>
    </div> }
}

// ---------- MOVIES / SERIES ----------
#[component]
fn MediaPageHeader(title: String, icon: impl IntoView) -> impl IntoView {
    view! { <div class="flex items-center gap-4 mb-6 md:mb-8"><div class="p-3 bg-cyan-400/10 rounded-2xl text-cyan-400">{icon}</div><div><h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">{title.clone()}</h1><p class="text-gray-400 text-sm md:text-base mt-0.5">تصفح مجموعة {title}ك</p></div></div> }
}
#[component]
fn Movies() -> impl IntoView {
    let movies = Resource::new(|| (), |_| async move { fetch_movies().await });
    view! { <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <MediaPageHeader title="أفلام".to_string() icon=MovieIcon()/>
        <Suspense fallback=|| view! { <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6"><For each={move || (0..8).collect::<Vec<_>>()} key=|i| *i let:_>{CardSkeleton}</For></div> }>
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                <For each={move || movies.get().and_then(|x| x.ok()).unwrap_or_default()} key=|m| m.id let:item>
                    <MediaCard item=item.clone() kind=MediaType::Movie/>
                </For>
            </div>
        </Suspense>
    </div> }
}
#[component]
fn Series() -> impl IntoView {
    let series = Resource::new(|| (), |_| async move { fetch_series().await });
    view! { <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <MediaPageHeader title="مسلسلات".to_string() icon=SeriesIcon()/>
        <Suspense fallback=|| view! { <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6"><For each={move || (0..8).collect::<Vec<_>>()} key=|i| *i let:_>{CardSkeleton}</For></div> }>
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                <For each={move || series.get().and_then(|x| x.ok()).unwrap_or_default()} key=|m| m.id let:item>
                    <MediaCard item=item.clone() kind=MediaType::Series/>
                </For>
            </div>
        </Suspense>
    </div> }
}

// ---------- SEARCH PAGE ----------
#[component]
fn SearchHeader() -> impl IntoView {
    let query_map = use_query_map();
    let query = move || {
        query_map.with(|m| {
            m.get("q")
                .map(|s| s.to_string())
                .unwrap_or_default()
                .trim()
                .to_lowercase()
        })
    };
    view! { <div class="mb-6 md:mb-8">
        <h1 class="text-3xl sm:text-4xl font-black text-white mb-1">نتائج البحث</h1>
        {move || if query().is_empty() { Either::Left(view! { <p class="text-gray-400 text-sm sm:text-base">أدخل كلمة بحث للعثور على الوسائط.</p> }) }
            else { let q = query_map.with(|m| m.get("q").map(|s| s.to_string()).unwrap_or_default()); Either::Right(view! { <p class="text-gray-400 text-sm sm:text-base">نتائج البحث عن <span class="text-white font-semibold">{format!("\"{}\"", q)}</span></p> }) }
        }
    </div> }
}
#[component]
fn Search() -> impl IntoView {
    let query_map = use_query_map();
    let query = move || {
        query_map.with(|m| {
            m.get("q")
                .map(|s| s.to_string())
                .unwrap_or_default()
                .trim()
                .to_lowercase()
        })
    };
    let all_media = Resource::new(|| (), |_| async move { fetch_all_media().await });
    let results = Memo::new(move |_| {
        let q = query();
        if q.is_empty() {
            return vec![];
        }
        all_media
            .get()
            .and_then(|x| x.ok())
            .map(|media| {
                media
                    .into_iter()
                    .filter(|item| item.title.to_lowercase().contains(&q))
                    .collect()
            })
            .unwrap_or_default()
    });
    view! { <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SearchHeader/>
        <Suspense fallback=|| view! { <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6"><For each={move || (0..4).collect::<Vec<_>>()} key=|i| *i let:_>{CardSkeleton}</For></div> }>
            {move || if results.get().is_empty() { Either::Left(view! { <div class="text-center py-16 text-gray-400 text-sm sm:text-base">لا يوجد وسائط تطابق بحثك.</div> }) }
                else { Either::Right(view! { <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                    <For each={move || results.get()} key=|m| m.id let:item>
                        <MediaCard item=item.clone() kind=item.media_type/>
                    </For>
                </div> }) }
            }
        </Suspense>
    </div> }
}

// ---------- UPLOAD PAGE ----------
#[derive(Clone, Debug)]
struct EpUpload {
    id: u32,
    file: web_sys::File,
    title: String,
}

#[component]
fn UploadHeader() -> impl IntoView {
    view! { <div class="mb-8 md:mb-10 text-center"><div class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4"><span class="text-cyan-400">{UploadIcon()}</span></div><h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">رفع وسائط جديدة</h1><p class="text-gray-400 text-sm sm:text-base mt-2">"أضف فيلمًا أو مسلسلًا إلى مكتبتك المنزلية"</p></div> }
}
#[component]
fn TypeSelector(media_type: RwSignal<MediaType>) -> impl IntoView {
    view! { <div class="flex justify-center"><div class="inline-flex bg-white/5 rounded-2xl p-1" role="group">
        <button type="button" on:click=move |_| media_type.set(MediaType::Series) class=move || format!("px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 {}", if matches!(media_type.get(), MediaType::Series) { "bg-purple-500/20 text-purple-400 shadow-lg shadow-purple-500/10" } else { "text-gray-400 hover:text-white" })>{SeriesIcon()} "مسلسل"</button>
        <button type="button" on:click=move |_| media_type.set(MediaType::Movie) class=move || format!("px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 {}", if matches!(media_type.get(), MediaType::Movie) { "bg-cyan-500/20 text-cyan-400 shadow-lg shadow-cyan-500/10" } else { "text-gray-400 hover:text-white" })>{MovieIcon()} "فيلم"</button>
    </div></div> }
}
#[component]
fn TitleInput(title: RwSignal<String>, disabled: Signal<bool>) -> impl IntoView {
    view! { <div><label class="block text-sm font-medium text-gray-300 mb-1.5">العنوان *</label><input type="text" prop:value=title on:input=move |ev| set_title_target(ev, title) required placeholder="مثال: Breaking Bad" class=move || format!("w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition {}", if disabled.get() { "opacity-60 cursor-not-allowed" } else { "" }) disabled=disabled.get()/></div> }
}
#[component]
fn DescriptionInput(description: RwSignal<String>) -> impl IntoView {
    view! { <div><label class="block text-sm font-medium text-gray-300 mb-1.5">الوصف (اختياري)</label><textarea prop:value=description on:input=move |ev| set_description_target(ev, description) rows=3 placeholder="وصف مختصر (اختياري)..." class="w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none"/></div> }
}
fn set_title_target(ev: web_sys::Event, setter: RwSignal<String>) {
    if let Some(input) = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
    {
        setter.set(input.value());
    }
}
fn set_description_target(ev: web_sys::Event, setter: RwSignal<String>) {
    if let Some(textarea) = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
    {
        setter.set(textarea.value());
    }
}

#[component]
fn SeriesSettings(
    is_new_series: RwSignal<bool>,
    existing_series_id: RwSignal<Option<i64>>,
    series_list: Signal<Vec<Media>>,
) -> impl IntoView {
    view! { <div class="space-y-4">
        <div class="flex flex-wrap items-center gap-4">
            <label class="text-sm font-medium text-gray-300">نوع المسلسل:</label>
            <div class="inline-flex bg-white/5 rounded-xl p-0.5">
                <button type="button" on:click=move |_| { is_new_series.set(true); existing_series_id.set(None); } class=move || format!("px-3 py-1.5 rounded-lg text-sm font-medium transition {}", if is_new_series.get() { "bg-cyan-500/20 text-cyan-400" } else { "text-gray-400 hover:text-white" })>جديد</button>
                <button type="button" on:click=move |_| is_new_series.set(false) class=move || format!("px-3 py-1.5 rounded-lg text-sm font-medium transition {}", if !is_new_series.get() { "bg-cyan-500/20 text-cyan-400" } else { "text-gray-400 hover:text-white" })>موجود</button>
            </div>
        </div>
        {move || if !is_new_series.get() { Some(view! { <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">اختر المسلسل الموجود</label>
            <select prop:value=move || existing_series_id.get().map(|id| id.to_string()).unwrap_or_default()
                on:change=move |ev| { if let Some(sel) = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) { existing_series_id.set(sel.value().parse().ok()); } }
                class="w-full bg-white/10 backdrop-blur-md text-white rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50">
                <option value="" class="bg-gray-800">"-- اختر --"</option>
                <For each={move || series_list.get()} key=|s| s.id let:s>
                    <option value={s.id.to_string()} class="bg-gray-800">{s.title}</option>
                </For>
            </select>
        </div> }) } else { None }}
    </div> }
}

#[component]
fn MovieFileInput(movie_file: RwSignal<Option<web_sys::File>>) -> impl IntoView {
    view! { <div><label class="block text-sm font-medium text-gray-300 mb-1.5">ملف الفيلم</label><div class="flex flex-wrap items-center gap-4">
        <input type="file" id="movieFileInput" class="hidden" accept="video/*" on:change=move |ev| { if let Some(input) = ev.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) { movie_file.set(input.files().and_then(|f| f.get(0))); } }/>
        <label for="movieFileInput" class="inline-flex items-center gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-2 px-5 rounded-xl cursor-pointer transition text-sm">{UploadIcon()} اختر ملف</label>
        <span class="text-sm text-gray-400">{move || movie_file.get().as_ref().map(|f| f.name()).unwrap_or_else(|| "لم يتم اختيار ملف".to_string())}</span>
    </div></div> }
}

#[component]
fn EpisodeItem(episodes: RwSignal<Vec<EpUpload>>, ep_id: u32, index: usize) -> impl IntoView {
    let total = move || episodes.get().len();
    let remove = move |_| episodes.update(|eps| eps.retain(|e| e.id != ep_id));
    let move_up = move |_| {
        episodes.update(|eps| {
            if let Some(pos) = eps.iter().position(|e| e.id == ep_id) {
                if pos > 0 {
                    eps.swap(pos, pos - 1);
                }
            }
        })
    };
    let move_down = move |_| {
        episodes.update(|eps| {
            if let Some(pos) = eps.iter().position(|e| e.id == ep_id) {
                if pos + 1 < eps.len() {
                    eps.swap(pos, pos + 1);
                }
            }
        })
    };
    let title_update = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            episodes.update(|eps| {
                if let Some(ep) = eps.iter_mut().find(|e| e.id == ep_id) {
                    ep.title = input.value();
                }
            });
        }
    };
    let ep = move || episodes.get().into_iter().find(|e| e.id == ep_id).unwrap();
    view! {
        <div class="bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start">
            <div class="flex-1 grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
                <div><span class="text-gray-400 text-sm font-medium">رقم الحلقة</span><div class="text-white font-semibold mt-0.5">{index + 1}</div></div>
                <div class="sm:col-span-2">
                    <label class="text-xs text-gray-400 mb-0.5 block">عنوان الحلقة</label>
                    <input type="text" prop:value=move || ep().title on:input=title_update placeholder="عنوان الحلقة"
                        class="w-full bg-white/10 text-white rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
                </div>
                <div class="hidden sm:block"><span class="text-xs text-gray-400">الملف</span><div class="text-xs text-gray-300 truncate mt-0.5 max-w-32">{move || ep().file.name()}</div></div>
            </div>
            <div class="flex items-center gap-1 mt-1 sm:mt-0">
                <button on:click=move_up disabled=move || index == 0 class="text-gray-400 hover:text-white transition disabled:opacity-30 p-1" title="نقل للأعلى">{UpArrow()}</button>
                <button on:click=move_down disabled=move || index + 1 == total() class="text-gray-400 hover:text-white transition disabled:opacity-30 p-1" title="نقل للأسفل">{DownArrow()}</button>
                <button on:click=remove class="text-red-400 hover:text-red-300 transition p-1" title="حذف الحلقة">{DeleteIcon()}</button>
            </div>
        </div>
    }
}

#[component]
fn EpisodesToolbar(episodes: RwSignal<Vec<EpUpload>>, next_id: RwSignal<u32>) -> impl IntoView {
    let sort =
        move |_| episodes.update(|eps| eps.sort_by(|a, b| a.file.name().cmp(&b.file.name())));
    let file_handler = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            if let Some(files) = input.files() {
                let mut new_eps: Vec<EpUpload> = (0..files.length())
                    .filter_map(|i| files.get(i))
                    .map(|file| {
                        let name = file.name();
                        let title = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
                        EpUpload {
                            id: next_id.get(),
                            file,
                            title,
                        }
                    })
                    .collect();
                new_eps.sort_by(|a, b| a.file.name().cmp(&b.file.name()));
                episodes.update(|eps| eps.extend(new_eps));
                next_id.update(|id| *id += files.length());
                input.set_value("");
            }
        }
    };
    view! {
        <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="text-lg font-bold text-white flex items-center gap-2">{SeriesIcon()} الحلقات</h2>
            <div class="flex flex-wrap items-center gap-2">
                <input type="file" id="multiEpisodeInput" class="hidden" multiple accept="video/*" on:change=file_handler/>
                <label for="multiEpisodeInput" class="inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm">{UploadIcon()} "اختيار ملفات"</label>
                <button type="button" on:click=sort class="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm">{SortIcon()} "ترتيب"</button>
            </div>
        </div>
    }
}

#[component]
fn EpisodesSection(episodes: RwSignal<Vec<EpUpload>>, next_id: RwSignal<u32>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <EpisodesToolbar episodes=episodes next_id=next_id/>
            <div class="space-y-3 max-h-80 overflow-y-auto p-1">
                <For each={move || episodes.get().into_iter().enumerate().collect::<Vec<_>>()} key=|(i, _)| *i let:item>
                    {move || { let (i, ep) = item.clone(); view! { <EpisodeItem episodes=episodes ep_id=ep.id index=i/> } }}
                </For>
            </div>
            <p class="text-xs text-gray-500">"يتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب أو زر ترتيب للفرز الأبجدي."</p>
        </div>
    }
}

#[component]
fn Upload() -> impl IntoView {
    let title = RwSignal::new(String::new());
    let media_type = RwSignal::new(MediaType::Series);
    let description = RwSignal::new(String::new());
    let movie_file = RwSignal::new(None::<web_sys::File>);
    let is_new_series = RwSignal::new(true);
    let existing_series_id = RwSignal::new(None::<i64>);
    let all_media = Resource::new(|| (), |_| async move { fetch_all_media().await });
    let series_list = Memo::new(move |_| {
        all_media
            .get()
            .and_then(|x| x.ok())
            .map(|m| {
                m.into_iter()
                    .filter(|x| matches!(x.media_type, MediaType::Series))
                    .collect()
            })
            .unwrap_or_default()
    });
    let episodes = RwSignal::new(Vec::<EpUpload>::new());
    let next_id = RwSignal::new(1u32);
    let disabled = Signal::derive(move || {
        !is_new_series.get() && matches!(media_type.get(), MediaType::Series)
    });
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if matches!(media_type.get(), MediaType::Series) && episodes.get().is_empty() {
            return;
        }
        title.set(String::new());
        description.set(String::new());
        movie_file.set(None);
        episodes.set(Vec::new());
        next_id.set(1);
        is_new_series.set(true);
        existing_series_id.set(None);
    };
    view! { <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
        <UploadHeader/>
        <div class="backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl">
            <form on:submit=handle_submit class="space-y-6 md:space-y-8">
                <TypeSelector media_type=media_type/>
                <div class="space-y-4"><TitleInput title=title disabled=disabled/><DescriptionInput description=description/></div>
                {move || if matches!(media_type.get(), MediaType::Series) { Some(view! {
                    <Suspense fallback=|| view! { <div class="text-gray-400 text-sm">"جارٍ تحميل قائمة المسلسلات..."</div> }>
                        <SeriesSettings is_new_series=is_new_series existing_series_id=existing_series_id series_list=Signal::derive(move || series_list.get())/>
                    </Suspense>
                }) } else { None }}
                {move || if matches!(media_type.get(), MediaType::Movie) { Some(view! { <MovieFileInput movie_file=movie_file/> }) } else { None }}
                {move || if matches!(media_type.get(), MediaType::Series) { Some(view! { <EpisodesSection episodes=episodes next_id=next_id/> }) } else { None }}
                <UploadSubmitButton/>
            </form>
        </div>
    </div> }
}

#[component]
fn UploadSubmitButton() -> impl IntoView {
    view! { <button type="submit" class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 flex items-center justify-center gap-2">{UploadIcon()} رفع الوسائط</button> }
}

// ---------- SETTINGS ----------
#[component]
fn Settings() -> impl IntoView {
    view! { <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16"><div class="text-center text-white"><h1 class="text-4xl font-black mb-4">الإعدادات</h1><p class="text-gray-400">سيتم إضافة صفحة الإعدادات قريباً.</p></div></div> }
}

// ---------- SHELL & APP ----------
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ar" dir="rtl">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/><MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/mydisc.css"/>
        <Title text="وسائطي - سينماك الشخصية"/>
        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <ParentRoute path=path!("") view=Layout>
                    <Route path=path!("/") view=Home/>
                    <Route path=path!("/movies") view=Movies/>
                    <Route path=path!("/series") view=Series/>
                    <Route path=path!("/upload") view=Upload/>
                    <Route path=path!("/search") view=Search/>
                    <Route path=path!("/settings") view=Settings/>
                    <Route path=path!("/detail/:kind/:id") view=Detail/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
