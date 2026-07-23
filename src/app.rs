use crate::app::{
    home::HomePage,
    series::{
        details::SeriesDetailPage,
        fetch_season,
        listing::{
            mock_series, Episode, EpisodeSelector, Season, SeasonSelector, Series, SeriesPage,
        },
    },
    upload::UploadPage,
    video_player::VideoPlayer,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    hooks::{use_navigate, use_params_map, use_query_map},
    path, Lazy,
};
use serde::{Deserialize, Serialize};
use web_sys::MouseEvent;

mod home;
mod resource_view;
mod series;
mod upload;
mod video_player;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaId(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSize(pub u64);

impl FileSize {
    pub fn human_readable(&self) -> String {
        let bytes = self.0 as f64;
        if bytes >= 1_000_000_000.0 {
            format!("{:.1} جيجابايت", bytes / 1_000_000_000.0)
        } else if bytes >= 1_000_000.0 {
            format!("{:.1} ميجابايت", bytes / 1_000_000.0)
        } else if bytes >= 1_000.0 {
            format!("{:.1} كيلوبايت", bytes / 1_000.0)
        } else {
            format!("{} بايت", bytes)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationSeconds(pub u64);

impl DurationSeconds {
    pub fn human_readable(&self) -> String {
        let secs = self.0;
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        if hours > 0 {
            format!("{} ساعة و{} دقيقة", hours, minutes)
        } else if minutes > 0 {
            format!("{} دقيقة", minutes)
        } else {
            format!("{} ثانية", seconds)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub path: String,
    pub size: FileSize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Movie {
    pub id: MediaId,
    pub title: String,
    pub poster: String,
    pub description: Option<String>,
    pub file: MediaFile,
    pub duration: DurationSeconds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Media {
    Movie(Movie),
    Series(Series),
}

// ── Helper methods ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaType {
    Movie,
    Series,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => write!(f, "movie"),
            MediaType::Series => write!(f, "series"),
        }
    }
}

impl TryFrom<&str> for MediaType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "movie" => Ok(MediaType::Movie),
            "series" => Ok(MediaType::Series),
            _ => Err("Media type must be 'movie' or 'series'"),
        }
    }
}

impl Media {
    fn kind(&self) -> MediaType {
        match self {
            Media::Movie(_) => MediaType::Movie,
            Media::Series(_) => MediaType::Series,
        }
    }
    pub fn title(&self) -> &str {
        match self {
            Media::Movie(m) => &m.title,
            Media::Series(s) => &s.title,
        }
    }
    pub fn poster(&self) -> &str {
        match self {
            Media::Movie(m) => &m.poster,
            Media::Series(s) => &s.poster,
        }
    }
    pub fn description(&self) -> Option<&str> {
        match self {
            Media::Movie(m) => m.description.as_deref(),
            Media::Series(s) => s.description.as_deref(),
        }
    }
    pub fn duration_display(&self) -> String {
        match self {
            Media::Movie(m) => m.duration.human_readable(),
            Media::Series(s) => format!("{} مواسم", s.season_count),
        }
    }
    pub fn size_display(&self) -> String {
        match self {
            Media::Movie(m) => m.file.size.human_readable(),
            Media::Series(s) => format!("{} مواسم", s.season_count),
        }
    }
    pub fn id(&self) -> i64 {
        match self {
            Media::Movie(m) => m.id.0,
            Media::Series(s) => s.id.0,
        }
    }
    pub fn file_path(&self) -> Option<&str> {
        match self {
            Media::Movie(m) => Some(&m.file.path),
            Media::Series(_) => None,
        }
    }
}

// ── CLIENT UPLOAD (unchanged) ─────────────────────────────────────────────
#[derive(Clone, Debug)]
pub struct EpUpload {
    pub id: u32,
    pub file: web_sys::File,
    pub title: String,
}

// ── MOCK DATA ────────────────────────────────────────────────────────────

const TEST_VIDEO: &str = "https://www.w3schools.com/html/mov_bbb.mp4";

fn fake_media_file() -> MediaFile {
    MediaFile {
        path: TEST_VIDEO.into(),
        size: FileSize(2_100_000_000), // ~2.1 GB
    }
}

fn fake_duration(seconds: u64) -> DurationSeconds {
    DurationSeconds(seconds)
}

fn mock_movies() -> Vec<Media> {
    vec![
        Media::Movie(Movie {
            id: MediaId(1),
            title: "Inception".into(),
            poster: "https://picsum.photos/seed/inception/300/450".into(),
            description: Some("لص يسرق أسرار الشركات من خلال تقنية مشاركة الأحلام.".into()),
            file: fake_media_file(),
            duration: fake_duration(8880), // 2h28m
        }),
        Media::Movie(Movie {
            id: MediaId(2),
            title: "The Matrix".into(),
            poster: "https://picsum.photos/seed/matrix/300/450".into(),
            description: Some("هاكر كمبيوتر يكتشف حقيقة الواقع.".into()),
            file: fake_media_file(),
            duration: fake_duration(8160),
        }),
        Media::Movie(Movie {
            id: MediaId(3),
            title: "Interstellar".into(),
            poster: "https://picsum.photos/seed/interstellar/300/450".into(),
            description: Some("فريق من المستكشفين يسافرون عبر ثقب دودي في الفضاء.".into()),
            file: fake_media_file(),
            duration: fake_duration(10140),
        }),
        Media::Movie(Movie {
            id: MediaId(4),
            title: "The Dark Knight".into(),
            poster: "https://picsum.photos/seed/darkknight/300/450".into(),
            description: Some("عندما يهدد الجوكر مدينة غوثام بالدمار.".into()),
            file: fake_media_file(),
            duration: fake_duration(9120),
        }),
        Media::Movie(Movie {
            id: MediaId(5),
            title: "Pulp Fiction".into(),
            poster: "https://picsum.photos/seed/pulpfiction/300/450".into(),
            description: Some("تتشابك حياة اثنين من القتلة وملاكم وزوجين من اللصوص.".into()),
            file: fake_media_file(),
            duration: fake_duration(9240),
        }),
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
async fn fetch_all_media() -> Result<Vec<Media>, ServerFnError> {
    delay(300).await;
    let mut all = mock_movies();
    all.extend(mock_series().into_iter().map(Media::Series));
    Ok(all)
}

#[server]
async fn fetch_media_detail(media_type: String, id: i64) -> Result<Media, ServerFnError> {
    delay(200).await;
    let list = match media_type.as_str() {
        "movie" => mock_movies(),
        "series" => mock_series()
            .into_iter()
            .map(|x| Media::Series(x))
            .collect(),
        _ => return Err(ServerFnError::new("not found")),
    };
    list.into_iter()
        .find(|m| m.id() == id)
        .ok_or(ServerFnError::new("not found"))
}

// ── ICONS ──────────────────────────────────────────────────────────────────

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

// ── LAYOUT ────────────────────────────────────────────────────────────────

#[component]
fn Layout() -> impl IntoView {
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

// ── MEDIA CARD ─────────────────────────────────────────────────────────

#[component]
fn MediaCard(item: Media) -> impl IntoView {
    let navigate = use_navigate();
    let kind = item.kind();
    let href = format!("/detail/{}/{}", kind, item.id());
    let href1 = href.clone();
    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        navigate(&href1, Default::default());
    };
    view! {
        <a href=href.clone()
            class="group relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03] hover:-translate-y-2"
            on:click=on_click>
            <MediaCardImage item=item.clone()/>
            <MediaCardInfo item=item.clone()/>
        </a>
    }
}

#[component]
fn MediaCardImage(item: Media) -> impl IntoView {
    let poster = item.poster().to_string();
    let title = item.title().to_string();
    let duration_display = item.duration_display();
    view! {
        <div class="aspect-[2/3] relative overflow-hidden">
            <img src=poster alt=title.clone()
                class="w-full h-full object-cover transition-transform duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] group-hover:scale-110"
                loading="lazy" on:error=|_| {} />
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                    <div class="flex items-center gap-2 mt-1 text-gray-300 text-sm">
                        <span class="flex items-center"><ClockIcon/>{duration_display}</span>
                    </div>
                </div>
            </div>
            <MediaTypeBadge kind=item.kind()/>
        </div>
    }
}

#[component]
fn MediaTypeBadge(kind: MediaType) -> impl IntoView {
    let icon = match kind {
        MediaType::Movie => Either::Left(MovieIcon()),
        MediaType::Series => Either::Right(SeriesIcon()),
    };
    let name = match kind {
        MediaType::Movie => "فيلم",
        MediaType::Series => "مسلسل",
    };
    view! {
        <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
            {icon}
            {name}
        </div>
    }
}

#[component]
fn MediaCardInfo(item: Media) -> impl IntoView {
    let title = item.title().to_string();
    let size = item.size_display();
    view! {
        <div
            class="p-4 flex flex-col gap-1"
        >
            <h3
                class="text-white font-semibold truncate text-sm"
            >
                {title}
            </h3>
            <h4
                class="text-white font-semibold truncate text-sm"
            >
                {size}
            </h4>
            <div
                class="flex items-center justify-between text-gray-500 text-xs"
            >
                <span
                    class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity"
                >
                    "← التفاصيل"
                </span>
            </div>
        </div>
    }
}

// ── DETAIL PAGE ────────────────────────────────────────────────────────

#[component]
fn DetailPoster(poster: String, title: String) -> impl IntoView {
    view! {
        <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
            <img src=poster class="w-full rounded-2xl shadow-2xl border border-white/10" alt=title />
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
        <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
            {media_icon}
            {name}
        </div>
    }
}

#[component]
fn DetailInfo(data: Media) -> impl IntoView {
    let title = data.title().to_string();
    let duration = data.duration_display();
    let size = data.size_display();
    let description = data.description().unwrap_or("لا يوجد وصف متاح.").to_string();
    let download = match &data {
        Media::Movie(m) => Some(view! {
            <a href=m.file.path.clone()
                class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm">
                <DownloadIcon/> "تحميل"
            </a>
        }),
        _ => None,
    };
    view! {
        <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">{title}</h1>
        <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
            <span class="flex items-center gap-1"><ClockIcon/>{duration}</span>
            <span>{size}</span>
        </div>
        <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{description}</p>
        <div class="mt-6 flex gap-3">{download}</div>
    }
}

#[component]
fn Detail() -> impl IntoView {
    let params = use_params_map();
    let media_type_str =
        move || params.with(|p| p.get("kind").map(|s| s.to_string()).unwrap_or_default());
    let id = move || params.with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0));

    let detail = Resource::new(
        move || (media_type_str(), id()),
        |(t, i): (String, i64)| async move { fetch_media_detail(t, i).await },
    );

    let selected_season = RwSignal::new(1u32);
    let episodes_resource = Resource::new(
        move || (id(), selected_season.get()),
        |(series_id, season): (i64, u32)| async move { fetch_season(series_id, season).await },
    );

    let selected_episode = RwSignal::new(None::<Episode>);

    Effect::new(move || {
        if let Some(Ok(season)) = episodes_resource.get() {
            let eps = season.episodes.clone();
            if !eps.is_empty() {
                selected_episode.set(Some(eps[0].clone()));
            } else {
                selected_episode.set(None);
            }
        }
    });

    let video_src = Memo::new(move |_| {
        if let Some(Ok(data)) = detail.get() {
            match data {
                Media::Movie(m) => m.file.path.clone(),
                Media::Series(_) => selected_episode
                    .get()
                    .map(|ep| ep.file.path)
                    .unwrap_or_default(),
            }
        } else {
            String::new()
        }
    });

    let fallback = || {
        view! { <div class="min-h-screen flex items-center justify-center text-white text-lg">"جارٍ التحميل..."</div> }
    };

    view! {
        <Suspense fallback=fallback>
            {move || detail.get().and_then(|x| x.ok()).map(|data| view! {
                <div class="relative min-h-screen bg-black text-white overflow-hidden">
                    <div class="absolute inset-0">
                        <img src=data.poster().to_string()
                             class="w-full h-full object-cover scale-110 blur-3xl opacity-20" alt="" />
                        <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
                    </div>
                    <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                        <DetailBody
                            data=data
                            video_src=video_src
                            selected_episode=selected_episode
                            selected_season=selected_season
                            episodes_resource=episodes_resource
                        />
                    </div>
                </div>
            })}
        </Suspense>
    }
}

#[component]
fn DetailBody(
    data: Media,
    video_src: Memo<String>,
    selected_episode: RwSignal<Option<Episode>>,
    selected_season: RwSignal<u32>,
    episodes_resource: Resource<Result<Season, ServerFnError>>,
) -> impl IntoView {
    let is_series = matches!(data, Media::Series(_));
    let series_summaries = if let Media::Series(ref s) = data {
        s.season_summaries.clone()
    } else {
        vec![]
    };

    view! {
        <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <DetailPoster poster=data.poster().to_string() title=data.title().to_string() />
            <div class="flex-1 w-full">
                <DetailMetaBadge media_type=data.kind() />
                <DetailInfo data=data.clone() />
            </div>
        </div>

        {move || (!video_src.get().is_empty()).then_some(view! {
            <div class="mt-10">
                <VideoPlayer src=Signal::derive(move || video_src.get()) title=data.title().to_string() />
            </div>
        })}

        {move || if is_series {
            Some(view! {
                <div class="mt-10">
                    <SeasonSelector summaries=series_summaries.clone() selected_season=selected_season />
                    <Suspense fallback=|| view! { <p class="text-gray-400">جارٍ تحميل الحلقات...</p> }>
                        {move || episodes_resource.get().and_then(|res| res.ok()).map(|season| {
                            view! {
                                <EpisodeSelector
                                    episodes=season.episodes.clone()
                                    selected_episode=selected_episode
                                />
                            }
                        })}
                    </Suspense>
                </div>
            })
        } else {
            None
        }}
    }
}

// ── HOME PAGE ──────────────────────────────────────────────────────────

// ── MOVIES / SERIES PAGES ──────────────────────────────────────────────

#[component]
fn MediaPageHeader(title: String, icon: impl IntoView) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 mb-6 md:mb-8">
            <div class="p-3 bg-cyan-400/10 rounded-2xl text-cyan-400">{icon}</div>
            <div>
                <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">{title.clone()}</h1>
                <p class="text-gray-400 text-sm md:text-base mt-0.5">"تصفح مجموعة "{title}"ك"</p>
            </div>
        </div>
    }
}

