use crate::app::{
    icons::{ClockIcon, DownloadIcon, MovieIcon},
    model::{self, Movie, MovieChapter},
    resource_view::ResourceView,
    video_player::VideoPlayer,
    view_schema::PosterView,
};
use leptos::prelude::*;
use leptos_router::{hooks::use_params_map, lazy_route, LazyRoute};
use web_sys::wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

#[server]
pub async fn fetch_movie_detail(id: u64) -> Result<model::Movie, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(200).await;
    let list = mockary::mock_movies();
    list.into_iter()
        .find(|m| m.id == id)
        .ok_or(ServerFnError::new("not found"))
}

pub struct MovieDetailPage {
    movie: Resource<Result<Movie, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for MovieDetailPage {
    fn data() -> Self {
        let params = use_params_map();
        let id =
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0));

        let movie = Resource::new(id, fetch_movie_detail);
        Self { movie }
    }

    fn view(this: Self) -> AnyView {
        let adapter = |movie| MovieDetailProps { movie };
        view! {
            <ResourceView
                resource=this.movie
                view_fn=MovieDetail
                adapter=adapter
            />
        }
        .into_any()
    }
}

#[component]
fn MovieDetail(movie: Movie) -> impl IntoView {
    let selected_chapter_idx = RwSignal::new(0usize);
    let chapters = movie.chapters.clone();

    let selector = (chapters.len() > 1).then_some(view! {
        <ChapterSelector
            chapters=movie.chapters.clone()
            selected_idx=selected_chapter_idx
        />
    });

    let selected_chapter = Memo::new(move |_| chapters.get(selected_chapter_idx.get()).cloned());

    // Memo for the video source path of the selected chapter
    let video_src = Memo::new(move |_| {
        selected_chapter
            .get()
            .map(|ch| ch.file.path.clone())
            .unwrap_or_default()
    });

    view! {
        <div class="relative min-h-screen bg-black text-white overflow-hidden">
            <div class="absolute inset-0">
                {movie.clone().poster()}
                <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
            </div>
            <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                <DetailBody
                    movie=movie.clone()
                    selected_chapter=selected_chapter
                    video_src=video_src
                />
                {selector}
            </div>
        </div>
    }
}

#[component]
fn DetailBody(
    movie: Movie,
    selected_chapter: Memo<Option<MovieChapter>>,
    video_src: Memo<String>,
) -> impl IntoView {
    // Clone for closure use in video player
    let movie_title = movie.title.clone();

    view! {
        <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
                {movie.clone().poster()}
            </div>
            <div class="flex-1 w-full">
                <DetailMetaBadge/>
                <DetailInfo movie=movie.clone() selected_chapter=selected_chapter />
            </div>
        </div>

        <Show when=move || !video_src.get().is_empty()>
            <div class="mt-10">
                <VideoPlayer
                    src=Signal::derive(move || video_src.get())
                    title=movie_title.clone()
                />
            </div>
        </Show>
    }
}

#[component]
fn ChapterSelector(chapters: Vec<MovieChapter>, selected_idx: RwSignal<usize>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 mt-6 mb-4">
            <span class="text-gray-300 text-sm">اختر الجزء:</span>
            <select
                class="bg-white/10 backdrop-blur-md text-white rounded-xl py-1.5 px-3 focus:outline-none focus:ring-1 focus:ring-cyan-400"
                prop:value=move || selected_idx.get().to_string()
                on:change=move |ev| {
                    if let Some(sel) = ev.target()
                        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                    {
                        if let Ok(num) = sel.value().parse::<usize>() {
                            selected_idx.set(num);
                        }
                    }
                }
            >
                <For
                    each={move || chapters.clone()}
                    key=|ch| ch.id
                    let:chapter
                >
                    <option
                        value={chapter.number.to_string()}
                        selected={chapter.number as usize == selected_idx.get()}
                    >
                        {chapter.title.clone().unwrap_or_else(|| format!("Chapter {}", chapter.number + 1))}
                    </option>
                </For>
            </select>
        </div>
    }
}

#[component]
fn DetailMetaBadge() -> impl IntoView {
    let media_icon = MovieIcon();
    let name = "فيلم";
    view! {
        <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
            {media_icon}
            {name}
        </div>
    }
}

#[component]
fn DetailInfo(movie: Movie, selected_chapter: Memo<Option<MovieChapter>>) -> impl IntoView {
    let title = movie.title.to_string();
    let description = movie
        .description
        .unwrap_or_else(|| "لا يوجد وصف متاح.".to_string());

    // Closure to get download link and file info from selected chapter
    let download_link = move || {
        selected_chapter
            .get()
            .map(|ch| ch.file.path.clone())
            .unwrap_or_default()
    };
    let duration = move || {
        selected_chapter
            .get()
            .map(|ch| ch.file.human_readable_duration())
            .unwrap_or_default()
    };
    let size = move || {
        selected_chapter
            .get()
            .map(|ch| ch.file.human_readable_size())
            .unwrap_or_default()
    };

    view! {
        <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">{title}</h1>
        <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
            <span class="flex items-center gap-1"><ClockIcon/>{duration}</span>
            <span>{size}</span>
        </div>
        <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{description}</p>
        <div class="mt-6 flex gap-3">
            <a
                download="download"
                href=download_link
                class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm"
            >
                <DownloadIcon/> "تحميل"
            </a>
        </div>
    }
}
