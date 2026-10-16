//! Parallel classification batch tracker.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use uuid::Uuid;

use crate::app::error::AppError;
use crate::app::types::{JobId, RelPath, Title};
use crate::app::upload_api::ConversionStatus;

use super::types::{ClassifyDestination, ClassifyInput, FileJobHandle};

const POLL_INTERVAL_MS: u32 = 700;
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

#[derive(Clone)]
pub struct FileSlot {
    pub name: String,
    pub job_id: JobId,
    pub section_slug: String,
    pub collection_id: Option<Uuid>,
    pub rel_path: RelPath,
    pub title: Title,
    pub season_number: Option<i64>,
    pub status: RwSignal<Option<ConversionStatus>>,
}

impl FileSlot {
    fn from_handle(h: FileJobHandle) -> Self {
        Self {
            name: h.name,
            job_id: h.job_id,
            section_slug: h.section_slug,
            collection_id: h.collection_id,
            rel_path: h.rel_path,
            title: h.title,
            season_number: h.season_number,
            status: RwSignal::new(None),
        }
    }
}

#[derive(Clone)]
pub enum BatchPhase {
    Setup,
    Running,
    Done,
    Cancelled,
    Failed(AppError),
    Partial {
        succeeded: usize,
        failed: usize,
        cancelled: usize,
        first_error: Option<AppError>,
    },
}

#[derive(Clone, Copy)]
pub struct BatchTracker {
    slots: RwSignal<Vec<FileSlot>>,
    running: RwSignal<bool>,
    error: RwSignal<Option<AppError>>,
    retrying: RwSignal<bool>,
}

impl BatchTracker {
    pub fn new() -> Self {
        Self {
            slots: RwSignal::new(Vec::new()),
            running: RwSignal::new(false),
            error: RwSignal::new(None),
            retrying: RwSignal::new(false),
        }
    }

    pub fn begin(&self, handles: Vec<FileJobHandle>) {
        let slots: Vec<FileSlot> = handles.into_iter().map(FileSlot::from_handle).collect();
        self.error.set(None);
        self.retrying.set(false);
        self.slots.set(slots);
        self.running.set(true);
        leptos::task::spawn_local(poll_batch_loop(*self));
    }

    pub fn reset(&self) {
        self.running.set(false);
        self.retrying.set(false);
        self.slots.set(Vec::new());
        self.error.set(None);
    }

    pub fn phase(&self) -> Signal<BatchPhase> {
        let slots_sig = self.slots;
        let error = self.error;
        let retrying = self.retrying;
        Signal::derive(move || {
            if retrying.get() {
                return BatchPhase::Running;
            }
            if let Some(e) = error.get() {
                return BatchPhase::Failed(e);
            }
            let slots = slots_sig.get();
            if slots.is_empty() {
                return BatchPhase::Setup;
            }
            let statuses: Vec<Option<ConversionStatus>> =
                slots.iter().map(|s| s.status.get()).collect();
            if statuses.iter().any(|s| s.is_none()) {
                return BatchPhase::Running;
            }
            let mut succeeded = 0usize;
            let mut failed = 0usize;
            let mut cancelled = 0usize;
            let mut first_error: Option<AppError> = None;
            let mut all_terminal = true;
            for s in &statuses {
                match s {
                    Some(ConversionStatus::Done) => succeeded += 1,
                    Some(ConversionStatus::Cancelled) => cancelled += 1,
                    Some(ConversionStatus::Failed(e)) => {
                        failed += 1;
                        if first_error.is_none() {
                            first_error = Some(e.clone());
                        }
                    }
                    Some(_) => all_terminal = false,
                    None => all_terminal = false,
                }
            }
            if !all_terminal {
                return BatchPhase::Running;
            }
            match (succeeded, failed, cancelled) {
                (_, 0, 0) => BatchPhase::Done,
                (0, 0, _) => BatchPhase::Cancelled,
                (0, _, 0) => BatchPhase::Failed(
                    first_error.unwrap_or_else(|| AppError::detailed("Move failed")),
                ),
                _ => BatchPhase::Partial {
                    succeeded,
                    failed,
                    cancelled,
                    first_error,
                },
            }
        })
    }

    pub fn slots(&self) -> RwSignal<Vec<FileSlot>> {
        self.slots
    }

