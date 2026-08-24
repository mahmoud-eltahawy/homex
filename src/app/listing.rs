use crate::app::icons::{NextIcon, PrevIcon};
use crate::app::{
    audio::{fetch_audio_groups, fetch_audio_groups_count},
    common::{CardsLoading, MediaPageHeader},
    model::{AudioGroup, Movie, Series},
    movies::{fetch_movies, fetch_movies_count},
    resource_view::ResourceView,
    series::{fetch_series, fetch_series_count},
    view_schema::{Card, CardsList},
};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
use serde::{de::DeserializeOwned, Serialize};

const LISTING_PAGE_SIZE: usize = 18;

pub struct ListingPage<C>
where
    C: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    pub offset: RwSignal<usize>,
    pub data: Resource<Result<Vec<C>, ServerFnError>>,
    pub count: Resource<Result<usize, ServerFnError>>,
}

impl<C> ListingPage<C>
where
    C: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn view(self) -> impl IntoView {
        let title = C::media_type().ar_title();
        let context = format!("تحميل {} ...", title);
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <MediaPageHeader title=title.to_string() icon=C::icon()/>
                <ResourceView
                    resource=self.data
                    view_fn=CardsList::cards_list
                    fallback=CardsLoading
                    adapter=|x| x
                    context=context
                />
                <PaginationControls offset=self.offset count=self.count/>
            </div>
        }
    }
}

pub type SeriesListingPage = ListingPage<Series>;

#[lazy_route]
impl LazyRoute for SeriesListingPage {
    fn data() -> Self {
        let offset = RwSignal::new(0);
        let data = Resource::new(
            move || offset.get(),
            |offset| fetch_series(offset, LISTING_PAGE_SIZE),
        );
        let count = Resource::new(|| (), |_| fetch_series_count());
        Self {
            data,
            offset,
            count,
        }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type MovieListingPage = ListingPage<Movie>;

#[lazy_route]
impl LazyRoute for MovieListingPage {
    fn data() -> Self {
        let offset = RwSignal::new(0);
        let data = Resource::new(
            move || offset.get(),
            |offset| fetch_movies(offset, LISTING_PAGE_SIZE),
        );
        let count = Resource::new(|| (), |_| fetch_movies_count());
        Self {
            data,
            offset,
            count,
        }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type AudioGroupListingPage = ListingPage<AudioGroup>;

#[lazy_route]
impl LazyRoute for AudioGroupListingPage {
    fn data() -> Self {
        let offset = RwSignal::new(0);
        let data = Resource::new(
            move || offset.get(),
            |offset| fetch_audio_groups(offset, LISTING_PAGE_SIZE),
        );
        let count = Resource::new(|| (), |_| fetch_audio_groups_count());
        Self {
            data,
            offset,
            count,
        }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

#[component]
pub fn PaginationControls(
    offset: RwSignal<usize>,
    count: Resource<Result<usize, ServerFnError>>,
    #[prop(default = 5)] window_size: usize,
) -> impl IntoView {
    let total_pages = move || {
        count
            .get()
            .transpose()
            .ok()
            .flatten()
            .map(|total| (total.saturating_add(LISTING_PAGE_SIZE - 1)) / LISTING_PAGE_SIZE)
    };

    let current_page = move || offset.get() / LISTING_PAGE_SIZE + 1;

    let window_start = RwSignal::new(1usize);

    Effect::new(move || {
        let total = total_pages().unwrap_or(1).max(1);
        let current = current_page().min(total);

        let mut new_start = window_start.get();

        if current < new_start {
            new_start = current;
        } else if current >= new_start + window_size {
            new_start = current - window_size + 1;
        }

        new_start = new_start.clamp(1, total.saturating_sub(window_size - 1).max(1));

        if new_start != window_start.get() {
            window_start.set(new_start);
        }
    });

    let can_shift_left = move || window_start.get() > 1;
    let can_shift_right = move || {
        let total = total_pages().unwrap_or(1).max(1);
        window_start.get() + window_size - 1 < total
    };

    let shift_left = move |_| {
        let new_start = window_start.get().saturating_sub(window_size).max(1);
        window_start.set(new_start);
    };

    let shift_right = move |_| {
        let total = total_pages().unwrap_or(1).max(1);
        let max_start = total.saturating_sub(window_size - 1).max(1);
        let new_start = (window_start.get() + window_size).min(max_start);
        window_start.set(new_start);
    };

    let go_to_page = move |page: usize| {
        offset.set((page - 1) * LISTING_PAGE_SIZE);
    };

    let pages = move || {
        let total = total_pages().unwrap_or(0);
        if total == 0 {
            return Vec::new();
        }
        let start = window_start.get().min(total);
        let end = (start + window_size - 1).min(total);
        (start..=end).collect::<Vec<_>>()
    };

    view! {
        <Transition>
        <div class="flex items-center justify-center gap-2 mt-8">
            <button
                on:click=shift_left
                disabled=move || !can_shift_left()
                class="flex h-9 w-9 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 disabled:cursor-not-allowed disabled:opacity-30"
                aria-label="Previous pages"
            >
                <PrevIcon/>
            </button>

            <div class="flex items-center gap-1 rounded-full border border-white/10 bg-white/5 p-1 backdrop-blur">
                <For
                    each=pages
                    key=|page| *page
                    let:page
                >
                    <button
                        on:click=move |_| go_to_page(page)
                        class=move || format!(
                            "flex h-8 min-w-8 items-center justify-center rounded-full px-2 text-sm font-medium transition {}",
                            if page == current_page() {
                                "bg-cyan-500/20 text-cyan-400"
                            } else {
                                "text-slate-300 hover:bg-white/10 hover:text-white"
                            }
                        )
                    >
                        {page}
                    </button>
                </For>
            </div>

            <button
                on:click=shift_right
                disabled=move || !can_shift_right()
                class="flex h-9 w-9 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 disabled:cursor-not-allowed disabled:opacity-30"
                aria-label="Next pages"
            >
                <NextIcon/>
            </button>
        </div>
        </Transition>
    }
}
