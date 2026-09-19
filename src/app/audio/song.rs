use super::{fetch_audio, fetch_audio_group_detail, fetch_audios};
use crate::app::{
    audio::AudioArtwork,
    detail::DetailShell,
    icons::{AudioIcon, ClockIcon, DownloadIcon},
    media_player::{MediaItem, MediaPlayer},
    model::{Audio, AudioGroup},
    resource_view::ResourceView,
    route_params::use_u64_param,
};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

pub struct AudioSongDetailPage {
    pub song: Resource<Result<Audio, ServerFnError>>,
    pub group: Resource<Result<AudioGroup, ServerFnError>>,
    pub audios: Resource<Result<Vec<Audio>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for AudioSongDetailPage {
    fn data() -> Self {
        let group_id = use_u64_param("id");
        let song_id = use_u64_param("song_id");

        Self {
            song: Resource::new(move || (group_id(), song_id()), |(g, s)| fetch_audio(g, s)),
            group: Resource::new(group_id, fetch_audio_group_detail),
            audios: Resource::new(group_id, fetch_audios),
        }
    }
    fn view(this: Self) -> AnyView {
        let group = this.group;
        let audios = this.audios;
        let adapter = move |song: Audio| AudioSongHelperProps {
            song,
            group,
            audios,
        };
        view! {
            <ResourceView resource=this.song view_fn=AudioSongHelper adapter=adapter />
        }
        .into_any()
    }
}

#[component]
fn AudioSongHelper(
    song: Audio,
    group: Resource<Result<AudioGroup, ServerFnError>>,
    audios: Resource<Result<Vec<Audio>, ServerFnError>>,
) -> impl IntoView {
    let adapter = move |group: AudioGroup| AudioSongWithGroupProps {
        song: song.clone(),
        group,
        audios,
    };
    view! {
        <ResourceView resource=group view_fn=AudioSongWithGroup adapter=adapter />
    }
}

#[component]
fn AudioSongWithGroup(
    song: Audio,
    group: AudioGroup,
    audios: Resource<Result<Vec<Audio>, ServerFnError>>,
) -> impl IntoView {
    let adapter = move |list: Vec<Audio>| AudioSongDetailProps {
        song: song.clone(),
        group: group.clone(),
        audios: list,
    };
    view! {
        <ResourceView resource=audios view_fn=AudioSongDetail adapter=adapter />
    }
}

#[component]
fn AudioSongDetail(song: Audio, group: AudioGroup, audios: Vec<Audio>) -> impl IntoView {
    let title = song.title.clone();
    let download = song.file.path.clone();
    let duration = song.file.human_readable_duration();
    let size = song.file.human_readable_size();

    let poster = group.poster.clone();
    let group_name = group.title.clone();
    let group_href = format!("/audio/detail/{}", group.id);

    let items: Vec<MediaItem> = audios
        .iter()
        .map(|a| MediaItem::new(a.id, a.title.clone(), a.file.path.clone()))
        .collect();
    let initial_index = audios.iter().position(|a| a.id == song.id).unwrap_or(0);
    let playlist_title = format!("مقاطع {}", group.title);
    let group_artwork = group.poster.clone();

    view! {
        <DetailShell poster=poster.clone()>
            <div class="flex flex-col gap-6">
                <Breadcrumb href=group_href name=group_name/>

                <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
                    <div class="flex-shrink-0 w-48 sm:w-56 md:w-64 mx-auto lg:mx-0">
                        <AudioArtwork poster=poster.clone() title=title.clone()/>
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
                            download={title.clone()}
                            class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm"
                        >
                            <DownloadIcon/> "تحميل"
                        </a>
                    </div>
                </div>

                <div class="mt-4">
                    <MediaPlayer
                        items=items
                        initial_index=initial_index
                        audio=true
                        artwork=group_artwork
                        playlist_title=playlist_title
                    />
                </div>
            </div>
        </DetailShell>
    }
}

#[component]
fn Breadcrumb(href: String, name: String) -> impl IntoView {
    let name = if name.is_empty() {
        "المجموعة".to_string()
    } else {
        name
    };
    view! {
        <a
            href=href
            class="inline-flex items-center gap-1 text-sm text-gray-400 hover:text-white transition w-fit"
        >
            <span class="text-cyan-400">"←"</span>
            <span>{name}</span>
        </a>
    }
}
