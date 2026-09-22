use crate::app::common::ContextBundle;
use crate::app::icons::{
    DeleteIcon, DownloadIcon, EditIcon, FullscreenExitIcon, FullscreenIcon, MuteIcon, NextPageIcon,
    PauseIcon, PlayIcon, PrevPageIcon, VolumeIcon,
};
use crate::app::inline_edit::use_edit_mode;
use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, ev::fullscreenchange};
use leptos::{html, prelude::*};
use leptos_use::{UseTimeoutFnReturn, use_document, use_event_listener, use_timeout_fn};
use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, MouseEvent};

// ─── Shared class strings ─────────────────────────────────────────────────

const ICON_BUTTON: &str = "hover:scale-110 transition-transform duration-200 p-1 \
                           rounded-full hover:bg-white/10";

const NAV_BUTTON: &str = "hover:scale-110 transition-transform duration-200 p-1 \
                          rounded-full hover:bg-white/10 disabled:opacity-30 \
                          disabled:hover:bg-transparent disabled:hover:scale-100";

const VOLUME_SLIDER: &str = "w-16 sm:w-20 h-1.5 bg-white/20 rounded-full \
                             appearance-none cursor-pointer \
                             [&::-webkit-slider-thumb]:appearance-none \
                             [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 \
                             [&::-webkit-slider-thumb]:rounded-full \
                             [&::-webkit-slider-thumb]:bg-cyan-400";

const SEEK_SLIDER: &str = "flex-1 h-1.5 bg-white/20 rounded-full appearance-none \
                           cursor-pointer \
                           [&::-webkit-slider-thumb]:appearance-none \
                           [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 \
                           [&::-webkit-slider-thumb]:rounded-full \
                           [&::-webkit-slider-thumb]:bg-cyan-400 \
                           [&::-webkit-slider-thumb]:shadow-lg \
                           [&::-webkit-slider-thumb]:shadow-cyan-400/30";

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

// ─── Context marker types ─────────────────────────────────────────────────

impl ContextBundle for PlayerSignals {}
impl ContextBundle for PlayerDerived {}
impl ContextBundle for PlayerHandlers {}
impl ContextBundle for PlayerNav {}
impl ContextBundle for PlaylistConfig {}

// ─── Signal bundle ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerSignals {
    playing: RwSignal<bool>,
    current_time: RwSignal<f64>,
    duration: RwSignal<f64>,
    volume: RwSignal<f64>,
    last_volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    fullscreen: RwSignal<bool>,
    controls_visible: RwSignal<bool>,
    play_after_load: RwSignal<bool>,
}

impl PlayerSignals {
    fn new() -> Self {
        Self {
            playing: RwSignal::new(false),
            current_time: RwSignal::new(0.0),
            duration: RwSignal::new(0.0),
            volume: RwSignal::new(1.0),
            last_volume: RwSignal::new(1.0),
            muted: RwSignal::new(false),
            fullscreen: RwSignal::new(false),
            controls_visible: RwSignal::new(true),
            play_after_load: RwSignal::new(false),
        }
    }
}

// ─── Derived signals ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerDerived {
    current_src: Signal<String>,
    current_title: Signal<String>,
    current_artwork: Signal<Option<String>>,
    has_prev: Signal<bool>,
    has_next: Signal<bool>,
    has_playlist: Signal<bool>,
}

impl PlayerDerived {
    fn build(
        items: Signal<Vec<MediaItem>>,
        current_idx: RwSignal<usize>,
        show_playlist: bool,
        artwork_fallback: Option<String>,
    ) -> Self {
        let current_item = Memo::new(move |_| items.get().get(current_idx.get()).cloned());

        let current_src =
            Signal::derive(move || current_item.get().map(|i| i.src).unwrap_or_default());
        let current_title =
            Signal::derive(move || current_item.get().map(|i| i.title).unwrap_or_default());
        let current_artwork = {
            let fallback = artwork_fallback;
            Signal::derive(move || {
                current_item
                    .get()
                    .and_then(|i| i.artwork)
                    .or_else(|| fallback.clone())
            })
        };

        let has_prev = Signal::derive(move || current_idx.get() > 0);
        let has_next = Signal::derive(move || current_idx.get() + 1 < items.get().len());
        let has_playlist = Signal::derive(move || show_playlist && items.get().len() > 1);

        Self {
            current_src,
            current_title,
            current_artwork,
            has_prev,
            has_next,
            has_playlist,
        }
    }
}

