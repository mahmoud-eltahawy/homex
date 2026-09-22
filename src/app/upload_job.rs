use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;

use crate::app::upload_api::{ConversionStatus, UploadResult, poll_conversion, upload_media};

const POLL_INTERVAL_MS: u32 = 700;

#[derive(Clone, Copy)]
pub struct UploadJob {
    action: Action<web_sys::FormData, Result<UploadResult, ServerFnError>>,
    pub status: RwSignal<Option<ConversionStatus>>,
    pub done_tick: RwSignal<u32>,
    pub pending: Signal<bool>,
}

// ─── Signal bundle ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct UploadSignals {
    status: RwSignal<Option<ConversionStatus>>,
    done_tick: RwSignal<u32>,
    active_job: RwSignal<Option<String>>,
}

impl UploadSignals {
    fn new() -> Self {
        Self {
            status: RwSignal::new(None),
            done_tick: RwSignal::new(0),
            active_job: RwSignal::new(None),
        }
    }

    fn bump_done(&self) {
        self.done_tick.update(|n| *n += 1);
    }

    fn clear_active_if(&self, id: &str) {
        if self.active_job.get_untracked().as_deref() == Some(id) {
            self.active_job.set(None);
        }
    }

    fn mark_done_if_successful(&self) {
        if matches!(self.status.get_untracked(), Some(ConversionStatus::Done)) {
            self.bump_done();
        }
    }
}

// ─── Polling ──────────────────────────────────────────────────────────────

fn is_terminal(s: &ConversionStatus) -> bool {
    matches!(s, ConversionStatus::Done | ConversionStatus::Failed(_))
}

async fn poll_until_terminal(job_id: String, signals: UploadSignals) {
    loop {
        match poll_conversion(job_id.clone()).await {
            Ok(s) => {
                let terminal = is_terminal(&s);
                signals.status.set(Some(s));
                if terminal {
                    break;
                }
            }
            Err(_) => { /* transient */ }
        }
        TimeoutFuture::new(POLL_INTERVAL_MS).await;
    }
    signals.clear_active_if(&job_id);
    signals.mark_done_if_successful();
}

// ─── Effects ──────────────────────────────────────────────────────────────

fn install_dispatch_result_effect(
    action: Action<web_sys::FormData, Result<UploadResult, ServerFnError>>,
    signals: UploadSignals,
) {
    Effect::new(move |_| {
        let Some(Ok(result)) = action.value().get() else {
            return;
        };
        match result.job_id {
            Some(id) => signals.active_job.set(Some(id)),
            None => signals.bump_done(),
        }
    });
}

fn install_polling_effect(signals: UploadSignals) {
    Effect::new(move |_| {
        let Some(id) = signals.active_job.get() else {
            return;
        };
        signals.status.set(None);
        leptos::task::spawn_local(poll_until_terminal(id, signals));
    });
}

fn derive_pending(
    action: Action<web_sys::FormData, Result<UploadResult, ServerFnError>>,
    signals: UploadSignals,
) -> Signal<bool> {
    Signal::derive(move || action.pending().get() || signals.active_job.get().is_some())
}

// ─── Public ───────────────────────────────────────────────────────────────

impl UploadJob {
    pub fn new() -> Self {
        let action = Action::new_local(|fd: &web_sys::FormData| upload_media(fd.clone().into()));
        let signals = UploadSignals::new();

        install_dispatch_result_effect(action, signals);
        install_polling_effect(signals);

        let pending = derive_pending(action, signals);

        Self {
            action,
            status: signals.status,
            done_tick: signals.done_tick,
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

// ─── Progress view ────────────────────────────────────────────────────────

#[component]
pub fn UploadProgress(status: Signal<Option<ConversionStatus>>) -> impl IntoView {
    view! {
        <Show when=move || status.get().is_some()>
            {move || status.get().map(|s| match s {
                ConversionStatus::Writing => view! { <WritingStage/> }.into_any(),
                ConversionStatus::Converting {
                    conversion_index,
                    conversion_count,
                    current_file,
                    progress,
                } => view! {
                    <ConvertingStage
                        conversion_index=conversion_index
                        conversion_count=conversion_count
                        current_file=current_file
                        progress=progress
                    />
                }
                .into_any(),
                ConversionStatus::Finalizing => view! { <FinalizingStage/> }.into_any(),
                ConversionStatus::Done => view! { <DoneStage/> }.into_any(),
                ConversionStatus::Failed(e) => view! { <FailedStage error=e/> }.into_any(),
            })}
        </Show>
    }
}

#[component]
fn WritingStage() -> impl IntoView {
    view! {
        <div class="text-cyan-300 text-sm">"💾 جاري حفظ الملفات..."</div>
    }
}

#[component]
fn FinalizingStage() -> impl IntoView {
    view! {
        <div class="text-cyan-300 text-sm">"💾 جاري حفظ البيانات..."</div>
    }
}

#[component]
fn DoneStage() -> impl IntoView {
    view! {
        <div class="bg-green-500/15 border border-green-500/30 rounded-xl p-3 \
                    text-green-300 text-sm">
            "✓ تم التحويل بنجاح"
        </div>
    }
}

#[component]
fn FailedStage(#[prop(into)] error: String) -> impl IntoView {
    view! {
        <div class="bg-red-500/15 border border-red-500/30 rounded-xl p-3 \
                    text-red-300 text-sm">
            <div class="font-bold mb-1">"فشل التحويل"</div>
            <div class="text-xs break-all">{error}</div>
        </div>
    }
}

#[component]
fn ConvertingStage(
    conversion_index: usize,
    conversion_count: usize,
    #[prop(into)] current_file: String,
    progress: f32,
) -> impl IntoView {
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
                    class="h-full bg-gradient-to-r from-cyan-400 to-blue-500 \
                           rounded-full transition-all duration-300"
                    style=format!("width: {pct}%")
                ></div>
            </div>
            <div class="text-xs text-cyan-300 font-mono text-right">{pct}"%"</div>
        </div>
    }
}