    pub fn retrying(&self) -> RwSignal<bool> {
        self.retrying
    }

    pub fn error_pub(&self) -> RwSignal<Option<AppError>> {
        self.error
    }

    pub fn pending_job_ids(&self) -> Vec<JobId> {
        self.slots
            .get_untracked()
            .iter()
            .filter(|s| {
                !matches!(
                    s.status.get_untracked(),
                    Some(ConversionStatus::Done)
                        | Some(ConversionStatus::Cancelled)
                        | Some(ConversionStatus::Failed(_))
                )
            })
            .map(|s| s.job_id)
            .collect()
    }

    pub fn failed_count(&self) -> usize {
        self.slots
            .get_untracked()
            .iter()
            .filter(|s| matches!(s.status.get_untracked(), Some(ConversionStatus::Failed(_))))
            .count()
    }

    pub fn retry_failed(&self) {
        if self.retrying.get_untracked() {
            return;
        }

        let slots = self.slots.get_untracked();
        let mut groups: std::collections::HashMap<(Uuid, Option<i64>), Vec<ClassifyInput>> =
            std::collections::HashMap::new();

        for s in &slots {
            if !matches!(s.status.get_untracked(), Some(ConversionStatus::Failed(_))) {
                continue;
            }
            let Some(cid) = s.collection_id else { continue };
            let key = (cid, s.season_number);
            groups.entry(key).or_default().push(ClassifyInput {
                rel_path: s.rel_path.clone(),
                title: s.title.clone(),
            });
        }

        if groups.is_empty() {
            return;
        }

        self.retrying.set(true);
        self.error.set(None);

        let tracker = *self;
        leptos::task::spawn_local(async move {
            use crate::app::unclassified::classify_unclassified_files;

            let mut new_handles: Vec<FileJobHandle> = Vec::new();
            let mut failed_to_start: Option<AppError> = None;

            let mut keys: Vec<_> = groups.keys().cloned().collect();
            keys.sort_by_key(|(_, season)| season.unwrap_or(-1));

            for key in keys {
                let (cid, season) = key;
                let files = groups.remove(&key).unwrap();
                match classify_unclassified_files(files, ClassifyDestination::Existing(cid), season)
                    .await
                {
                    Ok(mut h) => new_handles.append(&mut h),
                    Err(e) => {
                        failed_to_start = Some(e);
                        break;
                    }
                }
            }

            tracker.slots.update(|slots| {
                let mut fresh: std::collections::VecDeque<FileJobHandle> =
                    new_handles.into_iter().collect();
                for slot in slots.iter_mut() {
                    let is_failed = matches!(
                        slot.status.get_untracked(),
                        Some(ConversionStatus::Failed(_))
                    );
                    if !is_failed {
                        continue;
                    }
                    if let Some(next) = fresh.pop_front() {
                        *slot = FileSlot::from_handle(next);
                    }
                }
            });
            tracker.retrying.set(false);

            match failed_to_start {
                Some(e) => tracker.error.set(Some(e)),
                None => {
                    tracker.running.set(true);
                    leptos::task::spawn_local(poll_batch_loop(tracker));
                }
            }
        });
    }
}

async fn poll_batch_loop(tracker: BatchTracker) {
    use crate::app::unclassified::poll_batch;

    let mut consecutive_errors = 0u32;

    loop {
        if !tracker.running.get_untracked() {
            return;
        }
        let job_ids: Vec<JobId> = tracker
            .slots
            .get_untracked()
            .iter()
            .map(|s| s.job_id)
            .collect();
        if job_ids.is_empty() {
            return;
        }

        match poll_batch(job_ids.clone()).await {
            Ok(statuses) => {
                consecutive_errors = 0;
                let all_terminal = statuses.iter().all(|s| {
                    matches!(
                        s,
                        ConversionStatus::Done
                            | ConversionStatus::Cancelled
                            | ConversionStatus::Failed(_)
                    )
                });

                tracker.slots.update(|slots| {
                    for (slot, status) in slots.iter_mut().zip(statuses.iter()) {
                        slot.status.set(Some(status.clone()));
                    }
                });

                if all_terminal {
                    tracker.running.set(false);
                    return;
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    tracker.error.set(Some(e));
                    tracker.running.set(false);
                    return;
                }
            }
        }

        TimeoutFuture::new(POLL_INTERVAL_MS).await;
    }
}