// ─── Handlers ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerHandlers {
    toggle_play: Callback<MouseEvent>,
    toggle_mute: Callback<MouseEvent>,
    toggle_fullscreen: Callback<MouseEvent>,
    handle_seek: Callback<web_sys::Event>,
    handle_volume: Callback<web_sys::Event>,
}

// ─── Navigation callbacks ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerNav {
    on_prev: Callback<MouseEvent>,
    on_next: Callback<MouseEvent>,
}

// ─── Playlist config ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlaylistConfig {
    on_rename: Option<Callback<(u64, String)>>,
    on_delete: Option<Callback<u64>>,
    show_download: bool,
}

// ─── Effects ──────────────────────────────────────────────────────────────

fn install_clamp_effect(items: Signal<Vec<MediaItem>>, current_idx: RwSignal<usize>) {
    Effect::new(move |_| {
        let n = items.get().len();
        if n == 0 {
            current_idx.set(0);
        } else if current_idx.get() >= n {
            current_idx.set(n - 1);
        }
    });
}

fn install_src_reload_effect(
    video_ref: NodeRef<html::Video>,
    signals: PlayerSignals,
    current_src: Signal<String>,
) {
    Effect::new(move || {
        let src = current_src.get();
        if let Some(video) = video_ref.get() {
            video.set_src(&src);
            video.load();
            signals.playing.set(false);
            signals.current_time.set(0.0);
            signals.duration.set(0.0);
            if signals.play_after_load.get_untracked() {
                signals.play_after_load.set(false);
                let _ = video.play();
            }
        }
    });
}

// ─── Handler factories ────────────────────────────────────────────────────

fn make_toggle_play(
    video_ref: NodeRef<html::Video>,
    playing: RwSignal<bool>,
) -> impl Fn(MouseEvent) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if playing.get() {
                video.pause().ok();
            } else {
                let _ = video.play();
            }
        }
    }
}

fn make_handle_loaded_metadata(
    video_ref: NodeRef<html::Video>,
    duration: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            duration.set(video.duration());
        }
    }
}

fn make_handle_time_update(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            current_time.set(video.current_time());
        }
    }
}

fn make_handle_seek(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_current_time(val);
            current_time.set(val);
        }
    }
}

fn make_handle_volume(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    last_volume: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |ev: web_sys::Event| {
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
    }
}

fn make_toggle_mute(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    last_volume: RwSignal<f64>,
    muted: RwSignal<bool>,
) -> impl Fn(MouseEvent) + Copy {
    move |_| {
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
    }
}

