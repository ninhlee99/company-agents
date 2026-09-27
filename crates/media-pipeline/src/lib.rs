#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaJob {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub format: MediaFormat,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub max_duration_seconds: u32,
    pub normalize_audio: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaFormat {
    Mp4H264,
    WebMvp9,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaProbe {
    pub duration_millis: u64,
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub codec: String,
    pub audio_present: bool,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaQaPolicy {
    pub max_file_size_bytes: u64,
    pub min_duration_millis: u64,
    pub max_duration_millis: u64,
    pub allowed_codecs: Vec<String>,
    pub require_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaQaResult {
    pub passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaError {
    InvalidJob(String),
    QaFailed(Vec<String>),
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJob(v) => write!(f, "invalid media job: {v}"),
            Self::QaFailed(v) => write!(f, "media QA failed: {}", v.join("; ")),
        }
    }
}

impl std::error::Error for MediaError {}

pub trait MediaExecutor {
    fn execute(&self, job: &MediaJob) -> Result<(), MediaError>;
}

pub struct FfmpegExecutor {
    executable: PathBuf,
    workspace: PathBuf,
    timeout: Duration,
}

impl FfmpegExecutor {
    pub fn new(
        executable: impl Into<PathBuf>,
        workspace: impl Into<PathBuf>,
        timeout: Duration,
    ) -> Result<Self, MediaError> {
        if timeout < Duration::from_secs(1) || timeout > Duration::from_secs(86_400) {
            return Err(MediaError::InvalidJob("FFmpeg timeout outside safe bounds".into()));
        }
        Ok(Self {
            executable: executable.into(),
            workspace: workspace.into(),
            timeout,
        })
    }

    pub fn from_env() -> Result<Self, MediaError> {
        let executable = std::env::var("FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".into());
        let workspace = std::env::var("MEDIA_WORKSPACE")
            .map(PathBuf::from)
            .map_err(|_| MediaError::InvalidJob("MEDIA_WORKSPACE is required for FFmpeg execution".into()))?;
        let timeout_seconds = std::env::var("FFMPEG_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(900);
        Self::new(executable, workspace, Duration::from_secs(timeout_seconds))
    }

    fn safe_path(&self, relative: &str) -> Result<PathBuf, MediaError> {
        validate_safe_path(relative)?;
        Ok(self.workspace.join(relative))
    }
}

impl MediaExecutor for FfmpegExecutor {
    fn execute(&self, job: &MediaJob) -> Result<(), MediaError> {
        let args = ffmpeg_args(job)?;
        let input = self.safe_path(&job.input_path)?;
        let output = self.safe_path(&job.output_path)?;

        if !input.exists() {
            return Err(MediaError::InvalidJob("input media does not exist".into()));
        }
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MediaError::InvalidJob(format!("cannot create output directory: {e}")))?;
        }

        let mut child = Command::new(&self.executable)
            .current_dir(&self.workspace)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| MediaError::InvalidJob(format!("failed to start FFmpeg: {e}")))?;

        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        return Ok(());
                    }
                    return Err(MediaError::InvalidJob(format!(
                        "FFmpeg exited with status {status}"
                    )));
                }
                Ok(None) => {
                    if started.elapsed() >= self.timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(MediaError::InvalidJob(
                            "FFmpeg execution timed out and was terminated".into(),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(MediaError::InvalidJob(format!(
                        "FFmpeg process polling failed: {error}"
                    )));
                }
            }
        }
    }
}

pub fn validate_job(job: &MediaJob) -> Result<(), MediaError> {
    if job.id.trim().is_empty() {
        return Err(MediaError::InvalidJob("job id is required".into()));
    }
    validate_safe_path(&job.input_path)?;
    validate_safe_path(&job.output_path)?;
    if job.input_path == job.output_path {
        return Err(MediaError::InvalidJob(
            "input and output paths must differ".into(),
        ));
    }
    if !(16..=8_192).contains(&job.width) || !(16..=8_192).contains(&job.height) {
        return Err(MediaError::InvalidJob(
            "dimensions outside safe bounds".into(),
        ));
    }
    if !(1..=120).contains(&job.fps) {
        return Err(MediaError::InvalidJob("fps outside safe bounds".into()));
    }
    if !(1..=86_400).contains(&job.max_duration_seconds) {
        return Err(MediaError::InvalidJob(
            "duration outside safe bounds".into(),
        ));
    }
    Ok(())
}

