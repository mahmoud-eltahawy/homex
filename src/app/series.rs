#[cfg(feature = "ssr")]
use crate::app::delay;
use crate::app::{
    fake_duration, fake_media_file, video_player::VideoPlayer, CardsLoading, ClockIcon,
    DurationSeconds, Media, MediaCard, MediaFile, MediaId, MediaPageHeader, SeriesIcon,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::{hooks::use_params_map, lazy_route, LazyRoute};
use serde::{Deserialize, Serialize};
use web_sys::HtmlSelectElement;

pub struct SeriesPage {
    data: Resource<Result<Vec<Series>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for SeriesPage {
    fn data() -> Self {
        Self {
            data: Resource::new(|| (), |_| fetch_series()),
        }
    }

    fn view(this: Self) -> AnyView {
        let series = this.data;
        view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <MediaPageHeader title="مسلسلات".to_string() icon=SeriesIcon()/>
            <Suspense fallback=CardsLoading>
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                    <For each={move || series.get().and_then(|x| x.ok()).unwrap_or_default()} key=|m| m.id let:item>
                        <MediaCard item=Media::Series(item.clone())/>
                    </For>
                </div>
            </Suspense>
        </div>
        }
        .into_any()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub season: u32,
    pub episode: u32,
    pub file: MediaFile,
    pub duration: DurationSeconds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub id: MediaId,
    pub title: String,
    pub poster: String,
    pub description: Option<String>,
    pub start_year: Option<u32>,
    pub season_count: u32,
    pub season_summaries: Vec<SeasonSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonSummary {
    pub season_number: u32,
    pub episode_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Season {
    pub season_number: u32,
    pub episodes: Vec<Episode>,
}

pub fn mock_series() -> Vec<Series> {
    vec![
        Series {
            id: MediaId(101),
            title: "Breaking Bad".into(),
            poster: "https://picsum.photos/seed/breakingbad/300/450".into(),
            description: Some("مدرس كيمياء يتحول إلى تاجر مخدرات.".into()),
            start_year: Some(2008),
            season_count: 5,
            season_summaries: vec![
                SeasonSummary {
                    season_number: 1,
                    episode_count: 3,
                },
                SeasonSummary {
                    season_number: 2,
                    episode_count: 2,
                },
            ],
        },
        Series {
            id: MediaId(102),
            title: "Stranger Things".into(),
            poster: "https://picsum.photos/seed/strangerthings/300/450".into(),
            description: Some("مجموعة من الأطفال يكشفون أسرارًا خارقة في بلدتهم.".into()),
            start_year: Some(2016),
            season_count: 4,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 2,
            }],
        },
        Series {
            id: MediaId(103),
            title: "The Crown".into(),
            poster: "https://picsum.photos/seed/thecrown/300/450".into(),
            description: Some("عهد الملكة إليزابيث الثانية.".into()),
            start_year: Some(2016),
            season_count: 4,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 1,
            }],
        },
        Series {
            id: MediaId(104),
            title: "Game of Thrones".into(),
            poster: "https://picsum.photos/seed/got/300/450".into(),
            description: Some("عائلات نبيلة تتصارع على السيطرة على ويستروس.".into()),
            start_year: Some(2011),
            season_count: 8,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 2,
            }],
        },
    ]
}

fn mock_season(series_id: i64, season_number: u32) -> Option<Season> {
    let episodes = match (series_id, season_number) {
        (101, 1) => vec![ep(1011, 1, 1), ep(1012, 1, 2), ep(1013, 1, 3)],
        (101, 2) => vec![ep(1014, 2, 1), ep(1015, 2, 2)],
        (102, 1) => vec![ep(1021, 1, 1), ep(1022, 1, 2)],
        (103, 1) => vec![ep(1031, 1, 1)],
        (104, 1) => vec![ep(1041, 1, 1), ep(1042, 1, 2)],
        _ => return None,
    };
    Some(Season {
        season_number,
        episodes,
    })
}

#[server]
pub async fn fetch_season(series_id: i64, season_number: u32) -> Result<Season, ServerFnError> {
    delay(200).await;
    mock_season(series_id, season_number).ok_or(ServerFnError::new("season not found"))
}

#[server]
async fn fetch_series() -> Result<Vec<Series>, ServerFnError> {
    delay(300).await;
    Ok(mock_series())
}

