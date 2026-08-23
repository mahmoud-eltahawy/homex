use crate::app::{
    icons::{
        AudioIcon, ClockIcon, MovieIcon, MoviePosterSvg, MusicPosterSvg, SeriesIcon,
        SeriesPosterSvg,
    },
    model::{AudioGroup, Media, MediaType, Movie, Series},
};
use leptos::{either::Either, prelude::*};

#[component]
fn MediaLink(href: String, children: Children) -> impl IntoView {
    view! {
        <a href=href class="group relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03] hover:-translate-y-2">
            {children()}
        </a>
    }
}

#[component]
pub fn MediaPageHeader(title: String, icon: impl IntoView) -> impl IntoView {
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
pub fn MovieCard(item: Movie) -> impl IntoView {
    let href = format!("/detail/movie/{}", item.id.0);
    view! {
        <MediaLink href=href>
            <MovieCardImage item=item.clone()/>
            <MovieCardInfo item=item/>
        </MediaLink>
    }
}

#[component]
pub fn Poster(poster: Option<String>, media_type: MediaType) -> impl IntoView {
    match poster {
        Some(poster) => Either::Left(view! {
            <img
                src=poster
                class="w-full h-full object-cover transition-transform duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] group-hover:scale-110"
                loading="lazy"
            />
        }),
        None => Either::Right(match media_type {
            MediaType::Movie => Either::Left(MoviePosterSvg()),
            MediaType::Series => Either::Right(Either::Left(SeriesPosterSvg())),
            MediaType::AudioGroup => Either::Right(Either::Right(MusicPosterSvg())),
        }),
    }
}

#[component]
fn MovieCardImage(item: Movie) -> impl IntoView {
    let title = item.title.to_string();
    let duration_display = item.duration.human_readable();

    view! {
        <div class="aspect-[2/3] relative overflow-hidden">
            <Poster poster=item.poster media_type=MediaType::Movie/>
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                    <div class="flex items-center gap-2 mt-1 text-gray-300 text-sm">
                        <span class="flex items-center"><ClockIcon/>{duration_display}</span>
                    </div>
                </div>
            </div>
            <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                <MovieIcon/>
                "فيلم"
            </div>
        </div>
    }
}

#[component]
fn MovieCardInfo(item: Movie) -> impl IntoView {
    let title = item.title.to_string();
    let size = item.file.size.human_readable();
    view! {
        <div class="p-4 flex flex-col gap-1">
            <h3 class="text-white font-semibold truncate text-sm">{title}</h3>
            <h4 class="text-white font-semibold truncate text-sm">{size}</h4>
            <div class="flex items-center justify-between text-gray-500 text-xs">
                <span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">
                    "← التفاصيل"
                </span>
            </div>
        </div>
    }
}

#[component]
pub fn SeriesCard(item: Series) -> impl IntoView {
    let href = format!("/detail/series/{}", item.id.0);
    view! {
        <MediaLink href=href>
            <SeriesCardImage item=item.clone()/>
            <SeriesCardInfo item=item/>
        </MediaLink>
    }
}

#[component]
fn SeriesCardImage(item: Series) -> impl IntoView {
    let title = item.title.to_string();
    view! {
        <div class="aspect-[2/3] relative overflow-hidden">
            <Poster poster=item.poster media_type=MediaType::Series/>
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                </div>
            </div>
            <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                <SeriesIcon/>
                "مسلسل"
            </div>
        </div>
    }
}

#[component]
fn SeriesCardInfo(item: Series) -> impl IntoView {
    let title = item.title.to_string();
    view! {
        <div class="p-4 flex flex-col gap-1">
            <h3 class="text-white font-semibold truncate text-sm">{title}</h3>
            <div class="flex items-center justify-between text-gray-500 text-xs">
                <span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">
                    "← التفاصيل"
                </span>
            </div>
        </div>
    }
}

#[component]
pub fn AudioCard(item: AudioGroup) -> impl IntoView {
    let href = format!("/detail/audio/{}", item.id.0);
    view! {
        <MediaLink href=href>
            <AudioCardImage item=item.clone()/>
            <AudioCardInfo item=item/>
        </MediaLink>
    }
}

#[component]
fn AudioCardImage(item: AudioGroup) -> impl IntoView {
    let title = item.title.to_string();
    view! {
        <div class="aspect-[2/3] relative overflow-hidden">
            <Poster poster=item.poster media_type=MediaType::AudioGroup/>
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                </div>
            </div>
            <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                <AudioIcon/>
                "مجموعة صوتية"
            </div>
        </div>
    }
}

#[component]
fn AudioCardInfo(item: AudioGroup) -> impl IntoView {
    let title = item.title.to_string();
    view! {
        <div class="p-4 flex flex-col gap-1">
            <h3 class="text-white font-semibold truncate text-sm">{title}</h3>
            <div class="flex items-center justify-between text-gray-500 text-xs">
                <span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">
                    "← التفاصيل"
                </span>
            </div>
        </div>
    }
}

#[component]
pub fn MediaCard(item: Media) -> impl IntoView {
    match item {
        Media::Movie(item) => Either::Left(view! { <MovieCard item=item/> }),
        Media::Series(item) => Either::Right(Either::Left(view! { <SeriesCard item=item/> })),
        Media::AudioGroup(item) => Either::Right(Either::Right(view! { <AudioCard item=item/> })),
    }
}

#[component]
pub fn CardsLoading() -> impl IntoView {
    let cards = (0..5).map(|_| CardSkeleton()).collect_view();
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6 my-15">
            {cards}
        </div>
    }
}

#[component]
pub fn CardSkeleton() -> impl IntoView {
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
