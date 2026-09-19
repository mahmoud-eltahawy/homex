use super::{AudioArtwork, fetch_audio_group_detail, fetch_audios};
use crate::app::{
    detail::DetailShell,
    icons::{AudioIcon, ClockIcon, PlayIcon},
    model::{Audio, AudioGroup},
    resource_view::ResourceView,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, hooks::use_params_map, lazy_route};

pub struct AudioGroupDetailPage {
    pub group: Resource<Result<AudioGroup, ServerFnError>>,
    pub audios: Resource<Result<Vec<Audio>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for AudioGroupDetailPage {
    fn data() -> Self {
        let params = use_params_map();
        let id =
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0));

        Self {
            group: Resource::new(id, fetch_audio_group_detail),
            audios: Resource::new(id, fetch_audios),
        }
    }

    fn view(this: Self) -> AnyView {
        let params = use_params_map();
        let group_id =
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0));
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

    let adapter = move |list: Vec<Audio>| AudioListProps {
        audios: list,
        group_id,
    };

    let edit_href = format!("/audio/detail/{}/edit", group.id);
    view! {
        <DetailShell poster=poster.clone() edit_href>
            <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
                <div class="flex-shrink-0 w-48 sm:w-56 md:w-64 mx-auto lg:mx-0">
                    <AudioArtwork poster=poster title=title.clone()/>
                </div>
                <div class="flex-1 w-full">
                    <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
                        <AudioIcon/>
                        "مجموعة صوتية"
                    </div>
                    <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">
                        {title}
                    </h1>
                    <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
                        <span class="flex items-center gap-1">
                            <ClockIcon/>
                            {format!("{} مقطع", count)}
                        </span>
                    </div>
                    <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">
                        {description}
                    </p>
                </div>
            </div>
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