fn make_toggle_fullscreen(video_ref: NodeRef<html::Video>) -> impl Fn(MouseEvent) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if document().fullscreen_element().is_none() {
                let _ = video.request_fullscreen();
            } else {
                document().exit_fullscreen();
            }
        }
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
    #[prop(default = true)] show_download: bool,
    #[prop(optional)] on_rename: Option<Callback<(u64, String)>>,
    #[prop(optional)] on_delete: Option<Callback<u64>>,
) -> impl IntoView {
    let current_idx = RwSignal::new(initial_index);
    let video_ref = NodeRef::<html::Video>::new();
    let signals = PlayerSignals::new();
    let derived = PlayerDerived::build(items, current_idx, show_playlist, artwork.clone());

    install_clamp_effect(items, current_idx);
    install_src_reload_effect(video_ref, signals, derived.current_src);

    // ── Controls visibility timeout ───────────────────────────────────
    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_i: i8| signals.controls_visible.set(false), 3000.);
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
            signals.controls_visible.set(true);
            start_hide_timer();
        }
    };
    let toggle_controls = {
        let show_controls = show_controls.clone();
        let stop = stop.clone();
        move || {
            if signals.controls_visible.get() {
                signals.controls_visible.set(false);
                stop();
            } else {
                show_controls();
            }
        }
    };

    // ── Fullscreen listener ───────────────────────────────────────────
    let u_document = use_document();
    let _guard = use_event_listener(u_document.clone(), fullscreenchange, move |_| {
        signals
            .fullscreen
            .set(u_document.fullscreen().is_some_and(|x| x));
    });

    // ── Navigation ────────────────────────────────────────────────────
    let on_next: Callback<MouseEvent> = Callback::new(move |_| {
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            current_idx.update(|i| *i += 1);
        }
    });
    let on_prev: Callback<MouseEvent> = Callback::new(move |_| {
        current_idx.update(|i| *i = i.saturating_sub(1));
    });
    let nav = PlayerNav { on_prev, on_next };

    let handle_ended = move |_| {
        signals.playing.set(false);
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            signals.play_after_load.set(true);
            current_idx.update(|i| *i += 1);
        }
    };

    // ── Build handler bundle ──────────────────────────────────────────
    let handlers = PlayerHandlers {
        toggle_play: Callback::new(make_toggle_play(video_ref, signals.playing)),
        toggle_mute: Callback::new(make_toggle_mute(
            video_ref,
            signals.volume,
            signals.last_volume,
            signals.muted,
        )),
        toggle_fullscreen: Callback::new(make_toggle_fullscreen(video_ref)),
        handle_seek: Callback::new(make_handle_seek(video_ref, signals.current_time)),
        handle_volume: Callback::new(make_handle_volume(
            video_ref,
            signals.volume,
            signals.muted,
            signals.last_volume,
        )),
    };

    // Direct handlers still needed on the `<video>` element itself.
    let handle_loaded_metadata = make_handle_loaded_metadata(video_ref, signals.duration);
    let handle_time_update = make_handle_time_update(video_ref, signals.current_time);

    // ── Install contexts (must precede view!) ─────────────────────────
    PlayerSignals::provide(signals);
    PlayerDerived::provide(derived);
    PlayerHandlers::provide(handlers);
    PlayerNav::provide(nav);

    // ── View ──────────────────────────────────────────────────────────
    let video_class = if audio {
        "w-full h-0 pointer-events-none"
    } else {
        "w-full h-auto max-h-[60vh] md:max-h-[70vh] object-contain cursor-pointer"
    };

    let artwork_view = if audio {
        let tc = toggle_controls.clone();
        Some(view! {
            <AudioArtworkOverlay
                artwork=derived.current_artwork
                on_click=move |_| tc()
            />
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
                title=move || derived.current_title.get()
                class=video_class
                on:loadedmetadata=handle_loaded_metadata
                on:timeupdate=handle_time_update
                on:play=move |_| signals.playing.set(true)
                on:pause=move |_| signals.playing.set(false)
                on:ended=handle_ended
                on:click={let tc = toggle_controls.clone(); move |_| tc()}
                playsinline
            />
            {artwork_view}
            <MediaControls
                show_controls=show_controls.clone()
                start_hide_timer=start_hide_timer
                show_fullscreen=!audio
            />
        </div>
    };

    let playlist_view = view! {
        <Show when=move || derived.has_playlist.get()>
            <aside class="w-full lg:w-80 xl:w-96 lg:flex-shrink-0">
                <PlaylistPanel
                    items=items
                    current_idx=current_idx
                    title=playlist_title.clone()
                    on_rename=on_rename
                    on_delete=on_delete
                    show_download=show_download
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

// ─── Audio artwork overlay ────────────────────────────────────────────────

#[component]
fn AudioArtworkOverlay(
    artwork: Signal<Option<String>>,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div
            class="relative w-full aspect-video flex items-center justify-center \
                   bg-gradient-to-br from-[#1e1e2e] via-[#14141e] to-[#0a0a0f] \
                   cursor-pointer overflow-hidden"
            on:click=on_click
        >
            {move || artwork.get().map(|url| view! {
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
                    class="relative max-h-[70%] max-w-[70%] object-contain \
                           rounded-2xl shadow-2xl shadow-black/70 border border-white/10"
                    alt=""
                />
            })}
        </div>
    }
}

// ─── Controls overlay ─────────────────────────────────────────────────────
// Reads Signals / Derived / Handlers / Nav from context. Only receives what
// is genuinely local: the two timeout closures, and the audio-mode flag.

#[component]
fn MediaControls(
    show_controls: impl Fn() + Clone + 'static,
    start_hide_timer: impl Fn() + Clone + 'static,
    #[prop(default = true)] show_fullscreen: bool,
) -> impl IntoView {
    let signals = PlayerSignals::expect();
    let derived = PlayerDerived::expect();

    let class = move || {
        format!(
            "absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/50 \
             to-transparent p-3 sm:p-5 transition-opacity duration-300 {}",
            if signals.controls_visible.get() {
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
                    {move || derived.current_title.get()}
                </div>
                <SeekBar/>
                <ControlButtons show_fullscreen=show_fullscreen/>
            </div>
        </div>
    }
}

// ─── Button row ───────────────────────────────────────────────────────────

#[component]
fn ControlButtons(#[prop(default = true)] show_fullscreen: bool) -> impl IntoView {
    let fullscreen_btn = show_fullscreen.then(|| {
        view! {
            <FullscreenButton/>
        }
    });

    view! {
        <div class="flex items-center gap-2 sm:gap-3 text-white">
            <NavButton direction=NavDirection::Prev/>
            <PlayPauseButton/>
            <NavButton direction=NavDirection::Next/>
            <div class="flex items-center gap-2">
                <MuteButton/>
                <VolumeSlider/>
            </div>
            <div class="flex-1"></div>
            {fullscreen_btn}
        </div>
    }
}

// ─── Individual buttons ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum NavDirection {
    Prev,
    Next,
}

impl NavDirection {
    fn aria_label(self) -> &'static str {
        match self {
            Self::Prev => "السابق",
            Self::Next => "التالي",
        }
    }
}

#[component]
fn NavButton(direction: NavDirection) -> impl IntoView {
    let derived = PlayerDerived::expect();
    let nav = PlayerNav::expect();

    let show = derived.has_playlist;
    let disabled = Signal::derive(move || match direction {
        NavDirection::Prev => !derived.has_prev.get(),
        NavDirection::Next => !derived.has_next.get(),
    });
    let on_click = match direction {
        NavDirection::Prev => nav.on_prev,
        NavDirection::Next => nav.on_next,
    };
    let label = direction.aria_label();

    view! {
        <Show when=move || show.get()>
            <button
                on:click=move |ev| on_click.run(ev)
                disabled=move || disabled.get()
                class=NAV_BUTTON
                aria-label=label
            >
                {move || match direction {
                    NavDirection::Prev => Either::Left(view! { <PrevPageIcon/> }),
                    NavDirection::Next => Either::Right(view! { <NextPageIcon/> }),
                }}
            </button>
        </Show>
    }
}

#[component]
fn PlayPauseButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            on:click=move |ev| handlers.toggle_play.run(ev)
            class=ICON_BUTTON
            aria-label="تشغيل / إيقاف"
        >
            {move || if signals.playing.get() {
                Either::Left(PauseIcon())
            } else {
                Either::Right(PlayIcon())
            }}
        </button>
    }
}