pub fn ffmpeg_args(job: &MediaJob) -> Result<Vec<String>, MediaError> {
    validate_job(job)?;
    let mut args = vec![
        "-hide_banner".into(),
        "-nostdin".into(),
        "-y".into(),
        "-i".into(),
        job.input_path.clone(),
        "-vf".into(),
        format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,fps={}",
            job.width, job.height, job.fps
        ),
        "-t".into(),
        job.max_duration_seconds.to_string(),
    ];

    match job.format {
        MediaFormat::Mp4H264 => {
            args.extend([
                "-c:v".into(),
                "libx264".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-movflags".into(),
                "+faststart".into(),
            ]);
        }
        MediaFormat::WebMvp9 => {
            args.extend([
                "-c:v".into(),
                "libvpx-vp9".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
            ]);
        }
    }

    if job.normalize_audio {
        args.extend([
            "-af".into(),
            "loudnorm=I=-14:TP=-1.5:LRA=11".into(),
            "-c:a".into(),
            "aac".into(),
        ]);
    }
    args.push(job.output_path.clone());
    Ok(args)
}

pub fn qa(probe: &MediaProbe, policy: &MediaQaPolicy) -> MediaQaResult {
    let mut failures = Vec::new();
    if probe.file_size_bytes > policy.max_file_size_bytes {
        failures.push(format!(
            "file size {} exceeds {} bytes",
            probe.file_size_bytes, policy.max_file_size_bytes
        ));
    }
    if probe.duration_millis < policy.min_duration_millis {
        failures.push("duration below minimum".into());
    }
    if probe.duration_millis > policy.max_duration_millis {
        failures.push("duration above maximum".into());
    }
    if !policy
        .allowed_codecs
        .iter()
        .any(|codec| codec.eq_ignore_ascii_case(&probe.codec))
    {
        failures.push(format!("codec {} is not allowed", probe.codec));
    }
    if policy.require_audio && !probe.audio_present {
        failures.push("audio track required".into());
    }
    if probe.width == 0 || probe.height == 0 || probe.fps_num == 0 || probe.fps_den == 0 {
        failures.push("invalid media dimensions or frame rate".into());
    }
    MediaQaResult {
        passed: failures.is_empty(),
        failures,
    }
}

fn validate_safe_path(path: &str) -> Result<(), MediaError> {
    if path.trim().is_empty()
        || Path::new(path).is_absolute()
        || path.starts_with('/')
        || path.contains("..")
        || path.contains('\\')
        || path.contains('\0')
    {
        return Err(MediaError::InvalidJob(
            "media paths must be relative and traversal-free".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job() -> MediaJob {
        MediaJob {
            id: "job-1".into(),
            input_path: "input/source.mp4".into(),
            output_path: "output/final.mp4".into(),
            format: MediaFormat::Mp4H264,
            width: 1080,
            height: 1920,
            fps: 30,
            max_duration_seconds: 60,
            normalize_audio: true,
        }
    }

    #[test]
    fn ffmpeg_args_never_use_a_shell() {
        let args = ffmpeg_args(&job()).unwrap();
        assert!(args.contains(&"-nostdin".into()));
        assert!(args.contains(&"libx264".into()));
        assert!(!args.iter().any(|v| v.contains("&&") || v.contains(";")));
    }

    #[test]
    fn traversal_is_rejected() {
        let mut value = job();
        value.input_path = "../secret.mp4".into();
        assert!(validate_job(&value).is_err());
    }

    #[test]
    fn qa_rejects_wrong_codec_and_missing_audio() {
        let probe = MediaProbe {
            duration_millis: 30_000,
            width: 1080,
            height: 1920,
            fps_num: 30,
            fps_den: 1,
            codec: "mpeg4".into(),
            audio_present: false,
            file_size_bytes: 10_000_000,
        };
        let result = qa(
            &probe,
            &MediaQaPolicy {
                max_file_size_bytes: 20_000_000,
                min_duration_millis: 1_000,
                max_duration_millis: 60_000,
                allowed_codecs: vec!["h264".into()],
                require_audio: true,
            },
        );
        assert!(!result.passed);
        assert_eq!(result.failures.len(), 2);
    }

    #[test]
    fn malicious_filename_cannot_change_process_arguments() {
        let mut value = job();
        value.input_path = "uploads/video;rm -rf /".into();
        let result = ffmpeg_args(&value).unwrap();
        assert!(result.iter().any(|arg| arg == "uploads/video;rm -rf /"));
        assert_eq!(result[0], "-hide_banner");
    }
}
