use crate::app::{
    icons::SeriesIcon,
    model::{Episode, Media, SeasonSummary, Series},
    series::fetch_series,
    CardsLoading, MediaCard, MediaPageHeader,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::{lazy_route, LazyRoute};
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
