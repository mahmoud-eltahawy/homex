use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::app::error::AppError;
use crate::app::model::MediaKind;
use crate::app::server::AppState;
use crate::app::server::convert::{
    JobPhase, Jobs, TargetFormat, convert_file, ffprobe_duration, is_audio_container_supported,
    is_video_container_supported, job_set_phase,
};
use crate::app::types::{FileName, JobId, RelPath};

use super::naming::slugify;
use super::types::{StagedFile, UploadFile, UploadPayload, UploadTarget};

const COPY_CHUNK: usize = 4 * 1024 * 1024;

fn target_for(kind: MediaKind, filename: &FileName) -> Option<TargetFormat> {
    let ext = filename.extension();
    match kind {
        MediaKind::Video if ext.as_ref().is_some_and(is_video_container_supported) => None,
        MediaKind::Video => Some(TargetFormat::Mp4),
        MediaKind::Audio if ext.as_ref().is_some_and(is_audio_container_supported) => None,
        MediaKind::Audio => Some(TargetFormat::Mp3),
    }
}

fn targets_for(payload: &UploadPayload, kind: MediaKind) -> Vec<Option<TargetFormat>> {
    payload
        .files
        .iter()
        .map(|f| target_for(kind, &f.filename))
        .collect()
}

struct StagingPlan {
    subdir: String,
    slug: String,
}

impl StagingPlan {
    fn base_dir(&self, media_root: &Path) -> PathBuf {
        media_root.join(&self.subdir).join(&self.slug)
    }

    fn rel_for(&self, stem: &str, token: Uuid, ext: &str) -> RelPath {
        let s = format!("{}/{}/{}-{}.{}", self.subdir, self.slug, stem, token, ext);
        RelPath::from_stored(s)
    }
}

fn storage_slug(payload: &UploadPayload) -> String {
    match &payload.target {
        UploadTarget::New { title, .. } => {
            let raw = slugify(title.as_str());
            if raw.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                raw
            }
        }
        UploadTarget::Existing { collection_id, .. } => format!("collection-{collection_id}"),
    }
}

fn build_staging_plan(payload: &UploadPayload) -> StagingPlan {
    StagingPlan {
        subdir: payload.section_slug.to_string(),
        slug: storage_slug(payload),
    }
}

async fn ensure_plan_dir(plan: &StagingPlan, media_root: &Path) -> Result<(), AppError> {
    let dir = plan.base_dir(media_root);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::internal("mkdir media staging dir", e))
}

async fn move_temp_to_final(
    src: &Path,
    dst: &Path,
    total_size: u64,
    display_name: &str,
    jobs: &Jobs,
    job_id: Option<&JobId>,
) -> Result<(), AppError> {
    if tokio::fs::rename(src, dst).await.is_ok() {
        return Ok(());
    }

    let mut reader = tokio::fs::File::open(src)
        .await
        .map_err(|e| AppError::internal(format!("open {}", src.display()), e))?;
    let mut writer = tokio::fs::File::create(dst)
        .await
        .map_err(|e| AppError::internal(format!("create {}", dst.display()), e))?;

    let mut buf = vec![0u8; COPY_CHUNK];
    let mut copied: u64 = 0;
    let mut last_bucket: u32 = u32::MAX;

    loop {
        let n = reader
            .read(&mut buf)
            .await
            .map_err(|e| AppError::internal("read during move", e))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buf[..n])
            .await
            .map_err(|e| AppError::internal("write during move", e))?;
        copied += n as u64;

        if total_size > 0 {
            let pct = (copied as f64 / total_size as f64).clamp(0.0, 1.0) as f32;
            let bucket = (pct * 100.0) as u32;
            if bucket != last_bucket {
                last_bucket = bucket;
                job_set_phase(
                    jobs,
                    job_id,
                    JobPhase::Moving {
                        current_file: display_name.to_string(),
                        progress: pct,
                    },
                )
                .await;
            }
        }
    }

    writer
        .flush()
        .await
        .map_err(|e| AppError::internal("flush during move", e))?;

    let _ = tokio::fs::remove_file(src).await;
    Ok(())
}

struct PlacedRaw {
    token: Uuid,
    safe_stem: String,
    rel: RelPath,
    abs: PathBuf,
}

async fn place_raw_file(
    ctx: &StagingContext<'_>,
    file: &UploadFile,
) -> Result<PlacedRaw, AppError> {
    let ext = file.filename.extension();
    let ext_str = ext.as_ref().map(|e| e.as_str()).unwrap_or("");
    let stem = file.filename.stem().to_string();
    let token = Uuid::new_v4();
    let rel = ctx.plan.rel_for(&stem, token, ext_str);
    let abs = rel.join_under(ctx.media_root);

    move_temp_to_final(
        &file.temp_path,
        &abs,
        file.size,
        file.filename.as_str(),
        &ctx.state.jobs,
        ctx.job_id,
    )
    .await?;

    Ok(PlacedRaw {
        token,
        safe_stem: stem,
        rel,
        abs,
    })
}

