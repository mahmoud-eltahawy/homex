use super::{AudioArtwork, fetch_audio_group_detail, fetch_audios};
use crate::app::{
    detail::{DetailHero, DetailShell, HeroBadge, HeroDescription, HeroMeta, HeroTitle},
    icons::{AudioIcon, ClockIcon, PlayIcon},
    model::{Audio, AudioGroup, MediaType},
    resource_view::ResourceView,
    route_params::use_u64_param,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

pub struct AudioGroupDetailPage {
    pub group: Resource<Result<AudioGroup, ServerFnError>>,
    pub audios: Resource<Result<Vec<Audio>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for AudioGroupDetailPage {
    fn data() -> Self {
        let id = use_u64_param("id");
        Self {
            group: Resource::new(id, fetch_audio_group_detail),
            audios: Resource::new(id, fetch_audios),
        }
    }

    fn view(this: Self) -> AnyView {
        let group_id = use_u64_param("id");

        let adapter = move |group: AudioGroup| AudioGroupDetailProps {
            group,
            audios: this.audios,
            group_id: group_id(),
        };
        view! {
            <ResourceView
                resource=this.group
                view_fn=AudioGroupDetail
                adapter=adapter
            />
        }
        .into_any()
    }
}

#[component]
fn AudioGroupDetail(
    group: AudioGroup,
    audios: Resource<Result<Vec<Audio>, ServerFnError>>,
    group_id: u64,
) -> impl IntoView {
    let poster = group.poster.clone();
    let title = group.title.clone();
    let description = group
        .description
        .clone()
        .unwrap_or_else(|| "لا يوجد وصف متاح.".to_string());
    let count = group.audios_count;
    let edit_href = MediaType::AudioGroup.edit_href(group.id);

    let adapter = move |list: Vec<Audio>| AudioListProps {
        audios: list,
        group_id,
    };

    view! {
        <DetailShell poster=poster.clone() edit_href>
            <DetailHero poster=view! {
                <AudioArtwork poster=poster.clone() title=title.clone()/>
            }>
                <HeroBadge label="مجموعة صوتية" icon=AudioIcon()/>
                <HeroTitle title=title.clone()/>
                <HeroMeta>
                    <span class="flex items-center gap-1">
                        <ClockIcon/>
                        {format!("{} مقطع", count)}
                    </span>
                </HeroMeta>
                <HeroDescription text=description/>
            </DetailHero>
            <div class="mt-10">
                <h2 class="text-xl sm:text-2xl font-bold text-white mb-4 flex items-center gap-2">
                    <AudioIcon/> " المقاطع"
                </h2>
                <ResourceView
                    resource=audios
                    view_fn=AudioList
                    adapter=adapter
                />
            </div>
        </DetailShell>
    }
}

#[component]
fn AudioList(audios: Vec<Audio>, group_id: u64) -> impl IntoView {
    if audios.is_empty() {
        return Either::Left(view! {
            <div class="py-12 text-center text-gray-500 text-sm">
                "لا توجد مقاطع صوتية في هذه المجموعة."
            </div>
        });
    }
    Either::Right(view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <For each=move || audios.clone() key=|a| a.id let:audio>
                <AudioSongLink audio=audio group_id=group_id/>
            </For>
        </div>
    })
}

#[component]
fn AudioSongLink(audio: Audio, group_id: u64) -> impl IntoView {
    let href = audio.href(group_id);
    let title = audio.title.clone();
    let duration = audio.file.human_readable_duration();

    view! {
        <a
            href=href
            class="group flex items-center justify-between gap-3 p-3 rounded-xl border border-white/10 bg-white/5 hover:bg-white/10 hover:border-cyan-400/40 transition-all cursor-pointer"
        >
            <div class="flex items-center gap-3 min-w-0">
                <span class="flex items-center justify-center h-10 w-10 rounded-full bg-cyan-500/15 text-cyan-400 group-hover:bg-cyan-500/25 transition shrink-0">
                    <PlayIcon/>
                </span>
                <span class="text-white truncate font-medium">{title}</span>
            </div>
            <span class="text-xs text-gray-400 font-mono shrink-0 hidden sm:inline">
                {duration}
            </span>
        </a>
    }
}