#[component]
fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse rounded-2xl bg-[#1a1a24]/60 border border-white/5 overflow-hidden shadow-xl">
            <div class="aspect-[2/3] bg-gradient-to-b from-[#2a2a3a] to-[#1a1a24]"></div>
            <div class="p-4 space-y-2">
                <div class="h-3 bg-[#2a2a3a] rounded w-3/4"></div>
                <div class="h-2 bg-[#2a2a3a] rounded w-1/2"></div>
            </div>
        </div>
    }
}

#[component]
fn CardsLoading() -> impl IntoView {
    let cards = (0..8).map(|_| CardSkeleton()).collect_view();
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
            {cards}
        </div>
    }
}

#[component]
fn Movies() -> impl IntoView {
    let movies = Resource::new(|| (), |_| async move { fetch_movies().await });
    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <MediaPageHeader title="أفلام".to_string() icon=MovieIcon()/>
            <Suspense fallback=CardsLoading>
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                    <For each={move || movies.get().and_then(|x| x.ok()).unwrap_or_default()} key=|m| m.id() let:item>
                        <MediaCard item=item.clone()/>
                    </For>
                </div>
            </Suspense>
        </div>
    }
}

// ── SEARCH PAGE ────────────────────────────────────────────────────────

#[component]
fn SearchHeaderResults() -> impl IntoView {
    let query_map = use_query_map();
    let q = query_map.with(|m| m.get("q").map(|s| s.to_string()).unwrap_or_default());
    view! {
        <p class="text-gray-400 text-sm sm:text-base">
            "نتائج البحث عن" <span class="text-white font-semibold">{format!("\"{}\"", q)}</span>
        </p>
    }
}

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
    view! {
        <div class="mb-6 md:mb-8">
            <h1 class="text-3xl sm:text-4xl font-black text-white mb-1">"نتائج البحث"</h1>
            {move || if query().is_empty() {
                Either::Left(view! { <p class="text-gray-400 text-sm sm:text-base">"أدخل كلمة بحث للعثور على الوسائط."</p> })
            } else {
                Either::Right(SearchHeaderResults())
            }}
        </div>
    }
}

