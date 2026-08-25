use super::{fetch_season, fetch_series_detail};
use crate::app::{
    icons::{ClockIcon, SeriesIcon},
    model::{Episode, Season, SeasonSummary, Series},
    resource_view::ResourceView,
    video_player::VideoPlayer,
    view_schema::PosterView,
};
use leptos::prelude::*;
use leptos_router::{hooks::use_params_map, lazy_route, LazyRoute};
use web_sys::wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

pub struct SeriesDetailPage {
    pub series: Resource<Result<Series, ServerFnError>>,
    pub episodes: Resource<Result<Season, ServerFnError>>,
    pub selected_season: RwSignal<u32>,
}

#[lazy_route]
impl LazyRoute for SeriesDetailPage {
    fn data() -> Self {
        let params = use_params_map();
        let id = move || {
            params.with(|p| {
                p.get("id")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0)
            })
        };

        let selected_season = RwSignal::new(1);

        Self {
            series: Resource::new(id, fetch_series_detail),
            episodes: Resource::new(
                move || (id(), selected_season.get()),
                |(series_id, season)| fetch_season(series_id, season),
            ),
            selected_season,
        }
    }

    fn view(this: Self) -> AnyView {
        let adapter = move |series: Series| SeriesViewProps {
            series,
            episodes: this.episodes,
            selected_season: this.selected_season,
        };
        view! {
            <ResourceView
                resource=this.series
                view_fn=SeriesView
                adapter=adapter
                context="تحميل الحلقات"
            />
        }
        .into_any()
    }
}

#[component]
fn SeriesView(
    episodes: Resource<Result<Season, ServerFnError>>,
    series: Series,
    selected_season: RwSignal<u32>,
) -> impl IntoView {
    let selected_episode = RwSignal::new(None::<Episode>);
    Effect::new(move || {
        if let Some(Ok(season)) = episodes.get() {
            let eps = season.episodes.clone();
            selected_episode.set(if eps.is_empty() {
                None
            } else {
                Some(eps[0].clone())
            });
        }
    });

    let poster = series.poster.clone();
    let title = series.title.clone();
    let description = series
        .description
        .clone()
        .unwrap_or_else(|| "لا يوجد وصف متاح.".to_string());
    let summaries = series.season_summaries.clone();

    let video_src = Memo::new(move |_| {
        selected_episode
            .get()
            .map(|ep| ep.file.path)
            .unwrap_or_default()
    });

    let video_player = {
        let title = title.clone();
        move || {
            (!video_src.get().is_empty()).then_some(view! {
                <div class="mt-10">
                    <VideoPlayer
                        src=Signal::derive(move || video_src.get())
                        title=title.clone()
                    />
                </div>
            })
        }
    };

    view! {
        <div class="relative min-h-screen bg-black text-white overflow-hidden">
            <div class="absolute inset-0">
                {series.clone().poster()}
                <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
            </div>
            <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                <Info
                    poster
                    title
                    season_count=series.season_count
                    description
                />
                {video_player}

                <Selectors
                    selected_season
                    episodes
                    selected_episode
                    summaries
                />
            </div>
        </div>
    }
}

#[component]
fn Info(
    poster: Option<String>,
    title: String,
    season_count: u32,
    description: String,
) -> impl IntoView {
    view! {
    <div
        class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start"
    >
        <div
            class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0"
        >
            <img
                src=poster
                class="w-full rounded-2xl shadow-2xl border border-white/10"
                alt=title.clone()
            />
        </div>
        <div
            class="flex-1 w-full"
        >
            <div
                class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5"
                >
                    <SeriesIcon/>
                    "مسلسل"
            </div>
            <h1
                class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2"
            >
                {title.clone()}
            </h1>
            <div
                class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base"
            >
                <span
                    class="flex items-center gap-1"
                >
                    <ClockIcon/>
                    {format!("{} مواسم", season_count)}
                </span>
            </div>
            <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{description}</p>
        </div>
    </div>
    }
}

#[component]
fn Selectors(
    selected_season: RwSignal<u32>,
    selected_episode: RwSignal<Option<Episode>>,
    episodes: Resource<Result<Season, ServerFnError>>,
    summaries: Vec<SeasonSummary>,
) -> impl IntoView {
    let adapter = move |season: Season| EpisodeSelectorProps {
        episodes: season.episodes.clone(),
        selected_episode,
    };
    view! {
        <div class="mt-10">
            <SeasonSelector
                summaries=summaries
                selected_season=selected_season
            />
            <ResourceView
                resource=episodes
                view_fn=EpisodeSelector
                adapter=adapter
                context="تحميل تلخيصات المواسم"
            />
        </div>
    }
}

#[component]
pub fn SeasonSelector(
    summaries: Vec<SeasonSummary>,
    selected_season: RwSignal<u32>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 mt-6 mb-4">
            <span class="text-gray-300 text-sm">اختر الموسم:</span>
            <select
                class="bg-white/10 backdrop-blur-md text-white rounded-xl py-1.5 px-3 focus:outline-none focus:ring-1 focus:ring-cyan-400"
                prop:value=move || selected_season.get().to_string()
                on:change=move |ev| {
                    if let Some(sel) = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
                        if let Ok(num) = sel.value().parse::<u32>() {
                            selected_season.set(num);
                        }
                    }
                }
            >
                <For each={move || summaries.clone()} key=|s| s.season_number let:sum>
                    <option value={sum.season_number.to_string()} selected={sum.season_number == selected_season.get()}>
                        {format!("الموسم {} ({} حلقات)", sum.season_number, sum.episode_count)}
                    </option>
                </For>
            </select>
        </div>
    }
}

#[component]
pub fn EpisodeSelector(
    episodes: Vec<Episode>,
    selected_episode: RwSignal<Option<Episode>>,
) -> impl IntoView {
    view! {
        <div class="mt-6">
            <h2 class="text-xl sm:text-2xl font-bold text-white mb-4 flex items-center gap-2">
                <SeriesIcon/> " الحلقات"
            </h2>
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                <For each={move || episodes.clone()} key=|ep| ep.id let:ep>
                    <EpisodeCard ep=ep selected_episode=selected_episode/>
                </For>
            </div>
        </div>
    }
}

#[component]
pub fn EpisodeCard(ep: Episode, selected_episode: RwSignal<Option<Episode>>) -> impl IntoView {
    let is_selected = move || {
        selected_episode
            .get()
            .as_ref()
            .is_some_and(|s| s.id == ep.id)
    };
    let class = move || {
        format!(
            "p-3 rounded-xl border transition-all cursor-pointer backdrop-blur-sm {}",
            if is_selected() {
                "border-cyan-400 bg-cyan-400/10 shadow-lg shadow-cyan-400/10"
            } else {
                "border-white/10 bg-white/5 hover:bg-white/10 hover:border-white/20"
            }
        )
    };
    let label = format!("حلقة {}", ep.episode);
    let on_click = {
        let ep = ep.clone();
        move |_| selected_episode.set(Some(ep.clone()))
    };
    view! {
        <div class=class on:click=on_click>
            <div class="flex items-center gap-3">
                <span class="text-sm font-mono text-gray-400">
                    "S"{format!("{:02}", ep.season)}"E"{format!("{:02}", ep.episode)}
                </span>
                <span class="text-sm text-white truncate">{label}</span>
            </div>
        </div>
    }
}
