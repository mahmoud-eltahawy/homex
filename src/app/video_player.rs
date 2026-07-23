use crate::app::icons::{
    FullscreenExitIcon, FullscreenIcon, MuteIcon, PauseIcon, PlayIcon, VolumeIcon,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, ev::fullscreenchange};
use leptos_use::{use_document, use_event_listener, use_timeout_fn, UseTimeoutFnReturn};
use web_sys::{HtmlInputElement, MouseEvent};

#[component]
pub fn VideoPlayer(src: Signal<String>, #[prop(optional)] title: Option<String>) -> impl IntoView {
    let video_ref = NodeRef::<leptos::html::Video>::new();
    let playing = RwSignal::new(false);
    let current_time = RwSignal::new(0.0);
    let duration = RwSignal::new(0.0);
    let volume = RwSignal::new(1.0);
    let last_volume = RwSignal::new(1.0);
    let muted = RwSignal::new(false);
    let fullscreen = RwSignal::new(false);
    let controls_visible = RwSignal::new(true);

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
        {
            if let Ok(val) = input.value().parse::<f64>() {
                if let Some(video) = video_ref.get() {
                    video.set_current_time(val);
                    current_time.set(val);
                }
            }
        }
    };
    let handle_volume = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            if let Ok(val) = input.value().parse::<f64>() {
                if let Some(video) = video_ref.get() {
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
    Effect::new(move || {
        if let Some(video) = video_ref.get() {
            video.set_src(&src.get());
            video.load();
            playing.set(false);
            current_time.set(0.0);
            duration.set(0.0);
        }
    });

    view! {
    <div
        on:mousemove={let show = show_controls.clone(); move |_| show()}
        dir="ltr"
        class="relative bg-black rounded-2xl overflow-hidden shadow-2xl shadow-black/50 group"
    >
        <VideoElement
            video_ref=video_ref
            title=title
            playing=playing
            handle_loaded_metadata=handle_loaded_metadata
            handle_time_update=handle_time_update
            toggle_controls=toggle_controls
        />
        <VideoControls
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
        />
    </div>
    }
}

#[component]
fn VideoElement(
    video_ref: NodeRef<leptos::html::Video>,
    title: Option<String>,
    playing: RwSignal<bool>,
    handle_loaded_metadata: impl Fn(web_sys::Event) + 'static,
    handle_time_update: impl Fn(web_sys::Event) + 'static,
    toggle_controls: impl Fn() + Clone + 'static,
) -> impl IntoView {
    view! {
    <video
        node_ref=video_ref
        title=title
        class="w-full h-auto max-h-[60vh] md:max-h-[70vh] object-contain cursor-pointer"
        on:loadedmetadata=handle_loaded_metadata
        on:timeupdate=handle_time_update
        on:play=move |_| playing.set(true) on:pause=move |_| playing.set(false)
        on:ended=move |_| playing.set(false) on:click=move |_| toggle_controls() playsinline
    /> }
}

#[component]
pub fn VideoControls(
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
) -> impl IntoView {
    let class = move || {
        format!("absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3 sm:p-5 transition-opacity duration-300 {}", if controls_visible.get() { "opacity-100" } else { "opacity-0" })
    };
    let on_mouse_leave = {
        let start = start_hide_timer.clone();
        move |_| {
            start();
        }
    };
    view! {
    <div
        class=class
        on:mouseenter={let show = show_controls.clone(); move |_| show()}
        on:mouseleave={on_mouse_leave}
        on:touchstart={let show = show_controls.clone(); move |_| show()}>
        <div class="flex flex-col gap-2">
            <SeekBar
                current_time=current_time
                duration=duration
                handle_seek=handle_seek
            />
            <ControlButtons playing=playing muted=muted volume=volume fullscreen=fullscreen
                toggle_play=toggle_play toggle_mute=toggle_mute toggle_fullscreen=toggle_fullscreen handle_volume=handle_volume/>
        </div>
    </div> }
}

#[component]
pub fn SeekBar(
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
            <span class="text-white text-xs font-mono">{move || format_time(duration.get())}</span>
        </div>
    }
}

#[component]
pub fn ControlButtons(
    playing: RwSignal<bool>,
    muted: RwSignal<bool>,
    volume: RwSignal<f64>,
    fullscreen: RwSignal<bool>,
    toggle_play: impl Fn(MouseEvent) + 'static,
    toggle_mute: impl Fn(MouseEvent) + 'static,
    toggle_fullscreen: impl Fn(MouseEvent) + 'static,
    handle_volume: impl Fn(web_sys::Event) + 'static,
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
    view! {
    <div class="flex items-center gap-4 text-white">
        <button
            on:click=toggle_play
            class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
        >
            {play_icon}
        </button>
        <div class="flex items-center gap-2">
            <button
                on:click=toggle_mute
                class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
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
        <button
            on:click=toggle_fullscreen
            class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10"
        >
            {full_screen}
        </button>
    </div> }
}

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
