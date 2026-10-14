use crate::app::constants::{routes, storage};
use crate::app::error::AppError;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use uuid::Uuid;

const ALLOWED_POSTER_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];
const MAX_POSTER_BYTES: usize = 5 * 1024 * 1024;

/// The window (in seconds) from the start of a video from which the first
/// poster is picked. `0..=300` — the first five minutes. Contains title
/// cards and representative scenes for most feature films and episodes.
const FIRST_PICK_WINDOW_SECS: f64 = 300.0;

/// The regenerate path avoids the very start and the very end of the video
/// to reduce the odds of a black frame, studio logo, or end credits.
const RANDOM_MARGIN: f64 = 0.02;

// ─── Timestamp selection ──────────────────────────────────────────────────

/// Pick a timestamp for frame extraction.
///
/// `first_only=true` → uniform in `[0, min(duration, 300s)]`.
/// `first_only=false` → uniform in `[2%, 98%]` of duration.
pub fn pick_timestamp(duration_secs: f64, first_only: bool) -> f64 {
    if !duration_secs.is_finite() || duration_secs <= 0.0 {
        return 0.0;
    }
    let (lo, hi) = if first_only {
        (0.0, duration_secs.min(FIRST_PICK_WINDOW_SECS))
    } else {
        (
            duration_secs * RANDOM_MARGIN,
            duration_secs * (1.0 - RANDOM_MARGIN),
        )
    };
    if hi <= lo {
        return (lo + hi) * 0.5;
    }
    lo + pseudo_random_f64() * (hi - lo)
}

/// xorshift64 — good enough for "pick a different frame each click".
fn pseudo_random_f64() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static STATE: AtomicU64 = AtomicU64::new(0);

    let mut s = STATE.load(Ordering::Relaxed);
    if s == 0 {
        s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            | 1;
    }
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    STATE.store(s, Ordering::Relaxed);
    (s >> 11) as f64 / (1u64 << 53) as f64
}

// ─── Cache-busting filename ───────────────────────────────────────────────
//
// Every extraction produces a new filename. Two consequences:
//   1. The browser never serves a cached image after a regenerate.
//   2. The old file must be unlinked by the caller after the new one lands.

fn poster_filename(entity_id: Uuid) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{entity_id}-{ms}.jpg")
}

// ─── Frame extraction ─────────────────────────────────────────────────────

/// Extract a single frame at `timestamp_secs` and write it as
/// `<entity_id>-<millis>.jpg`. Returns the poster URL.
///
/// The frame is written to a hidden temp file first; only after ffmpeg
/// produces a non-empty result do we swap it in. A failure therefore never
/// destroys an existing poster.
pub async fn extract_and_save_poster(
    data_dir: &Path,
    subdir: &str,
    entity_id: Uuid,
    video_path: &Path,
    timestamp_secs: f64,
) -> Result<String, AppError> {
    let dir = data_dir.join(storage::POSTERS_DIR).join(subdir);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::internal("mkdir posters", e))?;

    let filename = poster_filename(entity_id);
    let final_path = dir.join(&filename);
    let tmp_path = dir.join(format!(".{filename}.extracting"));

    let ts_arg = format!("{timestamp_secs:.3}");
    let input = video_path.to_string_lossy().to_string();
    let output = tmp_path.to_string_lossy().to_string();

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-y",
            "-ss",
            &ts_arg,
            "-i",
            &input,
            "-frames:v",
            "1",
            "-q:v",
            "3",
            "-f",
            "image2",
            &output,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| AppError::internal("spawn ffmpeg for poster", e))?;

    if !status.success() {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(AppError::internal(
            "extract poster",
            format!("ffmpeg exited with {status}"),
        ));
    }

    let size = tokio::fs::metadata(&tmp_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    if size == 0 {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(AppError::internal("extract poster", "empty output"));
    }

    tokio::fs::rename(&tmp_path, &final_path)
        .await
        .map_err(|e| AppError::internal("rename extracted poster", e))?;

    Ok(routes::poster(subdir, &filename))
}

// ─── Manual poster write (audio only) ─────────────────────────────────────

fn resolve_poster_ext(client_ext: &str, bytes: &[u8]) -> Result<&'static str, AppError> {
    let candidate = match infer::get(bytes) {
        Some(kind) => kind.extension().to_string(),
        None => client_ext.to_ascii_lowercase(),
    };
    ALLOWED_POSTER_EXTS
        .iter()
        .find(|allowed| allowed.eq_ignore_ascii_case(&candidate))
        .copied()
        .ok_or_else(|| AppError::public(format!("Unsupported image format: .{candidate}")))
}

/// Write a manually-uploaded poster. Used only by the audio path — video
/// posters come from `extract_and_save_poster`.
pub async fn write_poster(
    data_dir: &Path,
    subdir: &str,
    entity_id: Uuid,
    client_ext: &str,
    bytes: &[u8],
) -> Result<String, AppError> {
    if bytes.len() > MAX_POSTER_BYTES {
        return Err(AppError::public(format!(
            "Image size exceeds {} MB",
            MAX_POSTER_BYTES / 1024 / 1024
        )));
    }

    let ext = resolve_poster_ext(client_ext, bytes)?;

    let dir = data_dir.join(storage::POSTERS_DIR).join(subdir);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::internal("mkdir posters", e))?;

    let filename = format!("{entity_id}.{ext}");
    tokio::fs::write(dir.join(&filename), bytes)
        .await
        .map_err(|e| AppError::internal("write poster", e))?;

    Ok(routes::poster(subdir, &filename))
}

// ─── Poster relocation on section change ──────────────────────────────────

/// Move a poster file from one subdirectory to another, returning the new
/// URL on success. Returns `None` for non-posters URLs or filesystem errors
/// (which are logged).
pub async fn move_poster_to_subdir(
    data_dir: &Path,
    from_subdir: &str,
    to_subdir: &str,
    poster_url: &str,
) -> Option<String> {
    let rest = poster_url.strip_prefix(routes::POSTERS_URL_PREFIX)?;
    let (_old, filename) = rest.split_once('/')?;

    let to_dir = data_dir.join(storage::POSTERS_DIR).join(to_subdir);
    if let Err(e) = tokio::fs::create_dir_all(&to_dir).await {
        leptos::logging::warn!("[poster] mkdir failed for {}: {e}", to_dir.display());
        return None;
    }

    let from_path = data_dir
        .join(storage::POSTERS_DIR)
        .join(from_subdir)
        .join(filename);
    let to_path = to_dir.join(filename);

    let moved = match tokio::fs::rename(&from_path, &to_path).await {
        Ok(()) => true,
        Err(rename_err) => {
            let copied = match tokio::fs::copy(&from_path, &to_path).await {
                Ok(_) => true,
                Err(copy_err) => {
                    leptos::logging::warn!(
                        "[poster] move failed: {} -> {} (rename: {rename_err}, copy: {copy_err})",
                        from_path.display(),
                        to_path.display(),
                    );
                    false
                }
            };
            if copied {
                let _ = tokio::fs::remove_file(&from_path).await;
            }
            copied
        }
    };
    if !moved {
        return None;
    }

    Some(format!(
        "{}{}/{}",
        routes::POSTERS_URL_PREFIX,
        to_subdir,
        filename
    ))
}
