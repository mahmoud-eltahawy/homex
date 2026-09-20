use crate::app::icons::{
    DeleteIcon, EditIcon, FullscreenExitIcon, FullscreenIcon, MuteIcon, NextPageIcon, PauseIcon,
    PlayIcon, PrevPageIcon, VolumeIcon,
};
use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, ev::fullscreenchange};
use leptos::{html, prelude::*};
use leptos_use::{UseTimeoutFnReturn, use_document, use_event_listener, use_timeout_fn};
use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, MouseEvent};

// ─── Public data type ─────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: u64,
    pub title: String,
    pub subtitle: Option<String>,
    pub src: String,
    pub artwork: Option<String>,
}

impl MediaItem {
    pub fn new(id: u64, title: impl Into<String>, src: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            src: src.into(),
            artwork: None,
        }
    }
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

// ─── Top-level component ──────────────────────────────────────────────────

#[component]
pub fn MediaPlayer(
    items: Signal<Vec<MediaItem>>,
    #[prop(optional)] initial_index: usize,
    #[prop(default = false)] audio: bool,
    #[prop(default = None)] artwork: Option<String>,
    #[prop(optional)] playlist_title: Option<String>,
    #[prop(default = true)] show_playlist: bool,
    #[prop(optional)] on_rename: Option<Callback<(u64, String)>>,
    #[prop(optional)] on_delete: Option<Callback<u64>>,
) -> impl IntoView {
    let current_idx = RwSignal::new(initial_index);

    let has_playlist = Signal::derive(move || show_playlist && items.get().len() > 1);

    Effect::new(move |_| {
        let n = items.get().len();
        if n == 0 {
            current_idx.set(0);
        } else if current_idx.get() >= n {
            current_idx.set(n - 1);
        }
    });

    let current_item = Memo::new(move |_| items.get().get(current_idx.get()).cloned());

    let current_src = Signal::derive(move || current_item.get().map(|i| i.src).unwrap_or_default());
    let current_title =
        Signal::derive(move || current_item.get().map(|i| i.title).unwrap_or_default());
    let current_artwork = {
        let fallback = artwork.clone();
        Signal::derive(move || {
            current_item
                .get()
                .and_then(|i| i.artwork)
                .or_else(|| fallback.clone())
        })
    };

    let has_prev = Signal::derive(move || current_idx.get() > 0);

    let has_next = Signal::derive(move || current_idx.get() + 1 < items.get().len());

    let on_next: Callback<MouseEvent> = Callback::new(move |_| {
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            current_idx.update(|i| *i += 1);
        }
    });

    let on_prev: Callback<MouseEvent> = Callback::new(move |_| {
        current_idx.update(|i| *i = i.saturating_sub(1));
    });

    // ── Element + state ──────────────────────────────────────────────
    let video_ref = NodeRef::<html::Video>::new();
    let playing = RwSignal::new(false);
    let current_time = RwSignal::new(0.0);
    let duration = RwSignal::new(0.0);
    let volume = RwSignal::new(1.0);
    let last_volume = RwSignal::new(1.0);
    let muted = RwSignal::new(false);
    let fullscreen = RwSignal::new(false);
    let controls_visible = RwSignal::new(true);
    let play_after_load = RwSignal::new(false);

    let u_document = use_document();
    let _guard = use_event_listener(u_document.clone(), fullscreenchange, move |_| {
        fullscreen.set(u_document.fullscreen().is_some_and(|x| x));
    });

    let UseTimeoutFnReturn { start, stop, .. } = use_timeout_fn(
        move |_i: i8| {
            controls_visible.set(false);
        },
        3000.,
    );
    let start_hide_timer = {
        let stop = stop.clone();
        move || {
            stop();
            start(3);
        }
    };
    let show_controls = {
        let start_hide_timer = start_hide_timer.clone();
        move || {
            controls_visible.set(true);
            start_hide_timer();
        }
    };
    let toggle_controls = {
        let show_controls = show_controls.clone();
        let stop = stop.clone();
        move || {
            if controls_visible.get() {
                controls_visible.set(false);
                stop();
            } else {
                show_controls();
            }
        }
    };

    let handle_loaded_metadata = move |_| {
        if let Some(video) = video_ref.get() {
            duration.set(video.duration());
        }
    };
    let handle_time_update = move |_| {
        if let Some(video) = video_ref.get() {
            current_time.set(video.current_time());
        }
    };
    let toggle_play = move |_| {
        if let Some(video) = video_ref.get() {
            if playing.get() {
                video.pause().ok();
            } else {
                let _ = video.play();
            }
        }
    };
    let handle_seek = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_current_time(val);
            current_time.set(val);
        }
    };
    let handle_volume = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_volume(val);
            video.set_muted(val == 0.0);
            volume.set(val);
            muted.set(val == 0.0);
            if val > 0.0 {
                last_volume.set(val);
            }
        }
    };
    let toggle_mute = move |_| {
        if let Some(video) = video_ref.get() {
            if muted.get() {
                video.set_muted(false);
                let restore = last_volume.get().max(0.1);
                video.set_volume(restore);
                volume.set(restore);
                muted.set(false);
            } else {
                last_volume.set(volume.get().max(0.1));
                video.set_muted(true);
                muted.set(true);
            }
        }
    };
    let toggle_fullscreen = move |_| {
        if let Some(video) = video_ref.get() {
            if document().fullscreen_element().is_none() {
                let _ = video.request_fullscreen();
            } else {
                document().exit_fullscreen();
            }
        }
    };

    // Auto-advance. We set a flag so the src-change effect knows to
    // immediately start playback (matching YouTube behaviour).

    let handle_ended = move |_| {
        playing.set(false);
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            play_after_load.set(true);
            current_idx.update(|i| *i += 1);
        }
    };

    // Reload the media element whenever the current item changes.
    Effect::new(move || {
        let src = current_src.get();
        if let Some(video) = video_ref.get() {
            video.set_src(&src);
            video.load();
            playing.set(false);
            current_time.set(0.0);
            duration.set(0.0);
            if play_after_load.get_untracked() {
                play_after_load.set(false);
                let _ = video.play();
            }
        }
    });

    let video_class = if audio {
        "w-full h-0 pointer-events-none"
    } else {
        "w-full h-auto max-h-[60vh] md:max-h-[70vh] object-contain cursor-pointer"
    };

    let artwork_view = if audio {
        let toggle_controls = toggle_controls.clone();
        Some(view! {
            <div
                class="relative w-full aspect-video flex items-center justify-center bg-gradient-to-br from-[#1e1e2e] via-[#14141e] to-[#0a0a0f] cursor-pointer overflow-hidden"
                on:click=move |_| toggle_controls()
            >
                {move || current_artwork.get().map(|url| view! {
                    <div
                        class="absolute inset-0 opacity-50"
                        style=format!(
                            "background-image: url('{url}'); background-size: cover; \
                             background-position: center; filter: blur(50px) saturate(1.4); \
                             transform: scale(1.3);"
                        )
                    ></div>
                    <img
                        src=url
                        class="relative max-h-[70%] max-w-[70%] object-contain rounded-2xl shadow-2xl shadow-black/70 border border-white/10"
                        alt=""
                    />
                })}
            </div>
        })
    } else {
        None
    };

    let player_view = view! {
        <div
            on:mousemove={let show = show_controls.clone(); move |_| show()}
            dir="ltr"
            class="relative bg-black rounded-2xl overflow-hidden shadow-2xl shadow-black/50 group"
        >
            <video
                node_ref=video_ref
                title=move || current_title.get()
                class=video_class
                on:loadedmetadata=handle_loaded_metadata
                on:timeupdate=handle_time_update
                on:play=move |_| playing.set(true)
                on:pause=move |_| playing.set(false)
                on:ended=handle_ended
                on:click={let tc = toggle_controls.clone(); move |_| tc()}
                playsinline
            />
            {artwork_view}
            <MediaControls
                controls_visible=controls_visible
                show_controls=show_controls.clone()
                current_time=current_time
                duration=duration
                playing=playing
                muted=muted
                volume=volume
                fullscreen=fullscreen
                toggle_play=toggle_play
                toggle_mute=toggle_mute
                toggle_fullscreen=toggle_fullscreen
                handle_seek=handle_seek
                handle_volume=handle_volume
                start_hide_timer=start_hide_timer
                show_fullscreen=!audio
                show_nav=has_playlist
                current_title=current_title
                on_prev=on_prev
                on_next=on_next
                has_prev=has_prev
                has_next=has_next
            />
        </div>
    };

    let playlist_view = view! {
        <Show when=move || has_playlist.get()>
            <aside class="w-full lg:w-80 xl:w-96 lg:flex-shrink-0">
                <PlaylistPanel
                    items=items
                    current_idx=current_idx
                    title=playlist_title.clone()
                    on_rename=on_rename
                    on_delete=on_delete
                />
            </aside>
        </Show>
    };

    view! {
        <div class="flex flex-col lg:flex-row gap-4 items-start">
            <div class="flex-1 min-w-0 w-full">{player_view}</div>
            {playlist_view}
        </div>
    }
}

