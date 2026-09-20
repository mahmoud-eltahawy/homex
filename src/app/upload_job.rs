use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;

use crate::app::upload_api::{ConversionStatus, UploadResult, poll_conversion, upload_media};

#[derive(Clone)]
pub struct UploadJob {
    action: Action<web_sys::FormData, Result<UploadResult, ServerFnError>>,
    pub status: RwSignal<Option<ConversionStatus>>,
    pub done_tick: RwSignal<u32>,
    pub pending: Signal<bool>,
}

impl UploadJob {
    pub fn new() -> Self {
        let action = Action::new_local(|fd: &web_sys::FormData| upload_media(fd.clone().into()));

        let status = RwSignal::new(None::<ConversionStatus>);
        let done_tick = RwSignal::new(0u32);
        let active_job = RwSignal::new(None::<String>);

        // Adopt the job id returned by the server.
        {
            let status = status;
            let done_tick = done_tick;
            let active_job = active_job;
            Effect::new(move |_| {
                let Some(Ok(result)) = action.value().get() else {
                    return;
                };
                match result.job_id {
                    Some(id) => active_job.set(Some(id)),
                    None => done_tick.update(|n| *n += 1), // fast path, no conversion
                }
            });
        }

        // Poll until terminal.
        {
            let status = status;
            let done_tick = done_tick;
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

                    if active_job.get_untracked().as_deref() == Some(my_id.as_str()) {
                        active_job.set(None);
                    }

                    if matches!(status.get_untracked(), Some(ConversionStatus::Done)) {
                        done_tick.update(|n| *n += 1);
                    }
                });
            });
        }

        let pending = Signal::derive(move || action.pending().get() || active_job.get().is_some());

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
        let action = self.action.clone();
        Signal::derive(move || match action.value().get() {
            Some(Err(e)) => Some(e.to_string()),
            _ => None,
        })
    }
}

// ─── Progress UI ────────────────────────────────────────────────────────────

#[component]
pub fn UploadProgress(status: Signal<Option<ConversionStatus>>) -> impl IntoView {
    view! {
        <Show when=move || status.get().is_some()>
            {move || {
                status.get().map(|s| match s {
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
                            format!(
                                "ملف {} من {} — {}",
                                conversion_index + 1,
                                conversion_count,
                                current_file
                            )
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
                })
            }}
        </Show>
    }
}
