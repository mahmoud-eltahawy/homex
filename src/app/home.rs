use crate::app::{
    audio::{fetch_audio_groups, fetch_audio_groups_count},
    common::CardsLoading,
    icons::{NextIcon, PrevIcon},
    model::{AudioGroup, Movie, Series},
    movies::{fetch_movies, fetch_movies_count},
    resource_view::ResourceView,
    series::{fetch_series, fetch_series_count},
    view_schema::{Card, CardsList},
};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
use serde::{de::DeserializeOwned, Serialize};

pub struct HomePage {
    movies: MediaLoaderProps<Movie>,
    series: MediaLoaderProps<Series>,
    audio: MediaLoaderProps<AudioGroup>,
}

const MEDIA_LIST_SIZE: usize = 6;

#[lazy_route]
impl LazyRoute for HomePage {
    fn data() -> Self {
        let offset = RwSignal::new(0usize);
        let resource = Resource::new(
            move || offset.get(),
            async |offset| fetch_movies(offset, MEDIA_LIST_SIZE, None).await,
        );
        let count = Resource::new(|| (), async |_| fetch_movies_count().await);
        let movies = MediaLoaderProps {
            resource,
            offset,
            count,
        };

        let offset = RwSignal::new(0);
        let resource = Resource::new(
            move || offset.get(),
            async |offset| fetch_series(offset, MEDIA_LIST_SIZE, None).await,
        );
        let count = Resource::new(|| (), async |_| fetch_series_count().await);
        let series = MediaLoaderProps {
            resource,
            offset,
            count,
        };

        let offset = RwSignal::new(0);
        let resource = Resource::new(
            move || offset.get(),
            async |offset| fetch_audio_groups(offset, MEDIA_LIST_SIZE, None).await,
        );
        let count = Resource::new(|| (), async |_| fetch_audio_groups_count().await);
        let audio = MediaLoaderProps {
            resource,
            offset,
            count,
        };

        Self {
            movies,
            series,
            audio,
        }
    }

    fn view(this: Self) -> AnyView {
        let HomePage {
            movies,
            series,
            audio,
        } = this;
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <HomeHero/>
                {MediaLoader(movies)}
                {MediaLoader(series)}
                {MediaLoader(audio)}
            </div>
        }
        .into_any()
    }
}

#[component]
pub fn HomeHero() -> impl IntoView {
    view! {
        <div class="py-12 sm:py-16 md:py-20 lg:py-24 text-center">
            <h1 class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-black tracking-tight leading-[1.1]">
                <span class="bg-gradient-to-r from-cyan-200 via-blue-300 to-indigo-400 bg-clip-text text-transparent">"سينماك"</span>
                <br class="sm:hidden"/>
                <span class="text-white">" الشخصية"</span>
            </h1>
            <p class="text-gray-400 text-base sm:text-lg md:text-xl max-w-2xl mx-auto mt-4 leading-relaxed">
                "شاهد وحمّل مجموعتك من الأفلام والمسلسلات والمجموعات الصوتية من أي مكان في منزلك."
            </p>
        </div>
    }
}

#[component]
fn MediaSection<C>(
    items: Vec<C>,
    items_offset: RwSignal<usize>,
    items_count: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView
where
    C: Card,
{
    let media_type = C::media_type();
    view! {
        <section class="mb-12 md:mb-16">
            <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
                <h2 class="flex items-center gap-3 text-2xl font-black text-white sm:text-3xl md:text-4xl">
                    <span class="text-cyan-400">{C::icon()}</span>
                    {media_type.ar_title()}
                </h2>

                <MediaSectionNav items_offset items_count href={media_type.to_string()}/>
            </div>

            {items.cards_list()}
        </section>
    }
}

#[component]
fn MediaSectionNav(
    items_offset: RwSignal<usize>,
    items_count: Resource<Result<usize, ServerFnError>>,
    href: String,
) -> impl IntoView {
    let can_prev = move || items_offset.get() > 0;

    let get_items_count = move || items_count.get().transpose().ok().flatten();
    let can_next = move || {
        get_items_count()
            .map(|count| items_offset.get() < count.saturating_sub(MEDIA_LIST_SIZE))
            .unwrap_or(false)
    };

    let go_prev = move |_| {
        items_offset.update(|x| {
            if *x > 0 {
                *x -= 1;
            }
        });
    };

    let go_next = move |_| {
        items_offset.update(|x| {
            let max_offset = get_items_count()
                .unwrap_or(0)
                .saturating_sub(MEDIA_LIST_SIZE);

            if *x < max_offset {
                *x += 1;
            }
        });
    };

    view! {

        <div class="flex items-center gap-3 sm:gap-4">
            <div class="flex items-center gap-1 rounded-full border border-white/10 bg-white/5 p-1 backdrop-blur">
                <button
                    on:click=go_prev
                    disabled=move || !can_prev()
                    aria-label="Previous"
                    class="flex h-8 w-8 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 disabled:cursor-not-allowed disabled:opacity-30"
                >
                    <PrevIcon/>
                </button>

                <button
                    on:click=go_next
                    disabled=move || !can_next()
                    aria-label="Next"
                    class="flex h-8 w-8 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 disabled:cursor-not-allowed disabled:opacity-30"
                >
                    <NextIcon/>
                </button>
            </div>

            <a
                href=href
                class="group inline-flex items-center gap-1 text-sm font-semibold text-cyan-400 transition hover:text-cyan-300"
            >
                <span class="transition-transform group-hover:translate-x-1">"عرض الكل"</span>
                <span class="text-lg transition-transform group-hover:translate-x-1" aria-hidden="true">"←"</span>
            </a>
        </div>
    }
}

#[component]
fn MediaLoader<T>(
    resource: Resource<Result<Vec<T>, ServerFnError>>,
    offset: RwSignal<usize>,
    count: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView
where
    T: Card + Send + Sync + Serialize + DeserializeOwned + Clone + 'static,
{
    let context = format!("تحميل {}...", T::media_type().ar_title());

    let adapter = move |items: Vec<T>| MediaSectionProps {
        items,
        items_offset: offset,
        items_count: count,
    };

    view! {
        <ResourceView
            resource={resource}
            view_fn={MediaSection}
            adapter={adapter}
            fallback={CardsLoading}
            context={context}
        />
    }
}
