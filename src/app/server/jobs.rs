//! Job orchestration primitives.
//!
//! The `Job` / `Jobs` / `JobPhase` types live in `convert.rs`. This module
//! adds the surrounding machinery: a global concurrency bound, path locks,
//! the `PreparedJob` shape, and the spawn/wait helpers used by any endpoint
//! that fans a batch of file operations out into per-file jobs.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::app::error::AppError;
use crate::app::model::MediaKind;
use crate::app::server::AppState;
use crate::app::server::convert::{Job, JobPhase, job_set_phase};
use crate::app::server::upload::{
    UploadFile, UploadPayload, process_upload, schedule_job_eviction,
};
use crate::app::unclassified::FileJobHandle;

// ─── Constants ────────────────────────────────────────────────────────────

pub const JOB_TIMEOUT_SECS: u64 = 15 * 60;
pub const WATCHER_DEADLINE_SECS: u64 = 30 * 60;

// ─── Global concurrency bound ─────────────────────────────────────────────

/// Process-wide semaphore bounding concurrent file operations. Sized to
/// `clamp(cpus * 2, 2, 16)` so a folder with hundreds of files doesn't
/// spawn hundreds of ffmpeg processes at once.
pub fn upload_semaphore() -> &'static tokio::sync::Semaphore {
    use tokio::sync::Semaphore;
    static SEM: OnceLock<Semaphore> = OnceLock::new();
    SEM.get_or_init(|| {
        let permits = std::thread::available_parallelism()
            .map(|n| n.get() * 2)
            .unwrap_or(4)
            .clamp(2, 16);
        Semaphore::new(permits)
    })
}

// ─── Path locks ───────────────────────────────────────────────────────────

fn active_classifications() -> &'static std::sync::Mutex<std::collections::HashSet<PathBuf>> {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static SET: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    SET.get_or_init(|| Mutex::new(HashSet::new()))
}

/// RAII guard releasing every path acquired by `acquire_classify_locks`.
pub struct ClassifyLocks(Vec<PathBuf>);

impl Drop for ClassifyLocks {
    fn drop(&mut self) {
        let mut set = active_classifications()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        for p in &self.0 {
            set.remove(p);
        }
    }
}

/// Atomically acquire locks on every path. Fails if any is already locked.
/// The lock is held until the returned guard is dropped.
pub fn acquire_classify_locks(paths: Vec<PathBuf>) -> Result<ClassifyLocks, AppError> {
    if paths.is_empty() {
        return Ok(ClassifyLocks(Vec::new()));
    }
    let mut set = active_classifications()
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    for p in &paths {
        if set.contains(p) {
            return Err(AppError::public(
                "A classification is already running for one of these paths",
            ));
        }
    }
    for p in &paths {
        set.insert(p.clone());
    }
    drop(set);
    Ok(ClassifyLocks(paths))
}

// ─── Config validation ────────────────────────────────────────────────────

/// Canonicalize `media_root` once per process. The result is leaked so it
/// can be reused across requests without re-running syscalls.
pub fn canonical_media_root(media_root: &Path) -> Result<&'static Path, AppError> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Result<&'static PathBuf, String>> = OnceLock::new();
    let result = CACHE.get_or_init(|| {
        std::fs::canonicalize(media_root)
            .map(|p| Box::leak(Box::new(p)) as &'static PathBuf)
            .map_err(|e| format!("canonicalize media_root: {e}"))
    });
    match result {
        Ok(p) => Ok(p.as_path()),
        Err(e) => Err(AppError::internal("config", e.clone())),
    }
}

/// Refuse to operate if the unclassified root and the media root overlap.
/// A rename-onto-itself would silently delete files.
pub fn ensure_roots_disjoint(unclassified_canon: &Path, media_root: &Path) -> Result<(), AppError> {
    let media_canon = canonical_media_root(media_root)?;
    if unclassified_canon == media_canon
        || unclassified_canon.starts_with(media_canon)
        || media_canon.starts_with(unclassified_canon)
    {
        leptos::logging::error!(
            "[config] unclassified_dir ({}) overlaps media_root ({})",
            unclassified_canon.display(),
            media_canon.display(),
        );
        return Err(AppError::internal(
            "config",
            "unclassified_dir and media_root must not overlap",
        ));
    }
    Ok(())
}

// ─── Job preparation ──────────────────────────────────────────────────────

pub fn make_dummy_tempdir() -> Result<tempfile::TempDir, AppError> {
    tempfile::TempDir::new().map_err(|e| AppError::internal("create dummy temp dir", e))
}

/// A fully-prepared file job. The client-facing `FileJobHandle` is built
/// from this when the job is registered.
pub struct PreparedJob {
    pub name: String,
    pub job_id: String,
    pub rel_path: String,
    pub title: String,
    pub season_number: Option<i64>,
    pub payload: UploadPayload,
    pub kind: MediaKind,
    pub cancel: CancellationToken,
}

