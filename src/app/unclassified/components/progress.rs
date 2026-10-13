use leptos::either::Either;
use leptos::prelude::*;
use uuid::Uuid;

use crate::app::constants::routes;
use crate::app::error_view::{ErrorStyle, ErrorView};
use crate::app::unclassified::tracker::{BatchPhase, BatchTracker};
use crate::app::upload_api::ConversionStatus;

#[component]
pub fn FileProgressBar(name: String, status: RwSignal<Option<ConversionStatus>>) -> impl IntoView {
    view! {
        {move || {
            let s = status.get();
            let (label, pct, bar_class, indeterminate, text_class) = match &s {
                None => ("Waiting…".to_string(), 0.0_f32,
                         "bg-gradient-to-r from-cyan-400 to-blue-500", true, "text-gray-300"),
                Some(ConversionStatus::Writing) => ("Starting…".to_string(), 0.0,
                         "bg-gradient-to-r from-cyan-400 to-blue-500", true, "text-gray-300"),
                Some(ConversionStatus::Moving { progress, .. }) => {
                    let p = if progress.is_finite() { *progress } else { 0.0 };
                    (format!("Moving {:.0}%", p * 100.0), p,
                     "bg-gradient-to-r from-cyan-400 to-blue-500", false, "text-gray-300")
                }
                Some(ConversionStatus::Converting { progress, .. }) => {
                    let p = if progress.is_finite() { *progress } else { 0.0 };
                    (format!("Converting {:.0}%", p * 100.0), p,
                     "bg-gradient-to-r from-cyan-400 to-blue-500", false, "text-gray-300")
                }
                Some(ConversionStatus::Finalizing) => ("Saving…".to_string(), 1.0,
                         "bg-gradient-to-r from-cyan-400 to-blue-500", true, "text-gray-300"),
                Some(ConversionStatus::Done) => ("Done".to_string(), 1.0,
                         "bg-gradient-to-r from-emerald-400 to-green-500", false, "text-emerald-300"),
                Some(ConversionStatus::Cancelled) => ("Cancelled".to_string(), 0.0,
                         "bg-gray-500/50", false, "text-gray-500"),
                Some(ConversionStatus::Failed(e)) => (format!("Failed: {}", e.message), 0.0,
                         "bg-red-500/60", false, "text-red-300"),
            };

            let name = name.clone();
            view! {
                <div class="space-y-1">
                    <div class="flex items-center justify-between gap-2 text-[11px]">
                        <span class=format!("truncate {text_class}")>{name}</span>
                        <span class="text-gray-500 shrink-0">{label}</span>
                    </div>
                    <div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
                        {if indeterminate {
                            Either::Left(view! {
                                <div class="h-full w-1/3 bg-gradient-to-r from-cyan-400 to-blue-500 \
                                                rounded-full animate-pulse"></div>
                            })
                        } else {
                            Either::Right(view! {
                                <div class=format!(
                                        "h-full {} rounded-full transition-all duration-300",
                                        bar_class,
                                    )
                                    style=format!("width: {:.0}%", pct * 100.0)
                                ></div>
                            })
                        }}
                    </div>
                </div>
            }
        }}
    }
}

