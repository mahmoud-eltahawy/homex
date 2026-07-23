use crate::app::{
    home::HomePage,
    icons::{ClockIcon, DownloadIcon, MovieIcon, SeriesIcon},
    layout::Layout,
    series::{
        details::SeriesDetailPage,
        fetch_season,
        listing::{Episode, EpisodeSelector, Season, SeasonSelector, Series, SeriesPage},
    },
    settings::SettingsPage,
    upload::UploadPage,
    video_player::VideoPlayer,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    hooks::{use_navigate, use_params_map, use_query_map},
    path, Lazy,
};
use serde::{Deserialize, Serialize};
use web_sys::MouseEvent;

mod home;
mod icons;
mod layout;
mod resource_view;
mod series;
mod settings;
mod upload;
mod video_player;
//TODO : DELETE this
#[cfg(feature = "ssr")]
mod mockary;

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

#[cfg(feature = "ssr")]
async fn delay(ms: i32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

#[server]
async fn fetch_movies() -> Result<Vec<Movie>, ServerFnError> {
    delay(300).await;
    Ok(mockary::mock_movies())
}

#[server]
async fn fetch_all_media() -> Result<Vec<Media>, ServerFnError> {
    delay(300).await;
    let mut all = mockary::mock_movies()
        .into_iter()
        .map(Media::Movie)
        .collect::<Vec<_>>();
    all.extend(mockary::mock_series().into_iter().map(Media::Series));
    Ok(all)
}

#[server]
async fn fetch_media_detail(media_type: String, id: i64) -> Result<Media, ServerFnError> {
    delay(200).await;
    let list: Vec<_> = match media_type.as_str() {
        "movie" => mockary::mock_movies()
            .into_iter()
            .map(|x| Media::Movie(x))
            .collect(),
        "series" => mockary::mock_series()
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
    let cards = (0..5).map(|_| CardSkeleton()).collect_view();
    view! {
        <div
            class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6 my-15"
        >
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
                    <For each={move || movies.get().and_then(|x| x.ok()).unwrap_or_default()} key=|m| m.id let:item>
                        <MediaCard item=Media::Movie(item.clone())/>
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
                    <Route path=path!("/settings") view={Lazy::<SettingsPage>::new()}/>
                    <Route path=path!("/detail/series/:id") view={Lazy::<SeriesDetailPage>::new()}/>
                    <Route path=path!("/detail/movie/:id") view=Detail/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
