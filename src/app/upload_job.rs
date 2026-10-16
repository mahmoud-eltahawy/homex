use crate::app::error::AppError;
use crate::app::types::JobId;
use crate::app::upload_api::{ConversionStatus, UploadResult, poll_conversion, upload_media};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_outcome::ErrorView;
use leptos_outcome::action_error;
use uuid::Uuid;

const POLL_INTERVAL_MS: u32 = 700;
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

#[derive(Clone, Copy)]
pub struct UploadJob {
    action: Action<web_sys::FormData, Result<UploadResult, AppError>>,
    pub status: RwSignal<Option<ConversionStatus>>,
    pub done_tick: RwSignal<u32>,
    pub pending: Signal<bool>,
    pub collection_id: RwSignal<Option<Uuid>>,
}

#[derive(Clone, Copy)]
struct UploadSignals {
    status: RwSignal<Option<ConversionStatus>>,
    done_tick: RwSignal<u32>,
    active_job: RwSignal<Option<JobId>>,
    collection_id: RwSignal<Option<Uuid>>,
    section_slug: RwSignal<Option<String>>,
}

impl UploadSignals {
    fn new() -> Self {
        Self {
            status: RwSignal::new(None),
            done_tick: RwSignal::new(0),
            active_job: RwSignal::new(None),
            collection_id: RwSignal::new(None),
            section_slug: RwSignal::new(None),
        }
    }

    fn bump_done(&self) {
        self.done_tick.update(|n| *n += 1);
    }

    fn clear_active_if(&self, id: JobId) {
        if self.active_job.get_untracked() == Some(id) {
            self.active_job.set(None);
        }
    }

    fn mark_done_if_successful(&self) {
        if matches!(self.status.get_untracked(), Some(ConversionStatus::Done)) {
            self.bump_done();
        }
    }
}

fn is_terminal(s: &ConversionStatus) -> bool {
    matches!(
        s,
        ConversionStatus::Done | ConversionStatus::Cancelled | ConversionStatus::Failed(_)
    )
}

async fn poll_until_terminal(job_id: JobId, signals: UploadSignals) {
    let mut consecutive_errors = 0u32;
    loop {
        match poll_conversion(job_id).await {
            Ok(s) => {
                consecutive_errors = 0;
                let terminal = is_terminal(&s);
                signals.status.set(Some(s));
                if terminal {
                    break;
                }
            }
            Err(_) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    signals.status.set(Some(ConversionStatus::Failed(
                        AppError::detailed("Lost contact with the server").transient(),
                    )));
                    break;
                }
            }
        }
        TimeoutFuture::new(POLL_INTERVAL_MS).await;
    }
    signals.clear_active_if(job_id);
    signals.mark_done_if_successful();
}

fn install_dispatch_result_effect(
    action: Action<web_sys::FormData, Result<UploadResult, AppError>>,
    signals: UploadSignals,
) {
    Effect::new(move |_| {
        let Some(Ok(result)) = action.value().get() else {
            return;
        };
        signals.collection_id.set(Some(result.collection_id));
        signals.section_slug.set(Some(result.section_slug));
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
    action: Action<web_sys::FormData, Result<UploadResult, AppError>>,
    signals: UploadSignals,
) -> Signal<bool> {
    Signal::derive(move || action.pending().get() || signals.active_job.get().is_some())
}

impl UploadJob {
    pub fn new() -> Self {
        Self::build()
    }

    #[doc(alias = "new")]
    pub fn new_for_new_collection() -> Self {
        Self::build()
    }

    fn build() -> Self {
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
            collection_id: signals.collection_id,
        }
    }

    pub fn dispatch(&self, fd: web_sys::FormData) {
        self.action.dispatch(fd);
    }

    pub fn error(&self) -> Signal<Option<AppError>> {
        action_error(self.action)
    }

    pub fn is_terminal_success(&self) -> bool {
        self.done_tick.get() > 0
    }
}

#[component]
pub fn UploadProgress(status: Signal<Option<ConversionStatus>>) -> impl IntoView {
    view! {
        <Show when=move || status.get().is_some()>
            {move || status.get().map(|s| match s {
                ConversionStatus::Writing => view! { <WritingStage/> }.into_any(),
                ConversionStatus::Moving {
                    current_file,
                    progress,
                } => view! {
                    <MovingStage current_file=current_file progress=progress/>
                }
                .into_any(),
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
                ConversionStatus::Cancelled => view! { <CancelledStage/> }.into_any(),
                ConversionStatus::Failed(e) => view! { <FailedStage error=e/> }.into_any(),
            })}
        </Show>
    }
}

#[component]
fn WritingStage() -> impl IntoView {
    view! {
        <div class="text-cyan-300 text-sm">"💾 Saving files..."</div>
    }
}

#[component]
fn FinalizingStage() -> impl IntoView {
    view! {
        <div class="text-cyan-300 text-sm">"💾 Saving data..."</div>
    }
}

#[component]
fn DoneStage() -> impl IntoView {
    view! {
        <div class="bg-green-500/15 border border-green-500/30 rounded-xl p-3 \
                    text-green-300 text-sm">
            "✓ Conversion successful"
        </div>
    }
}

#[component]
fn CancelledStage() -> impl IntoView {
    view! {
        <div class="bg-gray-500/15 border border-gray-500/30 rounded-xl p-3 \
                    text-gray-300 text-sm">
            "Cancelled"
        </div>
    }
}

#[component]
fn FailedStage(error: AppError) -> impl IntoView {
    let signal = Signal::derive(move || Some(error.clone()));
    view! {
        <ErrorView
            error=signal
            banner=true
            heading="Conversion failed"
        />
    }
}

#[component]
fn MovingStage(#[prop(into)] current_file: String, progress: f32) -> impl IntoView {
    let pct = if progress.is_finite() {
        (progress * 100.0).round() as u32
    } else {
        0
    };
    view! {
        <div class="bg-cyan-500/10 border border-cyan-500/30 rounded-xl p-4 space-y-2">
            <div class="text-cyan-300 text-sm font-bold">"📦 Moving file..."</div>
            <div class="text-xs text-gray-400 truncate font-mono">{current_file}</div>
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
            "File {} of {} — {}",
            conversion_index + 1,
            conversion_count,
            current_file
        )
    } else {
        current_file
    };

    view! {
        <div class="bg-cyan-500/10 border border-cyan-500/30 rounded-xl p-4 space-y-2">
            <div class="text-cyan-300 text-sm font-bold">"🎬 Converting file..."</div>
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
