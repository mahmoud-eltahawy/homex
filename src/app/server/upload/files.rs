use std::path::{Path, PathBuf};

use leptos::prelude::ServerFnError;

use crate::app::model::MediaKind;
use crate::app::server::AppState;
use crate::app::server::convert::{
    TargetFormat, convert_file, ffprobe_duration, is_audio_container_supported,
    is_video_container_supported,
};
use crate::app::server::upload::new_storage_token;

use super::naming::{sanitize_filename, slugify};
use super::types::{StagedFile, UploadFile, UploadPayload};

// ─── Filename parsing ─────────────────────────────────────────────────────

fn extension_of(name: &str) -> String {
    name.rsplit('.').next().unwrap_or("").to_string()
}

fn stem_of(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(s, _)| s.to_string())
        .unwrap_or_else(|| name.to_string())
}

fn safe_stem(name: &str) -> String {
    sanitize_filename(&stem_of(name))
}

// ─── Target selection ─────────────────────────────────────────────────────

fn target_for(kind: MediaKind, filename: &str) -> Option<TargetFormat> {
    let ext = extension_of(filename);
    match kind {
        MediaKind::Video if is_video_container_supported(&ext) => None,
        MediaKind::Video => Some(TargetFormat::Mp4),
        MediaKind::Audio if is_audio_container_supported(&ext) => None,
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

// ─── Staging plan ─────────────────────────────────────────────────────────

struct StagingPlan {
    subdir: String,
    slug: String,
}

impl StagingPlan {
    fn base_dir(&self, media_root: &Path) -> PathBuf {
        media_root.join(&self.subdir).join(&self.slug)
    }

    fn rel_for(&self, stem: &str, token: &str, ext: &str) -> String {
        format!("{}/{}/{}-{}.{}", self.subdir, self.slug, stem, token, ext)
    }
}

fn storage_slug(payload: &UploadPayload) -> String {
    let raw = slugify(&payload.title);
    if raw.is_empty() {
        new_storage_token()
    } else {
        raw
    }
}

fn build_staging_plan(payload: &UploadPayload) -> StagingPlan {
    StagingPlan {
        subdir: payload.section_slug.clone(),
        slug: storage_slug(payload),
    }
}

async fn ensure_plan_dir(plan: &StagingPlan, media_root: &Path) -> Result<(), ServerFnError> {
    let dir = plan.base_dir(media_root);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir: {e}")))
}

// ─── Moving a temp file into place ────────────────────────────────────────

async fn move_temp_to_final(temp_path: &Path, dest: &Path) -> Result<(), ServerFnError> {
    if tokio::fs::rename(temp_path, dest).await.is_ok() {
        return Ok(());
    }
    tokio::fs::copy(temp_path, dest)
        .await
        .map_err(|e| ServerFnError::new(format!("copy {}: {e}", dest.display())))?;
    let _ = tokio::fs::remove_file(temp_path).await;
    Ok(())
}

// ─── Placing a raw file ───────────────────────────────────────────────────

struct PlacedRaw {
    token: String,
    safe_stem: String,
    rel: String,
    abs: PathBuf,
}

async fn place_raw_file(
    plan: &StagingPlan,
    media_root: &Path,
    file: &UploadFile,
) -> Result<PlacedRaw, ServerFnError> {
    let ext = extension_of(&file.filename);
    let stem = safe_stem(&file.filename);
    let token = new_storage_token();
    let rel = plan.rel_for(&stem, &token, &ext);
    let abs = media_root.join(&rel);
    move_temp_to_final(&file.temp_path, &abs).await?;
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
) -> (String, PathBuf) {
    let rel = plan.rel_for(&placed.safe_stem, &placed.token, target.extension());
    let abs = media_root.join(&rel);
    (rel, abs)
}

// ─── Building the StagedFile ──────────────────────────────────────────────

async fn duration_secs_rounded(path: &Path) -> i64 {
    ffprobe_duration(path).await.round() as i64
}

async fn stage_kept_file(
    placed: &PlacedRaw,
    file: &UploadFile,
) -> Result<StagedFile, ServerFnError> {
    Ok(StagedFile {
        rel: placed.rel.clone(),
        duration: duration_secs_rounded(&placed.abs).await,
        size: file.size,
        title: file.title.clone(),
    })
}

async fn stage_converted_file(
    ctx: &StagingContext<'_>,
    placed: &PlacedRaw,
    file: &UploadFile,
    target: TargetFormat,
) -> Result<StagedFile, ServerFnError> {
    let (out_rel, out_abs) = final_path_for(ctx.plan, ctx.media_root, placed, target);
    let total_secs = ffprobe_duration(&placed.abs).await;

    convert_file(
        &placed.abs,
        &out_abs,
        target,
        total_secs,
        ctx.conversion_pos,
        ctx.conversion_count,
        &file.filename,
        &ctx.state.jobs,
        ctx.job_id,
        &ctx.state.cancel,
    )
    .await
    .map_err(ServerFnError::new)?;

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
    })
}

// ─── Rollback tracking ────────────────────────────────────────────────────

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

// ─── Context for one staging run ──────────────────────────────────────────

#[derive(Clone, Copy)]
struct StagingContext<'a> {
    state: &'a AppState,
    payload: &'a UploadPayload,
    plan: &'a StagingPlan,
    media_root: &'a Path,
    job_id: Option<&'a str>,
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
) -> Result<StagedFile, ServerFnError> {
    let placed = place_raw_file(ctx.plan, ctx.media_root, file).await?;
    track.add(placed.abs.clone());

    match target {
        None => stage_kept_file(&placed, file).await,
        Some(t) => {
            let staged = stage_converted_file(ctx, &placed, file, t).await?;
            let out_abs = ctx.media_root.join(&staged.rel);
            track.replace(placed.abs, out_abs);
            Ok(staged)
        }
    }
}

async fn stage_all(
    base_ctx: &StagingContext<'_>,
    targets: &[Option<TargetFormat>],
) -> Result<Vec<StagedFile>, ServerFnError> {
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

// ─── Public entry point ───────────────────────────────────────────────────

pub async fn stage_files(
    payload: &UploadPayload,
    state: &AppState,
    kind: MediaKind,
    job_id: Option<&str>,
) -> Result<Vec<StagedFile>, ServerFnError> {
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
        conversion_pos: 0,
        conversion_count,
    };

    stage_all(&ctx, &targets).await
}
