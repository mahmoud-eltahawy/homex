use super::{fetch_audio, fetch_audio_group_detail, fetch_audios};
use crate::app::{
    audio::AudioArtwork,
    detail::{DetailHero, DetailShell, DownloadButton, HeroBadge, HeroMeta, HeroTitle},
    icons::{AudioIcon, ClockIcon},
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

                <DetailHero poster=view! {
                    <AudioArtwork poster=poster.clone() title=title.clone()/>
                }>
                    <HeroBadge label="مقطع صوتي" icon=AudioIcon()/>
                    <HeroTitle title=title.clone()/>
                    <HeroMeta>
                        <span class="flex items-center gap-1"><ClockIcon/>{duration}</span>
                        <span>{size}</span>
                    </HeroMeta>
                    <div class="mt-6 flex gap-3">
                        <DownloadButton href=download download_name=title.clone()/>
                    </div>
                </DetailHero>

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