pub fn ep(id: i64, season: u32, episode: u32) -> Episode {
    Episode {
        id,
        season,
        episode,
        file: fake_media_file(),
        duration: fake_duration(3600), // 1 hour per episode
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
fn EpisodeCard(ep: Episode, selected_episode: RwSignal<Option<Episode>>) -> impl IntoView {
    let is_selected = move || {
        selected_episode
            .get()
            .as_ref()
            .map_or(false, |s| s.id == ep.id)
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

pub struct SeriesDetailPage {
    series: Resource<Result<Series, ServerFnError>>,
    episodes: Resource<Result<Season, ServerFnError>>,
    selected_season: RwSignal<u32>,
}

#[lazy_route]
impl LazyRoute for SeriesDetailPage {
    fn data() -> Self {
        let params = use_params_map();
        let id =
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0));

        let selected_season = RwSignal::new(1);

        Self {
            series: Resource::new(move || id(), |id| fetch_series_detail(id)),
            episodes: Resource::new(
                move || (id(), selected_season.get()),
                |(series_id, season)| fetch_season(series_id, season),
            ),
            selected_season,
        }
    }

    fn view(this: Self) -> AnyView {
        let selected_episode = RwSignal::new(None::<Episode>);

        Effect::new(move || {
            if let Some(Ok(season)) = this.episodes.get() {
                let eps = season.episodes.clone();
                selected_episode.set(if eps.is_empty() {
                    None
                } else {
                    Some(eps[0].clone())
                });
            }
        });

        let video_src = Memo::new(move |_| {
            selected_episode
                .get()
                .map(|ep| ep.file.path)
                .unwrap_or_default()
        });

        let fallback =
            || view! { <div class="py-20 text-center text-white">"جارٍ التحميل..."</div> };

        view! {
            <Suspense fallback=fallback>
                {move || match this.series.get() {
                    None => view! { <div class="py-20 text-center text-white">"جارٍ التحميل..."</div> }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="py-16 text-center">
                            <div class="text-red-400 text-lg font-bold mb-2">"حدث خطأ"</div>
                            <p class="text-gray-400 text-sm mb-4">{e.to_string()}</p>
                            <button
                                on:click=move |_| this.series.refetch()
                                class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-white text-sm transition"
                            >
                                "إعادة المحاولة"
                            </button>
                        </div>
                    }.into_any(),
                    Some(Ok(series)) => {
                        let poster = series.poster.clone();
                        let title = series.title.clone();
                        let year = series.start_year;
                        let description = series.description.clone().unwrap_or_else(|| "لا يوجد وصف متاح.".to_string());
                        let summaries = series.season_summaries.clone();

                        view! {
                            <div class="relative min-h-screen bg-black text-white overflow-hidden">
                                <div class="absolute inset-0">
                                    <img src=poster.clone()
                                         class="w-full h-full object-cover scale-110 blur-3xl opacity-20" alt="" />
                                    <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
                                </div>
                                <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                                    // Poster + info
                                    <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
                                        <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
                                            <img src=poster class="w-full rounded-2xl shadow-2xl border border-white/10" alt=title.clone() />
                                        </div>
                                        <div class="flex-1 w-full">
                                            <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
                                                <SeriesIcon/> "مسلسل"
                                            </div>
                                            <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">{title.clone()}</h1>
                                            <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
                                                {year.map(|y| view! { <span>{y}</span> })}
                                                <span class="flex items-center gap-1"><ClockIcon/> {format!("{} مواسم", series.season_count)}</span>
                                                <span>{format!("{} مواسم", series.season_count)}</span>
                                            </div>
                                            <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{description}</p>
                                        </div>
                                    </div>

                                    // Video player (always visible if an episode is selected)
                                    {move || (!video_src.get().is_empty()).then_some(view! {
                                        <div class="mt-10">
                                            <VideoPlayer src=Signal::derive(move || video_src.get()) title=title.clone() />
                                        </div>
                                    })}

                                    // Season & episode selectors
                                    <div class="mt-10">
                                        <SeasonSelector summaries=summaries selected_season=this.selected_season />
                                        <Suspense fallback=|| view! { <p class="text-gray-400">جارٍ تحميل الحلقات...</p> }>
                                            {move || match this.episodes.get() {
                                                None => view! { <p class="text-gray-400">جارٍ تحميل الحلقات...</p> }.into_any(),
                                                Some(Err(e)) => view! {
                                                    <div class="py-8 text-center">
                                                        <div class="text-red-400 text-sm font-bold mb-2">"خطأ في تحميل الحلقات"</div>
                                                        <p class="text-gray-500 text-xs mb-3">{e.to_string()}</p>
                                                        <button
                                                            on:click=move |_| this.episodes.refetch()
                                                            class="px-3 py-1 rounded-lg bg-white/10 hover:bg-white/20 text-white text-xs transition"
                                                        >
                                                            "إعادة المحاولة"
                                                        </button>
                                                    </div>
                                                }.into_any(),
                                                Some(Ok(season)) => view! {
                                                    <EpisodeSelector
                                                        episodes=season.episodes.clone()
                                                        selected_episode=selected_episode
                                                    />
                                                }.into_any(),
                                            }}
                                        </Suspense>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>
        }.into_any()
    }
}

#[server]
async fn fetch_series_detail(id: i64) -> Result<Series, ServerFnError> {
    delay(200).await;
    let list = mock_series();
    list.into_iter()
        .find(|m| m.id.0 == id)
        .ok_or(ServerFnError::new("not found"))
}