// ─── Controls overlay ─────────────────────────────────────────────────────

#[component]
fn MediaControls(
    controls_visible: RwSignal<bool>,
    show_controls: impl Fn() + Clone + 'static,
    current_time: RwSignal<f64>,
    duration: RwSignal<f64>,
    playing: RwSignal<bool>,
    muted: RwSignal<bool>,
    volume: RwSignal<f64>,
    fullscreen: RwSignal<bool>,
    toggle_play: impl Fn(MouseEvent) + 'static,
    toggle_mute: impl Fn(MouseEvent) + 'static,
    toggle_fullscreen: impl Fn(MouseEvent) + 'static,
    handle_seek: impl Fn(web_sys::Event) + 'static,
    handle_volume: impl Fn(web_sys::Event) + 'static,
    start_hide_timer: impl Fn() + 'static + Clone,
    #[prop(default = true)] show_fullscreen: bool,
    show_nav: Signal<bool>,
    #[prop(into)] current_title: Signal<String>,
    on_prev: Callback<MouseEvent>,
    on_next: Callback<MouseEvent>,
    #[prop(into)] has_prev: Signal<bool>,
    #[prop(into)] has_next: Signal<bool>,
) -> impl IntoView {
    let class = move || {
        format!(
            "absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/50 \
             to-transparent p-3 sm:p-5 transition-opacity duration-300 {}",
            if controls_visible.get() {
                "opacity-100"
            } else {
                "opacity-0"
            }
        )
    };
    let on_mouse_leave = {
        let start = start_hide_timer.clone();
        move |_| start()
    };
    view! {
        <div
            class=class
            on:mouseenter={let show = show_controls.clone(); move |_| show()}
            on:mouseleave=on_mouse_leave
            on:touchstart={let show = show_controls.clone(); move |_| show()}
        >
            <div class="flex flex-col gap-2">
                <div class="text-white text-sm font-medium truncate px-1 drop-shadow">
                    {move || current_title.get()}
                </div>
                <SeekBar current_time duration=duration handle_seek=handle_seek />
                <ControlButtons
                    playing=playing
                    muted=muted
                    volume=volume
                    fullscreen=fullscreen
                    toggle_play=toggle_play
                    toggle_mute=toggle_mute
                    toggle_fullscreen=toggle_fullscreen
                    handle_volume=handle_volume
                    show_fullscreen=show_fullscreen
                    show_nav=show_nav
                    on_prev=on_prev
                    on_next=on_next
                    has_prev=has_prev
                    has_next=has_next
                />
            </div>
        </div>
    }
}