#[component]
fn NoSearchResults() -> impl IntoView {
    view! { <div class="text-center py-16 text-gray-400 text-sm sm:text-base">"لا يوجد وسائط تطابق بحثك."</div> }
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
                    .filter(|item| item.title().to_lowercase().contains(&q))
                    .collect()
            })
            .unwrap_or_default()
    });
    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <SearchHeader/>
            <Suspense fallback=CardsLoading>
                {move || if results.get().is_empty() {
                    Either::Left(NoSearchResults())
                } else {
                    Either::Right(view! {
                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                            <For each={move || results.get()} key=|m| m.id() let:item>
                                <MediaCard item=item.clone()/>
                            </For>
                        </div>
                    })
                }}
            </Suspense>
        </div>
    }
}

// ── SETTINGS ───────────────────────────────────────────────────────────

#[component]
fn Settings() -> impl IntoView {
    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
            <div class="text-center text-white">
                <h1 class="text-4xl font-black mb-4">الإعدادات</h1>
                <p class="text-gray-400">سيتم إضافة صفحة الإعدادات قريباً.</p>
            </div>
        </div>
    }
}

// ── SHELL & APP ────────────────────────────────────────────────────────

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ar" dir="rtl">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
fn AppLink(
    href: impl Into<String>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let href: String = href.into();
    let navigate = use_navigate();
    let href_clone = href.clone();
    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        navigate(&href_clone, Default::default());
    };
    view! {
        <a href=href on:click=on_click class=class>
            {children()}
        </a>
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
                    <Route path=path!("/") view={Lazy::<HomePage>::new()}/>
                    <Route path=path!("/movies") view=Movies/>
                    <Route path=path!("/series") view={Lazy::<SeriesPage>::new()}/>
                    <Route path=path!("/upload") view={Lazy::<UploadPage>::new()}/>
                    <Route path=path!("/search") view=Search/>
                    <Route path=path!("/settings") view=Settings/>
                    <Route path=path!("/detail/series/:id") view={Lazy::<SeriesDetailPage>::new()}/>
                    <Route path=path!("/detail/movie/:id") view=Detail/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
