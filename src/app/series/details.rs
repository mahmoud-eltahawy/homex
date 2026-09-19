use super::{fetch_season, fetch_series_detail};
use crate::app::{
    detail::{DetailShell, Poster},
    icons::{ClockIcon, SeriesIcon, SeriesPosterSvg},
    media_player::{MediaItem, MediaPlayer},
    model::{Season, SeasonSummary, Series},
    resource_view::ResourceView,
};
use leptos::{either::Either, prelude::*};
use leptos_router::{LazyRoute, hooks::use_params_map, lazy_route};
use web_sys::HtmlSelectElement;
use web_sys::wasm_bindgen::JsCast;

pub struct SeriesDetailPage {
    pub series: Resource<Result<Series, ServerFnError>>,
    pub episodes: Resource<Result<Season, ServerFnError>>,
    pub selected_season: RwSignal<u32>,
}

#[lazy_route]
impl LazyRoute for SeriesDetailPage {
    fn data() -> Self {
        let params = use_params_map();
        let id =
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0));

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
            <ResourceView resource=this.series view_fn=SeriesView adapter=adapter />
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
    let poster = series.poster.clone();
    let title = series.title.clone();
    let description = series
        .description
        .clone()
        .unwrap_or_else(|| "لا يوجد وصف متاح.".to_string());
    let summaries = series.season_summaries.clone();

    let season_adapter = |season: Season| SeasonPlayerProps { season };

    let edit_href = format!("/series/detail/{}/edit", series.id);
    view! {
        <DetailShell poster=poster.clone() edit_href>
            <Info poster title season_count=series.season_count description/>
            <div class="mt-10">
                <SeasonSelector summaries selected_season/>
                <ResourceView
                    resource=episodes
                    view_fn=SeasonPlayer
                    adapter=season_adapter
                />
            </div>
        </DetailShell>
    }
}

#[component]
fn SeasonPlayer(season: Season) -> impl IntoView {
    let items: Vec<MediaItem> = season
        .episodes
        .iter()
        .map(|ep| {
            let title = format!("حلقة {}", ep.episode);
            let subtitle = format!("S{:02}E{:02}", ep.season, ep.episode);
            MediaItem::new(ep.id as u64, title, ep.file.path.clone()).with_subtitle(subtitle)
        })
        .collect();

    let has_items = !items.is_empty();
    let playlist_title = format!("الموسم {}", season.season_number);

    view! {
        <div class="mt-4">
            {if has_items {
                Either::Left(view! {
                    <MediaPlayer items=items playlist_title=playlist_title />
                })
            } else {
                Either::Right(view! {
                    <div class="py-8 text-center text-gray-500 text-sm">
                        "لا توجد حلقات في هذا الموسم."
                    </div>
                })
            }}
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
        <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
                <Poster
                    src=poster
                    alt=title.clone()
                    class="w-full rounded-2xl shadow-2xl border border-white/10".to_string()
                    placeholder=view! { <SeriesPosterSvg/> }.into_any()
                />
            </div>
            <div class="flex-1 w-full">
                <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
                    <SeriesIcon/> "مسلسل"
                </div>
                <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">
                    {title.clone()}
                </h1>
                <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
                    <span class="flex items-center gap-1">
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
pub fn SeasonSelector(
    summaries: Vec<SeasonSummary>,
    selected_season: RwSignal<u32>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 mb-4">
            <span class="text-gray-300 text-sm">اختر الموسم:</span>
            <select
                class="bg-white/10 backdrop-blur-md text-white rounded-xl py-1.5 px-3 focus:outline-none focus:ring-1 focus:ring-cyan-400"
                prop:value=move || selected_season.get().to_string()
                on:change=move |ev| {
                    if let Some(sel) = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                        && let Ok(num) = sel.value().parse::<u32>()
                    {
                        selected_season.set(num);
                    }
                }
            >
                <For each={move || summaries.clone()} key=|s| s.season_number let:sum>
                    <option value={sum.season_number.to_string()}>
                        {format!("الموسم {} ({} حلقات)", sum.season_number, sum.episode_count)}
                    </option>
                </For>
            </select>
        </div>
    }
}