#[component]
pub fn BatchProgress(
    tracker: BatchTracker,
    on_cancel: Callback<()>,
    cancel_pending: Signal<bool>,
) -> impl IntoView {
    let slots = tracker.slots();
    let total = move || slots.get().len();
    let done = move || {
        slots
            .get()
            .iter()
            .filter(|s| {
                matches!(
                    s.status.get(),
                    Some(ConversionStatus::Done)
                        | Some(ConversionStatus::Cancelled)
                        | Some(ConversionStatus::Failed(_))
                )
            })
            .count()
    };

    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between text-xs">
                <span class="text-gray-400">
                    {move || format!("{} / {} done", done(), total())}
                </span>
                <button type="button"
                    on:click=move |_| on_cancel.run(())
                    disabled=move || cancel_pending.get()
                    class="text-xs px-2 py-1 rounded-md text-gray-300 \
                               hover:bg-white/10 hover:text-white transition \
                               disabled:opacity-30 disabled:cursor-not-allowed">
                    {move || if cancel_pending.get() { "Cancelling…" } else { "Cancel" }}
                </button>
            </div>
            <div class="space-y-1.5 max-h-72 overflow-auto pr-1">
                {move || slots.get().into_iter().map(|slot| view! {
                    <FileProgressBar name=slot.name status=slot.status/>
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
pub fn FailedFilesList(tracker: BatchTracker) -> impl IntoView {
    let slots = tracker.slots();

    view! {
        {move || {
            let failures: Vec<(String, String, bool)> = slots
                .get()
                .into_iter()
                .filter_map(|s| match s.status.get() {
                    Some(ConversionStatus::Failed(e)) => {
                        Some((s.name.clone(), e.message, true))
                    }
                    Some(ConversionStatus::Cancelled) => {
                        Some((s.name.clone(), "cancelled".to_string(), false))
                    }
                    _ => None,
                })
                .collect();

            if failures.is_empty() {
                return ().into_any();
            }

            view! {
                <div class="text-xs bg-white/[0.03] border border-white/10 \
                                rounded-lg p-2 max-h-32 overflow-auto space-y-0.5">
                    {failures.into_iter().map(|(name, msg, is_error)| {
                        let color = if is_error { "text-red-300/80" } else { "text-gray-400" };
                        view! {
                            <div class="truncate">
                                <span class="text-gray-300">{name}</span>
                                <span class=format!(" ml-1 {color}")>" — " {msg}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }
            .into_any()
        }}
    }
}

#[component]
pub fn PhaseFinal(
    phase: Signal<BatchPhase>,
    tracker: BatchTracker,
    success_heading: Signal<String>,
    success_detail: Signal<String>,
    link: Signal<Option<(String, Option<Uuid>)>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let retrying = tracker.retrying();
    let failed_count = move || tracker.failed_count();

    let on_retry = Callback::new(move |_: ()| {
        tracker.retry_failed();
    });

    view! {
        <div class="space-y-4">
            <Show when=move || matches!(phase.get(), BatchPhase::Done)>
                <div class="bg-green-500/15 border border-green-500/30 rounded-xl p-4 \
                                flex items-start gap-3">
                    <span class="text-green-400 text-xl leading-none shrink-0">"✓"</span>
                    <div class="min-w-0 flex-1">
                        <div class="text-green-300 font-bold">
                            {move || success_heading.get()}
                        </div>
                        <div class="text-green-300/70 text-xs mt-1">
                            {move || success_detail.get()}
                        </div>
                        {move || link.get().map(|(slug, collection_id)| {
                            let href = match collection_id {
                                Some(id) => routes::collection(&slug, id),
                                None => routes::section(&slug),
                            };
                            let label = if collection_id.is_some() {
                                "View collection →"
                            } else {
                                "View section →"
                            };
                            view! {
                                <a href=href
                                    class="inline-block mt-2 text-xs text-green-300 \
                                               hover:text-green-200 underline underline-offset-2">
                                    {label}
                                </a>
                            }
                        })}
                    </div>
                </div>
            </Show>

            <Show when=move || matches!(phase.get(), BatchPhase::Cancelled)>
                <div class="bg-gray-500/15 border border-gray-500/30 rounded-xl p-4 \
                                flex items-start gap-3">
                    <span class="text-gray-400 text-xl leading-none shrink-0">"■"</span>
                    <div class="min-w-0">
                        <div class="text-gray-300 font-bold">"Cancelled"</div>
                        <div class="text-gray-400 text-xs mt-1">"No files were moved."</div>
                    </div>
                </div>
            </Show>

            <Show when=move || matches!(phase.get(), BatchPhase::Partial { .. })>
                {move || {
                    let (succeeded, failed, cancelled, first_error_msg) = match phase.get() {
                        BatchPhase::Partial { succeeded, failed, cancelled, first_error } => {
                            (succeeded, failed, cancelled, first_error.map(|e| e.message))
                        }
                        _ => return ().into_any(),
                    };
                    let total = succeeded + failed + cancelled;
                    let mut parts: Vec<String> = Vec::new();
                    if succeeded > 0 { parts.push(format!("{succeeded} moved")); }
                    if failed > 0 { parts.push(format!("{failed} failed")); }
                    if cancelled > 0 { parts.push(format!("{cancelled} cancelled")); }
                    let summary = format!("{} of {total}: {}", succeeded, parts.join(", "));
                    view! {
                        <div class="bg-amber-500/15 border border-amber-500/30 rounded-xl p-4 \
                                        flex items-start gap-3">
                            <span class="text-amber-400 text-xl leading-none shrink-0">"!"</span>
                            <div class="min-w-0">
                                <div class="text-amber-300 font-bold">{summary}</div>
                                {first_error_msg.map(|msg| view! {
                                    <div class="text-amber-300/70 text-xs mt-1 break-all">
                                        {format!("First error: {msg}")}
                                    </div>
                                })}
                            </div>
                        </div>
                    }
                    .into_any()
                }}
            </Show>

            <Show when=move || matches!(phase.get(), BatchPhase::Failed(_))>
                <ErrorView
                    error={Signal::derive(move || match phase.get() {
                        BatchPhase::Failed(e) => Some(e),
                        _ => None,
                    })}
                    style=ErrorStyle::Banner
                    heading="Move failed"
                />
            </Show>

            <Show when=move || !matches!(phase.get(), BatchPhase::Done)>
                <FailedFilesList tracker/>
            </Show>

            <div class="flex justify-end gap-2">
                <Show when=move || { failed_count() > 0 }>
                    <button type="button"
                        on:click=move |_| on_retry.run(())
                        disabled=move || retrying.get()
                        class="px-4 py-2 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 \
                                   text-amber-200 font-bold text-sm transition \
                                   disabled:opacity-50">
                        {move || {
                            let n = failed_count();
                            if retrying.get() {
                                "Retrying…".to_string()
                            } else {
                                format!("Retry {n} failed")
                            }
                        }}
                    </button>
                </Show>
                <button type="button"
                    on:click=move |_| on_close.run(())
                    class="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 \
                               text-white text-sm transition">
                    "Close"
                </button>
            </div>
        </div>
    }
}