#[component]
fn SeekBar(
    current_time: RwSignal<f64>,
    duration: RwSignal<f64>,
    handle_seek: impl Fn(web_sys::Event) + 'static,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            <span class="text-white text-xs font-mono">
                {move || format_time(current_time.get())}
            </span>
            <input
                type="range"
                min="0"
                prop:max=duration
                prop:value=current_time
                on:input=handle_seek
                class="flex-1 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-cyan-400/30"
            />
            <span class="text-white text-xs font-mono">
                {move || format_time(duration.get())}
            </span>
        </div>
    }
}

#[component]
fn ControlButtons(
    playing: RwSignal<bool>,
    muted: RwSignal<bool>,
    volume: RwSignal<f64>,
    fullscreen: RwSignal<bool>,
    toggle_play: impl Fn(MouseEvent) + 'static,
    toggle_mute: impl Fn(MouseEvent) + 'static,
    toggle_fullscreen: impl Fn(MouseEvent) + 'static,
    handle_volume: impl Fn(web_sys::Event) + 'static,
    #[prop(default = true)] show_fullscreen: bool,
    show_nav: Signal<bool>,
    on_prev: Callback<MouseEvent>,
    on_next: Callback<MouseEvent>,
    has_prev: Signal<bool>,
    has_next: Signal<bool>,
) -> impl IntoView {
    let play_icon = move || {
        if playing.get() {
            Either::Left(PauseIcon())
        } else {
            Either::Right(PlayIcon())
        }
    };
    let mute_icon = move || {
        if muted.get() || volume.get() == 0.0 {
            Either::Left(MuteIcon())
        } else {
            Either::Right(VolumeIcon())
        }
    };
    let vol_value = move || if muted.get() { 0.0 } else { volume.get() };
    let full_screen = move || {
        if fullscreen.get() {
            Either::Left(FullscreenExitIcon())
        } else {
            Either::Right(FullscreenIcon())
        }
    };
    let fullscreen_btn = show_fullscreen.then(move || {
        view! {
            <button
                on:click=toggle_fullscreen
                class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
                aria-label="ملء الشاشة"
            >
                {full_screen}
            </button>
        }
    });

    let nav_class = "hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10 disabled:opacity-30 disabled:hover:bg-transparent disabled:hover:scale-100";

    let prev_btn = move || {
        show_nav.get().then(move || {
            view! {
                <button
                    on:click=move |ev| on_prev.run(ev)
                    disabled=move || !has_prev.get()
                    class=nav_class
                    aria-label="السابق"
                >
                    <PrevPageIcon/>
                </button>
            }
        })
    };
    let next_btn = move || {
        show_nav.get().then(move || {
            view! {
                <button
                    on:click=move |ev| on_next.run(ev)
                    disabled=move || !has_next.get()
                    class=nav_class
                    aria-label="التالي"
                >
                    <NextPageIcon/>
                </button>
            }
        })
    };

    view! {
        <div class="flex items-center gap-2 sm:gap-3 text-white">
            {prev_btn}
            <button
                on:click=toggle_play
                class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
                aria-label="تشغيل / إيقاف"
            >
                {play_icon}
            </button>
            {next_btn}
            <div class="flex items-center gap-2">
                <button
                    on:click=toggle_mute
                    class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
                    aria-label="كتم / إلغاء"
                >
                    {mute_icon}
                </button>
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    prop:value={vol_value}
                    on:input=handle_volume
                    class="w-16 sm:w-20 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400"
                />
            </div>
            <div class="flex-1"></div>
            {fullscreen_btn}
        </div>
    }
}

