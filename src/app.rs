use crate::app::{
    audio::{detail::AudioGroupDetailPage, song::AudioSongDetailPage},
    home::HomePage,
    layout::Layout,
    listing::{AudioGroupListingPage, MovieListingPage, SeriesListingPage},
    movies::detail::MovieDetailPage,
    series::details::SeriesDetailPage,
    settings::SettingsPage,
};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    Lazy,
    components::{ParentRoute, Route, Router, Routes},
    path,
};

mod audio;
mod common;
pub mod detail;
mod home;
pub mod icons;
mod inline_edit;
mod layout;
mod listing;
mod media_api;
mod media_player;
mod model;
mod movies;
mod pagination;
mod resource_view;
mod route_params;
mod search;
mod series;
#[cfg(feature = "ssr")]
pub mod server;
mod settings;
mod upload_api;
mod view_schema;
mod upload_job {
    use gloo_timers::future::TimeoutFuture;
    use leptos::prelude::*;

    use crate::app::upload_api::{ConversionStatus, UploadResult, poll_conversion, upload_media};

    /// A tiny state machine for one media-upload job.
    /// Owns the `Action` that dispatches the FormData, the polling loop,
    /// and a signal that fires once when the job reaches `Done`.
    #[derive(Clone, Copy)]
    pub struct UploadJob {
        action: Action<web_sys::FormData, Result<UploadResult, ServerFnError>>,
        pub status: RwSignal<Option<ConversionStatus>>,
        /// Monotonic counter — bumped each time a job finishes successfully.
        /// Pages watch this in an `Effect` to refetch their list.
        pub done_tick: RwSignal<u32>,
        pub pending: Signal<bool>,
    }

    impl UploadJob {
        pub fn new() -> Self {
            let action =
                Action::new_local(|fd: &web_sys::FormData| upload_media(fd.clone().into()));

            let status = RwSignal::new(None::<ConversionStatus>);
            let done_tick = RwSignal::new(0u32);
            let active_job = RwSignal::new(None::<String>);

            // When the server hands us a job id, start polling it.
            Effect::new(move |_| {
                let Some(Ok(result)) = action.value().get() else {
                    return;
                };
                if let Some(id) = result.job_id {
                    active_job.set(Some(id));
                } else {
                    // Fast path — no conversion, job is already done.
                    done_tick.update(|n| *n += 1);
                }
            });

            // Poll until terminal.
            Effect::new(move |_| {
                let Some(my_id) = active_job.get() else {
                    return;
                };
                status.set(None);

                leptos::task::spawn_local(async move {
                    loop {
                        match poll_conversion(my_id.clone()).await {
                            Ok(s) => {
                                let terminal = matches!(
                                    s,
                                    ConversionStatus::Done | ConversionStatus::Failed(_)
                                );
                                status.set(Some(s));
                                if terminal {
                                    break;
                                }
                            }
                            Err(_) => { /* transient */ }
                        }
                        TimeoutFuture::new(700).await;
                    }

                    // Clear the active job id so we don't re-poll.
                    if active_job.get_untracked().as_deref() == Some(my_id.as_str()) {
                        active_job.set(None);
                    }

                    if matches!(status.get_untracked(), Some(ConversionStatus::Done)) {
                        done_tick.update(|n| *n += 1);
                    }
                });
            });

            let pending =
                Signal::derive(move || action.pending().get() || active_job.get().is_some());

            Self {
                action,
                status,
                done_tick,
                pending,
            }
        }

        pub fn dispatch(&self, fd: web_sys::FormData) {
            self.action.dispatch(fd);
        }

        pub fn error(&self) -> Signal<Option<String>> {
            let action = self.action;
            Signal::derive(move || match action.value().get() {
                Some(Err(e)) => Some(e.to_string()),
                _ => None,
            })
        }
    }

    // ─── Progress component (used by every detail page) ─────────────────────────

    #[component]
    pub fn ConversionProgress() -> impl IntoView {
        // Caller passes the `UploadJob` in via context-free prop for simplicity.
        // Actually — to keep it usable from anywhere, take the signal directly:
        // see `UploadProgress` below.
        view! { <></> }
    }

    #[component]
    pub fn UploadProgress(status: Signal<Option<ConversionStatus>>) -> impl IntoView {
        view! {
            <Show when=move || status.get().is_some()>
                {move || status.get().map(|s| match s {
                    ConversionStatus::Writing => view! {
                        <div class="text-cyan-300 text-sm">"💾 جاري حفظ الملفات..."</div>
                    }.into_any(),

                    ConversionStatus::Converting {
                        conversion_index,
                        conversion_count,
                        current_file,
                        progress,
                    } => {
                        let pct = (progress * 100.0).round() as u32;
                        let subtitle = if conversion_count > 1 {
                            format!("ملف {} من {} — {}", conversion_index + 1, conversion_count, current_file)
                        } else {
                            current_file
                        };
                        view! {
                            <div class="bg-cyan-500/10 border border-cyan-500/30 rounded-xl p-4 space-y-2">
                                <div class="text-cyan-300 text-sm font-bold">"🎬 جاري تحويل الملف..."</div>
                                <div class="text-xs text-gray-400 truncate font-mono">{subtitle}</div>
                                <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
                                    <div
                                        class="h-full bg-gradient-to-r from-cyan-400 to-blue-500 rounded-full transition-all duration-300"
                                        style=format!("width: {pct}%")
                                    ></div>
                                </div>
                                <div class="text-xs text-cyan-300 font-mono text-right">{pct}"%"</div>
                            </div>
                        }.into_any()
                    }

                    ConversionStatus::Finalizing => view! {
                        <div class="text-cyan-300 text-sm">"💾 جاري حفظ البيانات..."</div>
                    }.into_any(),

                    ConversionStatus::Done => view! {
                        <div class="bg-green-500/15 border border-green-500/30 rounded-xl p-3 text-green-300 text-sm">
                            "✓ تم التحويل بنجاح"
                        </div>
                    }.into_any(),

                    ConversionStatus::Failed(e) => view! {
                        <div class="bg-red-500/15 border border-red-500/30 rounded-xl p-3 text-red-300 text-sm">
                            <div class="font-bold mb-1">"فشل التحويل"</div>
                            <div class="text-xs break-all">{e}</div>
                        </div>
                    }.into_any(),
                })}
            </Show>
        }
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ar" dir="rtl">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/mydisc.css"/>
        <Title text="وسائطي - سينماك الشخصية"/>
        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <ParentRoute path=path!("") view=Layout>
                    <Route
                        path=path!("/")
                        view={Lazy::<HomePage>::new()}
                    />
                    <Route
                        path=path!("/series/detail/:id")
                        view={Lazy::<SeriesDetailPage>::new()}
                    />
                    <Route
                        path=path!("/movie/detail/:id")
                        view={Lazy::<MovieDetailPage>::new()}
                    />
                    <Route
                        path=path!("/movie")
                        view={Lazy::<MovieListingPage>::new()}
                    />
                    <Route
                        path=path!("/series")
                        view={Lazy::<SeriesListingPage>::new()}
                    />
                    <Route
                        path=path!("/audio")
                        view={Lazy::<AudioGroupListingPage>::new()}
                    />
                    <Route
                        path=path!("/settings")
                        view={Lazy::<SettingsPage>::new()}
                    />
                    <Route
                        path=path!("/audio/detail/:id")
                        view={Lazy::<AudioGroupDetailPage>::new()}
                    />
                    <Route
                        path=path!("/audio/detail/:id/song/:song_id")
                        view={Lazy::<AudioSongDetailPage>::new()}
                    />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
