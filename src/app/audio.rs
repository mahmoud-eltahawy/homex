use crate::app::{
    icons::{AudioIcon, MusicPosterSvg},
    model::{self, Audio, AudioGroup, MediaType},
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod detail;
pub mod song {

    use super::{fetch_audio, fetch_audio_group_detail};
    use crate::app::{
        detail::DetailShell,
        icons::{AudioIcon, ClockIcon, DownloadIcon},
        model::{Audio, AudioGroup},
        resource_view::ResourceView,
        video_player::VideoPlayer,
    };
    use leptos::either::Either;
    use leptos::prelude::*;
    use leptos_router::{hooks::use_params_map, lazy_route, LazyRoute};

    pub struct AudioSongDetailPage {
        pub song: Resource<Result<Audio, ServerFnError>>,
        pub group: Resource<Result<AudioGroup, ServerFnError>>,
    }

    #[lazy_route]
    impl LazyRoute for AudioSongDetailPage {
        fn data() -> Self {
            let params = use_params_map();
            let group_id = move || {
                params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0))
            };
            let song_id = move || {
                params.with(|p| {
                    p.get("song_id")
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                })
            };

            Self {
                song: Resource::new(move || (group_id(), song_id()), |(g, s)| fetch_audio(g, s)),
                group: Resource::new(group_id, fetch_audio_group_detail),
            }
        }

        fn view(this: Self) -> AnyView {
            let group = this.group;
            let adapter = move |song: Audio| AudioSongDetailProps { song, group };
            view! {
                <ResourceView
                    resource=this.song
                    view_fn=AudioSongDetail
                    adapter=adapter
                />
            }
            .into_any()
        }
    }

    #[component]
    fn AudioSongDetail(
        song: Audio,
        group: Resource<Result<AudioGroup, ServerFnError>>,
    ) -> impl IntoView {
        let title = song.title.clone();
        let src = Signal::derive({
            let path = song.file.path.clone();
            move || path.clone()
        });
        let download = song.file.path.clone();
        let duration = song.file.human_readable_duration();
        let size = song.file.human_readable_size();

        let poster =
            Signal::derive(move || group.get().and_then(|res| res.ok()).and_then(|g| g.poster));
        let group_name = Signal::derive(move || {
            group
                .get()
                .and_then(|res| res.ok())
                .map(|g| g.title)
                .unwrap_or_default()
        });
        let group_href = Signal::derive(move || {
            group
                .get()
                .and_then(|res| res.ok())
                .map(|g| format!("/audio/detail/{}", g.id))
                .unwrap_or_else(|| "/audio".to_string())
        });

        view! {
            <DetailShell poster=poster.get()>
                <div class="flex flex-col gap-6">
                    <Breadcrumb href=group_href name=group_name/>

                    <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
                        <div class="flex-shrink-0 w-48 sm:w-56 md:w-64 mx-auto lg:mx-0">
                            <AudioArtwork poster=poster title=title.clone()/>
                        </div>
                        <div class="flex-1 w-full">
                            <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
                                <AudioIcon/>
                                "مقطع صوتي"
                            </div>
                            <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">
                                {title.clone()}
                            </h1>
                            <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
                                <span class="flex items-center gap-1">
                                    <ClockIcon/>
                                    {duration}
                                </span>
                                <span>{size}</span>
                            </div>
                            <a
                                href=download
                                download="download"
                                class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm"
                            >
                                <DownloadIcon/> "تحميل"
                            </a>
                        </div>
                    </div>

                    <div class="mt-4">
                        <VideoPlayer
                            src=src
                            title=title
                            audio=true
                            // artwork=poster
                        />
                    </div>
                </div>
            </DetailShell>
        }
    }

    #[component]
    fn AudioArtwork(poster: Signal<Option<String>>, title: String) -> impl IntoView {
        move || match poster.get() {
            Some(src) => Either::Left(view! {
                <img
                    src=src
                    class="w-full aspect-square object-cover rounded-2xl shadow-2xl border border-white/10"
                    alt=title.clone()
                />
            }),
            None => Either::Right(view! {
                <div class="w-full aspect-square rounded-2xl border border-white/10 bg-gradient-to-br from-cyan-500/20 via-purple-500/20 to-pink-500/20 flex items-center justify-center overflow-hidden">
                    <div class="text-cyan-300/80 origin-center scale-[6]">
                        <AudioIcon/>
                    </div>
                </div>
            }),
        }
    }

    #[component]
    fn Breadcrumb(href: Signal<String>, name: Signal<String>) -> impl IntoView {
        view! {
            <a
                href=move || href.get()
                class="inline-flex items-center gap-1 text-sm text-gray-400 hover:text-white transition w-fit"
            >
                <span class="text-cyan-400">"←"</span>
                <span>{move || {
                    let n = name.get();
                    if n.is_empty() { "المجموعة".to_string() } else { n }
                }}</span>
            </a>
        }
    }
}

impl CardData for AudioGroup {
    fn id(&self) -> u64 {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn poster_url(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn media_type() -> MediaType {
        MediaType::AudioGroup
    }

    fn badge_icon() -> impl IntoView {
        AudioIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        MusicPosterSvg()
    }
}

#[server]
pub async fn fetch_audio_groups(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<model::AudioGroup>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audio_groups;
    delay(300).await;

    let list = match search_query {
        None => mock_audio_groups(),
        Some(pat) => mock_audio_groups()
            .into_iter()
            .filter(|x| x.title.to_lowercase().contains(&pat.to_lowercase()))
            .collect(),
    };

    let size = size.clamp(0, list.len());
    let offset = offset.clamp(0, list.len() - size);
    let end = (offset + size).clamp(0, list.len());

    Ok(list[offset..end].to_vec())
}

#[server]
pub async fn fetch_audio_groups_count(
    search_query: Option<String>,
) -> Result<usize, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audio_groups;
    delay(300).await;

    let list = match search_query {
        None => mock_audio_groups(),
        Some(pat) => mock_audio_groups()
            .into_iter()
            .filter(|x| x.title.to_lowercase().contains(&pat.to_lowercase()))
            .collect(),
    };

    Ok(list.len())
}

#[server]
async fn fetch_audio_group_detail(id: u64) -> Result<AudioGroup, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audio_groups;
    delay(200).await;
    mock_audio_groups()
        .into_iter()
        .find(|g| g.id == id)
        .ok_or(ServerFnError::new("audio group not found"))
}

#[server]
pub async fn fetch_audios(group_id: u64) -> Result<Vec<Audio>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audios;
    delay(200).await;
    Ok(mock_audios(group_id))
}

#[server]
pub async fn fetch_audio(group_id: u64, song_id: u64) -> Result<Audio, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audios;
    delay(200).await;
    mock_audios(group_id)
        .into_iter()
        .find(|a| a.id == song_id)
        .ok_or(ServerFnError::new("audio not found"))
}