// ─── Playlist ─────────────────────────────────────────────────────────────

#[component]
fn PlaylistPanel(
    items: Signal<Vec<MediaItem>>,
    current_idx: RwSignal<usize>,
    #[prop(default = None)] title: Option<String>,
    #[prop(default = None)] on_rename: Option<Callback<(u64, String)>>,
    #[prop(default = None)] on_delete: Option<Callback<u64>>,
) -> impl IntoView {
    let title = title.unwrap_or_else(|| "قائمة التشغيل".to_string());

    view! {
        <div class="bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10 overflow-hidden flex flex-col max-h-[60vh] lg:max-h-[500px]">
            <div class="px-4 py-3 border-b border-white/10 flex items-center justify-between shrink-0">
                <h3 class="text-sm font-bold text-white truncate">{title}</h3>
                <span class="text-xs text-gray-400 font-mono bg-white/10 px-2 py-0.5 rounded-full">
                    {move || items.get().len()}
                </span>
            </div>
            <div class="overflow-y-auto p-2 flex-1 min-h-0">
                <For
                    each=move || items.get().into_iter().enumerate()
                    key=|(_, item)| item.id
                    let:entry
                >
                    {
                        let (index, item) = entry;
                        view! {
                            <PlaylistItem
                                item=item
                                index=index
                                current_idx=current_idx
                                on_rename=on_rename
                                on_delete=on_delete
                            />
                        }
                    }
                </For>
            </div>
        </div>
    }
}