pub fn prepare_file_job(
    file: UploadFile,
    collection_id: Uuid,
    section_slug: &str,
    collection_title: &str,
    season: Option<i64>,
    kind: MediaKind,
    rel_path: String,
) -> Result<PreparedJob, AppError> {
    let name = file.filename.clone();
    let title = file.title.clone();
    Ok(PreparedJob {
        name,
        job_id: Uuid::new_v4().to_string(),
        rel_path,
        title,
        season_number: season,
        payload: UploadPayload {
            section_slug: section_slug.to_string(),
            collection_id,
            title: collection_title.to_string(),
            description: String::new(),
            season_number: season,
            files: vec![file],
            poster: None,
            temp_dir: make_dummy_tempdir()?,
        },
        kind,
        cancel: CancellationToken::new(),
    })
}

// ─── Registration + spawn ─────────────────────────────────────────────────

/// Register every prepared job in one lock acquisition, then spawn one
/// worker per job. Returns handles the client uses to poll.
pub async fn register_and_spawn(
    state: &AppState,
    prepared: Vec<PreparedJob>,
    section_slug: &str,
    collection_id: Option<Uuid>,
) -> Vec<FileJobHandle> {
    let handles: Vec<FileJobHandle> = prepared
        .iter()
        .map(|p| FileJobHandle {
            name: p.name.clone(),
            job_id: p.job_id.clone(),
            section_slug: section_slug.to_string(),
            collection_id,
            rel_path: p.rel_path.clone(),
            title: p.title.clone(),
            season_number: p.season_number,
        })
        .collect();

    {
        let mut jobs = state.jobs.write().await;
        for p in &prepared {
            jobs.insert(
                p.job_id.clone(),
                Job {
                    phase: JobPhase::Writing,
                    started_at: std::time::Instant::now(),
                    cancel: p.cancel.clone(),
                },
            );
        }
    }

    for p in prepared {
        spawn_job_worker(state.clone(), p);
    }

    handles
}

fn spawn_job_worker(state: AppState, p: PreparedJob) {
    tokio::spawn(async move {
        let _permit = upload_semaphore().acquire().await.ok();
        let cancel = p.cancel.clone();

        let outcome = tokio::time::timeout(
            Duration::from_secs(JOB_TIMEOUT_SECS),
            process_upload(p.payload, &state, p.kind, Some(&p.job_id), &cancel),
        )
        .await;

        match outcome {
            Ok(Ok(_)) => {}
            Ok(Err(_)) if cancel.is_cancelled() => {
                job_set_phase(&state.jobs, Some(&p.job_id), JobPhase::Cancelled).await;
                schedule_job_eviction(state.jobs.clone(), p.job_id);
            }
            Ok(Err(e)) => {
                job_set_phase(&state.jobs, Some(&p.job_id), JobPhase::Failed(e)).await;
                schedule_job_eviction(state.jobs.clone(), p.job_id);
            }
            Err(_) => {
                cancel.cancel();
                let e = AppError::public("Timed out").transient();
                job_set_phase(&state.jobs, Some(&p.job_id), JobPhase::Failed(e)).await;
                schedule_job_eviction(state.jobs.clone(), p.job_id);
            }
        }
    });
}

// ─── Watchers ─────────────────────────────────────────────────────────────

/// Wait (bounded) until every job id reaches a terminal phase.
pub async fn wait_for_terminal(state: &AppState, job_ids: &[String]) {
    let wait = async {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let jobs = state.jobs.read().await;
            let all_terminal = job_ids.iter().all(|id| {
                jobs.get(id)
                    .map(|j| {
                        matches!(
                            j.phase,
                            JobPhase::Done | JobPhase::Cancelled | JobPhase::Failed(_)
                        )
                    })
                    .unwrap_or(true)
            });
            drop(jobs);
            if all_terminal {
                return;
            }
        }
    };
    let _ = tokio::time::timeout(Duration::from_secs(WATCHER_DEADLINE_SECS), wait).await;
}

/// Hold a `ClassifyLocks` guard for the duration of a batch, releasing it
/// once every job has reached a terminal phase.
pub fn spawn_lock_holder(state: AppState, job_ids: Vec<String>, lock: ClassifyLocks) {
    tokio::spawn(async move {
        let _lock = lock;
        wait_for_terminal(&state, &job_ids).await;
    });
}

/// Remove a source folder once every job in its group has finished.
pub fn spawn_cleanup_watcher(state: AppState, job_ids: Vec<String>, dir: PathBuf) {
    tokio::spawn(async move {
        wait_for_terminal(&state, &job_ids).await;
        if let Err(e) = tokio::fs::remove_dir(&dir).await
            && e.kind() != std::io::ErrorKind::NotFound
        {
            leptos::logging::warn!("[classify] could not remove {}: {e}", dir.display());
        }
    });
}

// Re-export so callers don't need to know where `FolderDestination` lives.
pub use crate::app::unclassified::types::FolderDestination as _FolderDestination;
