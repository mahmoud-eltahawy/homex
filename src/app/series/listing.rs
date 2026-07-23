use crate::app::{
    fake_duration, fake_media_file, series::fetch_series, CardsLoading, DurationSeconds, Media,
    MediaCard, MediaFile, MediaId, MediaPageHeader, SeriesIcon,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::{lazy_route, LazyRoute};
use serde::{Deserialize, Serialize};
use web_sys::HtmlSelectElement;
pub struct SeriesPage {
    pub(crate) data: Resource<Result<Vec<Series>, ServerFnError>>,
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
            season_count: 8,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 2,
            }],
        },
    ]
}

pub fn mock_season(series_id: i64, season_number: u32) -> Option<Season> {
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
pub(crate) fn EpisodeCard(
    ep: Episode,
    selected_episode: RwSignal<Option<Episode>>,
) -> impl IntoView {
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