#[component]
fn PlaylistItem(
    item: MediaItem,
    index: usize,
    current_idx: RwSignal<usize>,
    #[prop(default = None)] on_rename: Option<Callback<(u64, String)>>,
    #[prop(default = None)] on_delete: Option<Callback<u64>>,
) -> impl IntoView {
    let id = item.id;

    // StoredValue is Copy and survives being captured by multiple closures.
    let title = StoredValue::new(item.title.clone());
    let subtitle = StoredValue::new(item.subtitle.clone());

    let is_current = move || current_idx.get() == index;

    let editing = RwSignal::new(false);
    let draft = RwSignal::new(item.title.clone());
    let input_ref = NodeRef::<html::Input>::new();

    let on_select = move |_| {
        if !editing.get_untracked() {
            current_idx.set(index);
        }
    };

    let begin_edit = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        draft.set(title.get_value());
        editing.set(true);
    };

    // Callback is Copy — both on:keydown and on:blur can hold a copy.
    let do_commit: Callback<()> = Callback::new(move |_| {
        editing.set(false);
        let new_title = draft.get_untracked();
        if new_title != title.get_value()
            && let Some(cb) = on_rename
        {
            cb.run((id, new_title));
        }
    });

    let on_delete_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        if let Some(cb) = on_delete {
            cb.run(id);
        }
    };

    Effect::new(move |_| {
        if editing.get()
            && let Some(input) = input_ref.get()
        {
            let _ = input.focus();
            input.select();
        }
    });

    let row_class = move || {
        format!(
            "w-full flex items-center gap-2 p-2 rounded-lg cursor-pointer transition text-right group/row {}",
            if is_current() {
                "bg-cyan-500/15 border border-cyan-500/30"
            } else {
                "border border-transparent hover:bg-white/5"
            }
        )
    };

    view! {
        <div class=row_class on:click=on_select>
            <PlaylistIndicator index=index is_current=Signal::derive(is_current)/>

            <div class="flex-1 min-w-0">
                <Show
                    when=move || editing.get()
                    fallback=move || view! {
                        <>
                            <div class="text-sm text-white truncate">{title.get_value()}</div>
                            {subtitle.get_value().map(|s| view! {
                                <div class="text-xs text-gray-400 truncate">{s}</div>
                            })}
                        </>
                    }
                >
                    <input
                        node_ref=input_ref
                        type="text"
                        class="w-full bg-white/10 text-white text-sm rounded px-2 py-1 focus:outline-none focus:ring-1 focus:ring-cyan-400"
                        prop:value=move || draft.get()
                        on:input=move |ev| draft.set(event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
                            "Enter" => { ev.prevent_default(); do_commit.run(()); }
                            "Escape" => { ev.prevent_default(); editing.set(false); }
                            _ => {}
                        }
                        on:blur=move |_| do_commit.run(())
                        on:click=move |ev| ev.stop_propagation()
                    />
                </Show>
            </div>

            {on_rename.map(|_| view! {
                <button
                    type="button"
                    on:click=begin_edit
                    class="opacity-0 group-hover/row:opacity-100 transition text-gray-400 hover:text-white p-1 shrink-0"
                    aria-label="إعادة تسمية"
                >
                    <EditIcon/>
                </button>
            })}

            {on_delete.map(|_| view! {
                <button
                    type="button"
                    on:click=on_delete_click
                    class="opacity-0 group-hover/row:opacity-100 transition text-red-400 hover:text-red-300 p-1 shrink-0"
                    aria-label="حذف"
                >
                    <DeleteIcon/>
                </button>
            })}
        </div>
    }
}

#[component]
fn PlaylistIndicator(index: usize, #[prop(into)] is_current: Signal<bool>) -> impl IntoView {
    view! {
        {move || if is_current.get() {
            Either::Left(view! {
                <span class="flex items-center justify-center w-8 h-8 rounded-full bg-cyan-500/20 text-cyan-400 shrink-0">
                    <PlayIcon/>
                </span>
            })
        } else {
            Either::Right(view! {
                <span class="flex items-center justify-center w-8 h-8 rounded-full bg-white/5 text-gray-300 text-xs font-bold shrink-0">
                    {index + 1}
                </span>
            })
        }}
    }
}

// ─── Util ─────────────────────────────────────────────────────────────────

fn format_time(time: f64) -> String {
    if time.is_nan() {
        return "00:00".into();
    }
    let t = time as u64;
    let h = t / 3600;
    let m = (t % 3600) / 60;
    let s = t % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