#[component]
fn MuteButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            on:click=move |ev| handlers.toggle_mute.run(ev)
            class=ICON_BUTTON
            aria-label="كتم / إلغاء"
        >
            {move || if signals.muted.get() || signals.volume.get() == 0.0 {
                Either::Left(MuteIcon())
            } else {
                Either::Right(VolumeIcon())
            }}
        </button>
    }
}

#[component]
fn FullscreenButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            on:click=move |ev| handlers.toggle_fullscreen.run(ev)
            class=ICON_BUTTON
            aria-label="ملء الشاشة"
        >
            {move || if signals.fullscreen.get() {
                Either::Left(FullscreenExitIcon())
            } else {
                Either::Right(FullscreenIcon())
            }}
        </button>
    }
}

#[component]
fn VolumeSlider() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    let vol_value = move || {
        if signals.muted.get() {
            0.0
        } else {
            signals.volume.get()
        }
    };

    view! {
        <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            prop:value=vol_value
            on:input=move |ev| handlers.handle_volume.run(ev)
            class=VOLUME_SLIDER
        />
    }
}

// ─── Seek bar ─────────────────────────────────────────────────────────────

#[component]
fn SeekBar() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <div class="flex items-center gap-2">
            <span class="text-white text-xs font-mono">
                {move || format_time(signals.current_time.get())}
            </span>
            <input
                type="range"
                min="0"
                prop:max=signals.duration
                prop:value=signals.current_time
                on:input=move |ev| handlers.handle_seek.run(ev)
                class=SEEK_SLIDER
            />
            <span class="text-white text-xs font-mono">
                {move || format_time(signals.duration.get())}
            </span>
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
    #[prop(default = true)] show_download: bool,
) -> impl IntoView {
    // Config only needs to reach PlaylistItem; scope it here so the panel
    // owns it and the item can read it without an argument.
    PlaylistConfig::provide(PlaylistConfig {
        on_rename,
        on_delete,
        show_download,
    });

    let title = title.unwrap_or_else(|| "قائمة التشغيل".to_string());

    view! {
        <div class="bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10 \
                    overflow-hidden flex flex-col max-h-[60vh] lg:max-h-[500px]">
            <div class="px-4 py-3 border-b border-white/10 flex items-center \
                        justify-between shrink-0">
                <h3 class="text-sm font-bold text-white truncate">{title}</h3>
                <span class="text-xs text-gray-400 font-mono bg-white/10 px-2 py-0.5 rounded-full">
                    {move || items.get().len()}
                </span>
            </div>
            <div class="overflow-y-auto p-2 flex-1 min-h-0">
                <For
                    each=move || items.get().into_iter().enumerate()
                    key=|(_, item)| item.id
                    let:((index,item))
                >
                    <PlaylistItem item=item index=index current_idx=current_idx/>
                </For>
            </div>
        </div>
    }
}

