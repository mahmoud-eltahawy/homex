use crate::app::{
    icons::{ClockIcon, DownloadIcon, MovieIcon},
    model::{self, Movie},
    resource_view::ResourceView,
    video_player::VideoPlayer,
};
use leptos::prelude::*;
use leptos_router::{hooks::use_params_map, lazy_route, LazyRoute};

#[server]
pub async fn fetch_movie_detail(id: i64) -> Result<model::Movie, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(200).await;
    let list = mockary::mock_movies();
    list.into_iter()
        .find(|m| m.id.0 == id)
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
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0));

        let movie = Resource::new(id, fetch_movie_detail);
        Self { movie }
    }

    fn view(this: Self) -> AnyView {
        let adapter = |movie| MovieDetailProps { movie };
        view! {
            <ResourceView
                resource= this.movie
                view_fn=MovieDetail
                adapter=adapter
                context="تحميل تفاصيل فيلم"
            />
        }
        .into_any()
    }
}

#[component]
fn MovieDetail(movie: Movie) -> impl IntoView {
    let video_src = movie.file.path.clone();

    view! {
        <div class="relative min-h-screen bg-black text-white overflow-hidden">
            <div class="absolute inset-0">
                <img src=movie.poster.to_string()
                     class="w-full h-full object-cover scale-110 blur-3xl opacity-20" alt="" />
                <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent"></div>
            </div>
            <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
                <DetailBody
                    data=movie
                    video_src=video_src
                />
            </div>
        </div>
    }
}

#[component]
fn DetailBody(data: Movie, video_src: String) -> impl IntoView {
    view! {
        <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <DetailPoster poster=data.poster.to_string() title=data.title.to_string() />
            <div class="flex-1 w-full">
                <DetailMetaBadge/>
                <DetailInfo data=data.clone() />
            </div>
        </div>

        {(!video_src.is_empty()).then_some(view! {
            <div class="mt-10">
                <VideoPlayer src=Signal::from(video_src) title=data.title.to_string() />
            </div>
        })}
    }
}

#[component]
fn DetailPoster(poster: String, title: String) -> impl IntoView {
    view! {
        <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
            <img src=poster class="w-full rounded-2xl shadow-2xl border border-white/10" alt=title />
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
fn DetailInfo(data: Movie) -> impl IntoView {
    let title = data.title.to_string();
    let duration = data.duration.human_readable();
    let size = data.file.size.human_readable();
    let description = data.description.unwrap_or("لا يوجد وصف متاح.".to_string());
    let download = view! {
        <a href=data.file.path.clone()
            class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm">
            <DownloadIcon/> "تحميل"
        </a>
    };
    view! {
        <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">{title}</h1>
        <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
            <span class="flex items-center gap-1"><ClockIcon/>{duration}</span>
            <span>{size}</span>
        </div>
        <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{description}</p>
        <div class="mt-6 flex gap-3">{download}</div>
    }
}