fn final_path_for(
    plan: &StagingPlan,
    media_root: &Path,
    placed: &PlacedRaw,
    target: TargetFormat,
) -> (RelPath, PathBuf) {
    let rel = plan.rel_for(&placed.safe_stem, placed.token, target.extension());
    let abs = rel.join_under(media_root);
    (rel, abs)
}

async fn duration_secs_rounded(path: &Path) -> i64 {
    ffprobe_duration(path).await.round() as i64
}

async fn stage_kept_file(placed: &PlacedRaw, file: &UploadFile) -> Result<StagedFile, AppError> {
    Ok(StagedFile {
        rel: placed.rel.clone(),
        duration: duration_secs_rounded(&placed.abs).await,
        size: file.size,
        title: file.title.clone(),
        origin: file.temp_path.clone(),
    })
}

async fn stage_converted_file(
    ctx: &StagingContext<'_>,
    placed: &PlacedRaw,
    file: &UploadFile,
    target: TargetFormat,
) -> Result<StagedFile, AppError> {
    let (out_rel, out_abs) = final_path_for(ctx.plan, ctx.media_root, placed, target);
    let total_secs = ffprobe_duration(&placed.abs).await;

    convert_file(
        &placed.abs,
        &out_abs,
        target,
        total_secs,
        ctx.conversion_pos,
        ctx.conversion_count,
        file.filename.as_str(),
        &ctx.state.jobs,
        ctx.job_id,
        ctx.cancel,
    )
    .await
    .map_err(|e| AppError::internal("convert file", e))?;

    let _ = tokio::fs::remove_file(&placed.abs).await;
    let size = tokio::fs::metadata(&out_abs)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(StagedFile {
        rel: out_rel,
        duration: total_secs.round() as i64,
        size,
        title: file.title.clone(),
        origin: file.temp_path.clone(),
    })
}

#[derive(Default)]
struct WrittenTracker {
    paths: Vec<PathBuf>,
}

impl WrittenTracker {
    fn add(&mut self, p: PathBuf) {
        self.paths.push(p);
    }

    fn replace(&mut self, old: PathBuf, new: PathBuf) {
        if let Some(slot) = self.paths.iter_mut().find(|p| **p == old) {
            *slot = new;
        } else {
            self.paths.push(new);
        }
    }

    async fn rollback(&self) {
        for p in &self.paths {
            let _ = tokio::fs::remove_file(p).await;
        }
    }
}

#[derive(Clone, Copy)]
struct StagingContext<'a> {
    state: &'a AppState,
    payload: &'a UploadPayload,
    plan: &'a StagingPlan,
    media_root: &'a Path,
    job_id: Option<&'a JobId>,
    cancel: &'a CancellationToken,
    conversion_pos: usize,
    conversion_count: usize,
}

impl<'a> StagingContext<'a> {
    fn files(&self) -> &'a [UploadFile] {
        &self.payload.files
    }
}

async fn stage_one_file(
    ctx: &StagingContext<'_>,
    file: &UploadFile,
    target: Option<TargetFormat>,
    track: &mut WrittenTracker,
) -> Result<StagedFile, AppError> {
    let placed = place_raw_file(ctx, file).await?;
    track.add(placed.abs.clone());

    match target {
        None => stage_kept_file(&placed, file).await,
        Some(t) => {
            let staged = stage_converted_file(ctx, &placed, file, t).await?;
            let out_abs = staged.rel.join_under(ctx.media_root);
            track.replace(placed.abs, out_abs);
            Ok(staged)
        }
    }
}

async fn stage_all(
    base_ctx: &StagingContext<'_>,
    targets: &[Option<TargetFormat>],
) -> Result<Vec<StagedFile>, AppError> {
    let files = base_ctx.files();
    let mut track = WrittenTracker::default();
    let mut staged = Vec::with_capacity(targets.len());
    let mut pos = 0usize;

    for (i, file) in files.iter().enumerate() {
        let ctx = StagingContext {
            conversion_pos: pos,
            ..*base_ctx
        };
        match stage_one_file(&ctx, file, targets[i], &mut track).await {
            Ok(sf) => {
                staged.push(sf);
                if targets[i].is_some() {
                    pos += 1;
                }
            }
            Err(e) => {
                track.rollback().await;
                return Err(e);
            }
        }
    }
    Ok(staged)
}

pub async fn stage_files(
    payload: &UploadPayload,
    state: &AppState,
    kind: MediaKind,
    job_id: Option<&JobId>,
    cancel: &CancellationToken,
) -> Result<Vec<StagedFile>, AppError> {
    let plan = build_staging_plan(payload);
    ensure_plan_dir(&plan, &state.config.storage.media_root).await?;

    let targets = targets_for(payload, kind);
    let conversion_count = targets.iter().filter(|t| t.is_some()).count();

    let ctx = StagingContext {
        state,
        payload,
        plan: &plan,
        media_root: &state.config.storage.media_root,
        job_id,
        cancel,
        conversion_pos: 0,
        conversion_count,
    };

    stage_all(&ctx, &targets).await
}
