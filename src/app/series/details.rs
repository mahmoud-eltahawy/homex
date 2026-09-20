use super::{fetch_season, fetch_series_detail};
use crate::app::{
    detail::{DetailHero, DetailShell, HeroBadge, HeroDescription, HeroMeta, HeroTitle, Poster},
    icons::{ClockIcon, SeriesIcon, SeriesPosterSvg},
    media_player::{MediaItem, MediaPlayer},
    model::{Season, SeasonSummary, Series},
    resource_view::ResourceView,
    route_params::use_u64_param,
};
use leptos::{either::Either, prelude::*};
use leptos_router::{LazyRoute, lazy_route};
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
        let id = use_u64_param("id");
        let selected_season = RwSignal::new(1);
        let series = Resource::new(id, fetch_series_detail);
        let episodes = Resource::new(
            move || {
                // Only fetch a season once we know which seasons exist.
                let series_id = id();
                let season = match series.get() {
                    Some(Ok(s)) => s
                        .season_summaries
                        .iter()
                        .any(|x| x.season_number == selected_season.get())
                        .then(|| selected_season.get()),
                    _ => None,
                };
                (series_id, season)
            },
            |(series_id, season)| async move {
                match season {
                    Some(s) => fetch_season(series_id, s).await,
                    None => Ok(Season {
                        season_number: 0,
                        episodes: vec![],
                    }),
                }
            },
        );

        Self {
            series,
            episodes,
            selected_season,
        }
    }

    fn view(this: Self) -> AnyView {
        let series = this.series;
        let episodes = this.episodes;
        let selected_season = this.selected_season;

        Effect::new(move |_| {
            let Some(Ok(s)) = series.get() else { return };
            let Some(first) = s.season_summaries.first() else {
                return;
            };
            let current = selected_season.get_untracked();
            if !s
                .season_summaries
                .iter()
                .any(|x| x.season_number == current)
            {
                selected_season.set(first.season_number);
            }
        });

        let adapter = move |s: Series| SeriesViewProps {
            series: s,
            episodes,
            selected_season,
        };
        view! {
            <ResourceView resource=series view_fn=SeriesView adapter=adapter />
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
    let season_count = series.season_count;

    let season_adapter = |season: Season| SeasonPlayerProps { season };

    view! {
        <DetailShell poster=poster.clone()>
            <DetailHero poster=view! {
                <Poster
                    src=poster.clone()
                    alt=title.clone()
                    class="w-full rounded-2xl shadow-2xl border border-white/10".to_string()
                    placeholder=view! { <SeriesPosterSvg/> }.into_any()
                />
            }>
                <HeroBadge label="مسلسل" icon=SeriesIcon()/>
                <HeroTitle title=title.clone()/>
                <HeroMeta>
                    <span class="flex items-center gap-1">
                        <ClockIcon/>
                        {format!("{} مواسم", season_count)}
                    </span>
                </HeroMeta>
                <HeroDescription text=description/>
            </DetailHero>
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
