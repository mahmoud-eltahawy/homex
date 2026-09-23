use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

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

pub fn is_video_container_supported(ext: &str) -> bool {
    matches!(ext.to_ascii_lowercase().as_str(), "mp4" | "m4v" | "webm")
}

pub fn is_audio_container_supported(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp3" | "m4a" | "aac" | "wav" | "ogg" | "oga" | "opus"
    )
}

pub fn is_convertible_video(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mkv" | "mov" | "avi" | "wmv" | "flv" | "ts" | "mpg" | "mpeg" | "3gp" | "ogv"
    )
}

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

// ─── Command builders (pure) ──────────────────────────────────────────────

fn base_ffmpeg_cmd() -> Command {
    let mut c = Command::new("ffmpeg");
    c.arg("-hide_banner")
        .arg("-y")
        .arg("-nostats")
        .args(["-progress", "pipe:1"]);
    c
}

fn ffmpeg_remux_mp4_cmd(input: &Path, output: &Path) -> Command {
    let mut c = base_ffmpeg_cmd();
    c.arg("-i")
        .arg(input)
        .args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-sn"])
        .arg(output);
    c
}

fn ffmpeg_transcode_mp4_cmd(input: &Path, output: &Path) -> Command {
    let mut c = base_ffmpeg_cmd();
    c.arg("-i")
        .arg(input)
        .args([
            "-c:v", "libx264", "-preset", "veryfast", "-crf", "23", "-c:a", "aac", "-b:a", "192k",
            "-sn",
        ])
        .arg(output);
    c
}

fn ffmpeg_mp3_cmd(input: &Path, output: &Path) -> Command {
    let mut c = base_ffmpeg_cmd();
    c.arg("-i")
        .arg(input)
        .args(["-vn", "-c:a", "libmp3lame", "-q:a", "2"])
        .arg(output);
    c
}

// ─── Process control ──────────────────────────────────────────────────────

fn configure_pipes(cmd: &mut Command) {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null());
}

async fn spawn_ffmpeg(mut cmd: Command) -> Result<Child, String> {
    configure_pipes(&mut cmd);
    cmd.spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {e}"))
}

fn take_stdout(child: &mut Child) -> Result<ChildStdout, String> {
    child
        .stdout
        .take()
        .ok_or_else(|| "Cannot read ffmpeg output".to_string())
}

async fn wait_ok(child: &mut Child) -> bool {
    child.wait().await.map(|s| s.success()).unwrap_or(false)
}

async fn kill_silently(child: &mut Child) {
    let _ = child.kill().await;
}

// ─── Progress line parsing ────────────────────────────────────────────────

fn parse_progress_line(line: &str, total_secs: f64) -> Option<f32> {
    if total_secs <= 0.0 {
        return None;
    }
    let val = line.strip_prefix("out_time=")?;
    let secs = parse_hms(val)?;
    Some((secs / total_secs).clamp(0.0, 1.0) as f32)
}

fn parse_hms(s: &str) -> Option<f64> {
    let mut p = s.split(':');
    let h: f64 = p.next()?.parse().ok()?;
    let m: f64 = p.next()?.parse().ok()?;
    let sec: f64 = p.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + sec)
}

/// Whole-percent throttle.
struct ProgressTracker {
    last_bucket: u32,
}

impl ProgressTracker {
    fn new() -> Self {
        Self {
            last_bucket: u32::MAX,
        }
    }

    fn observe(&mut self, pct: f32) -> Option<f32> {
        let bucket = (pct * 100.0) as u32;
        if bucket == self.last_bucket {
            return None;
        }
        self.last_bucket = bucket;
        Some(pct)
    }
}

// ─── Context for one conversion ───────────────────────────────────────────

#[derive(Clone, Copy)]
struct ConvertContext<'a> {
    jobs: &'a Jobs,
    job_id: Option<&'a str>,
    conversion_index: usize,
    conversion_count: usize,
    display_name: &'a str,
    total_secs: f64,
}

