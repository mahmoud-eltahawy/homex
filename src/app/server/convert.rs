use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;

// ─── Job state ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum JobPhase {
    Writing,
    Converting {
        conversion_index: usize,
        conversion_count: usize,
        current_file: String,
        progress: f32,
    },
    Finalizing,
    Done,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct Job {
    pub phase: JobPhase,
    pub started_at: Instant,
}

pub type Jobs = Arc<RwLock<HashMap<String, Job>>>;

pub fn new_jobs() -> Jobs {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn job_set_phase(jobs: &Jobs, job_id: Option<&str>, phase: JobPhase) {
    let Some(id) = job_id else { return };
    if let Some(job) = jobs.write().await.get_mut(id) {
        job.phase = phase;
    }
}

// ─── Target formats ───────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetFormat {
    Mp4,
    Mp3,
}

impl TargetFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mp3 => "mp3",
        }
    }
}

// ─── Container classification ─────────────────────────────────────────────

/// Browser can play this video container in `<video>` directly.
pub fn is_video_container_supported(ext: &str) -> bool {
    matches!(ext.to_ascii_lowercase().as_str(), "mp4" | "m4v" | "webm")
}

/// Browser can play this audio container in `<audio>` directly.
pub fn is_audio_container_supported(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp3" | "m4a" | "aac" | "wav" | "ogg" | "oga" | "opus"
    )
}

/// Video containers we know how to turn into MP4.
pub fn is_convertible_video(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mkv" | "mov" | "avi" | "wmv" | "flv" | "ts" | "mpg" | "mpeg" | "3gp" | "ogv"
    )
}

/// Audio containers we know how to turn into MP3.
pub fn is_convertible_audio(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "flac" | "wma" | "aiff" | "aif" | "alac" | "ape"
    )
}

// ─── ffprobe ──────────────────────────────────────────────────────────────

pub async fn ffprobe_duration(path: &Path) -> f64 {
    let out = match Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &path.to_string_lossy(),
        ])
        .output()
        .await
    {
        Ok(o) => o,
        Err(_) => return 0.0,
    };
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0)
}

// ─── Public dispatcher ────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn convert_file(
    input: &Path,
    output: &Path,
    target: TargetFormat,
    total_secs: f64,
    conversion_index: usize,
    conversion_count: usize,
    display_name: &str,
    jobs: &Jobs,
    job_id: Option<&str>,
) -> Result<(), String> {
    match target {
        TargetFormat::Mp4 => {
            ensure_mp4(
                input,
                output,
                total_secs,
                conversion_index,
                conversion_count,
                display_name,
                jobs,
                job_id,
            )
            .await
        }
        TargetFormat::Mp3 => {
            ensure_mp3(
                input,
                output,
                total_secs,
                conversion_index,
                conversion_count,
                display_name,
                jobs,
                job_id,
            )
            .await
        }
    }
}

// ─── Video → MP4 ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn ensure_mp4(
    input: &Path,
    output: &Path,
    total_secs: f64,
    conversion_index: usize,
    conversion_count: usize,
    display_name: &str,
    jobs: &Jobs,
    job_id: Option<&str>,
) -> Result<(), String> {
    // Attempt 1: remux video, transcode audio to AAC, drop subtitles.
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(["-nostats", "-progress", "pipe:1"])
        .args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-sn"])
        .arg(output);

    let ok = run_and_watch(
        cmd,
        total_secs,
        conversion_index,
        conversion_count,
        display_name,
        jobs,
        job_id,
    )
    .await?;
    if ok && file_ok(output).await {
        return Ok(());
    }

    let _ = tokio::fs::remove_file(output).await;

    // Attempt 2: full transcode.
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(["-nostats", "-progress", "pipe:1"])
        .args([
            "-c:v", "libx264", "-preset", "veryfast", "-crf", "23", "-c:a", "aac", "-b:a", "192k",
            "-sn",
        ])
        .arg(output);

    let ok = run_and_watch(
        cmd,
        total_secs,
        conversion_index,
        conversion_count,
        display_name,
        jobs,
        job_id,
    )
    .await?;
    if ok && file_ok(output).await {
        Ok(())
    } else {
        Err("فشل تحويل الفيديو (ffmpeg)".into())
    }
}

// ─── Audio → MP3 ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn ensure_mp3(
    input: &Path,
    output: &Path,
    total_secs: f64,
    conversion_index: usize,
    conversion_count: usize,
    display_name: &str,
    jobs: &Jobs,
    job_id: Option<&str>,
) -> Result<(), String> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(["-nostats", "-progress", "pipe:1"])
        .args(["-vn", "-c:a", "libmp3lame", "-q:a", "2"])
        .arg(output);

    let ok = run_and_watch(
        cmd,
        total_secs,
        conversion_index,
        conversion_count,
        display_name,
        jobs,
        job_id,
    )
    .await?;
    if ok && file_ok(output).await {
        Ok(())
    } else {
        Err("فشل تحويل الصوت (ffmpeg)".into())
    }
}

// ─── Shared runner ────────────────────────────────────────────────────────

async fn run_and_watch(
    mut cmd: Command,
    total_secs: f64,
    conversion_index: usize,
    conversion_count: usize,
    display_name: &str,
    jobs: &Jobs,
    job_id: Option<&str>,
) -> Result<bool, String> {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| format!("فشل تشغيل ffmpeg: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "لا يمكن قراءة مخرجات ffmpeg".to_string())?;

    let mut lines = BufReader::new(stdout).lines();
    let mut last_bucket: u32 = u32::MAX;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(val) = line.strip_prefix("out_time=")
            && total_secs > 0.0
            && let Some(secs) = parse_hms(val)
        {
            let pct = (secs / total_secs).clamp(0.0, 1.0) as f32;
            let bucket = (pct * 100.0) as u32;
            if bucket != last_bucket {
                last_bucket = bucket;
                job_set_phase(
                    jobs,
                    job_id,
                    JobPhase::Converting {
                        conversion_index,
                        conversion_count,
                        current_file: display_name.to_string(),
                        progress: pct,
                    },
                )
                .await;
            }
        }
    }

    Ok(child.wait().await.map(|s| s.success()).unwrap_or(false))
}

fn parse_hms(s: &str) -> Option<f64> {
    let mut p = s.split(':');
    let h: f64 = p.next()?.parse().ok()?;
    let m: f64 = p.next()?.parse().ok()?;
    let sec: f64 = p.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + sec)
}

async fn file_ok(p: &Path) -> bool {
    tokio::fs::metadata(p)
        .await
        .map(|m| m.len() > 0)
        .unwrap_or(false)
}
