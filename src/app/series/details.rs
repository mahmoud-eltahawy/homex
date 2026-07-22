use crate::app::{
    series::listing::{EpisodeSelector, SeasonSelector},
    video_player::VideoPlayer,
    ClockIcon, SeriesIcon,
};

use super::{
    fetch_season, fetch_series_detail,
    listing::{Episode, Season, Series},
};
use leptos::prelude::*;
use leptos_router::{hooks::use_params_map, lazy_route, LazyRoute};

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