impl<'a> ConvertContext<'a> {
    fn build(
        jobs: &'a Jobs,
        job_id: Option<&'a str>,
        conversion_index: usize,
        conversion_count: usize,
        display_name: &'a str,
        total_secs: f64,
    ) -> Self {
        Self {
            jobs,
            job_id,
            conversion_index,
            conversion_count,
            display_name,
            total_secs,
        }
    }
}

async fn publish_progress(ctx: &ConvertContext<'_>, pct: f32) {
    job_set_phase(
        ctx.jobs,
        ctx.job_id,
        JobPhase::Converting {
            conversion_index: ctx.conversion_index,
            conversion_count: ctx.conversion_count,
            current_file: ctx.display_name.to_string(),
            progress: pct,
        },
    )
    .await;
}

// ─── Progress streaming ───────────────────────────────────────────────────

async fn stream_ffmpeg_progress(stdout: ChildStdout, ctx: ConvertContext<'_>) {
    let mut lines = BufReader::new(stdout).lines();
    let mut tracker = ProgressTracker::new();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(pct) = parse_progress_line(&line, ctx.total_secs)
            && tracker.observe(pct).is_some()
        {
            publish_progress(&ctx, pct).await;
        }
    }
}

// ─── One ffmpeg attempt ───────────────────────────────────────────────────

async fn run_and_watch(
    cmd: Command,
    ctx: ConvertContext<'_>,
    cancel: &CancellationToken,
) -> Result<bool, String> {
    let mut child = spawn_ffmpeg(cmd).await?;
    let stdout = take_stdout(&mut child)?;

    tokio::select! {
        _ = cancel.cancelled() => {
            kill_silently(&mut child).await;
            return Err("Conversion cancelled".into());
        }
        _ = stream_ffmpeg_progress(stdout, ctx) => {}
    }

    Ok(wait_ok(&mut child).await)
}

async fn attempt_conversion(
    cmd: Command,
    output: &Path,
    ctx: ConvertContext<'_>,
    cancel: &CancellationToken,
) -> Result<bool, String> {
    let ok = run_and_watch(cmd, ctx, cancel).await?;
    if ok && file_ok(output).await {
        return Ok(true);
    }
    let _ = tokio::fs::remove_file(output).await;
    Ok(false)
}

async fn file_ok(p: &Path) -> bool {
    tokio::fs::metadata(p)
        .await
        .map(|m| m.len() > 0)
        .unwrap_or(false)
}

// ─── Format-specific attempts ─────────────────────────────────────────────

async fn ensure_mp4(
    input: &Path,
    output: &Path,
    ctx: ConvertContext<'_>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if attempt_conversion(ffmpeg_remux_mp4_cmd(input, output), output, ctx, cancel).await? {
        return Ok(());
    }
    if attempt_conversion(ffmpeg_transcode_mp4_cmd(input, output), output, ctx, cancel).await? {
        return Ok(());
    }
    Err("Video conversion failed (ffmpeg)".into())
}

async fn ensure_mp3(
    input: &Path,
    output: &Path,
    ctx: ConvertContext<'_>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if attempt_conversion(ffmpeg_mp3_cmd(input, output), output, ctx, cancel).await? {
        return Ok(());
    }
    Err("Audio conversion failed (ffmpeg)".into())
}

// ─── Public entry point ───────────────────────────────────────────────────

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
    cancel: &CancellationToken,
) -> Result<(), String> {
    let ctx = ConvertContext::build(
        jobs,
        job_id,
        conversion_index,
        conversion_count,
        display_name,
        total_secs,
    );
    match target {
        TargetFormat::Mp4 => ensure_mp4(input, output, ctx, cancel).await,
        TargetFormat::Mp3 => ensure_mp3(input, output, ctx, cancel).await,
    }
}