#[component]
fn PlaylistItem(item: MediaItem, index: usize, current_idx: RwSignal<usize>) -> impl IntoView {
    let config = PlaylistConfig::expect();

    let id = item.id;
    let download_src = item.src.clone();
    let download_name = item.title.clone();

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

    let do_commit: Callback<()> = Callback::new(move |_| {
        editing.set(false);
        let new_title = draft.get_untracked();
        if new_title != title.get_value()
            && let Some(cb) = config.on_rename
        {
            cb.run((id, new_title));
        }
    });

    let on_delete_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        if let Some(cb) = config.on_delete {
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
            "w-full flex items-center gap-2 p-2 rounded-lg cursor-pointer \
             transition text-right group/row {}",
            if is_current() {
                "bg-cyan-500/15 border border-cyan-500/30"
            } else {
                "border border-transparent hover:bg-white/5"
            }
        )
    };

    let edit_on = use_edit_mode();
    let can_rename = config.on_rename.is_some();
    let can_delete = config.on_delete.is_some();
    let show_download = config.show_download;

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
                        class="w-full bg-white/10 text-white text-sm rounded px-2 py-1 \
                               focus:outline-none focus:ring-1 focus:ring-cyan-400"
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

            <Show when=move || show_download>
                <PlaylistDownloadLink
                    href=download_src.clone()
                    download_name=download_name.clone()
                />
            </Show>

            <Show when=move || edit_on.get() && can_rename>
                <button
                    type="button"
                    on:click=begin_edit
                    class="opacity-0 group-hover/row:opacity-100 transition \
                           text-gray-400 hover:text-white p-1 shrink-0"
                    aria-label="إعادة تسمية"
                >
                    <EditIcon/>
                </button>
            </Show>

            <Show when=move || edit_on.get() && can_delete>
                <button
                    type="button"
                    on:click=on_delete_click
                    class="opacity-0 group-hover/row:opacity-100 transition \
                           text-red-400 hover:text-red-300 p-1 shrink-0"
                    aria-label="حذف"
                >
                    <DeleteIcon/>
                </button>
            </Show>
        </div>
    }
}

#[component]
fn PlaylistDownloadLink(
    #[prop(into)] href: String,
    #[prop(into)] download_name: String,
) -> impl IntoView {
    view! {
        <a
            href=href
            download=download_name
            class="opacity-0 group-hover/row:opacity-100 transition \
                   text-gray-400 hover:text-white p-1 shrink-0"
            aria-label="تحميل"
            on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
        >
            <DownloadIcon/>
        </a>
    }
}

#[component]
fn PlaylistIndicator(index: usize, #[prop(into)] is_current: Signal<bool>) -> impl IntoView {
    view! {
        {move || if is_current.get() {
            Either::Left(view! {
                <span class="flex items-center justify-center w-8 h-8 rounded-full \
                             bg-cyan-500/20 text-cyan-400 shrink-0">
                    <PlayIcon/>
                </span>
            })
        } else {
            Either::Right(view! {
                <span class="flex items-center justify-center w-8 h-8 rounded-full \
                             bg-white/5 text-gray-300 text-xs font-bold shrink-0">
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
