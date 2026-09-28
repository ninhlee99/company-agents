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
    probe_executable: PathBuf,
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
            return Err(MediaError::InvalidJob(
                "FFmpeg timeout outside safe bounds".into(),
            ));
        }
        Ok(Self {
            executable: executable.into(),
            probe_executable: PathBuf::from("ffprobe"),
            workspace: workspace.into(),
            timeout,
        })
    }

    pub fn from_env() -> Result<Self, MediaError> {
        let executable = std::env::var("FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".into());
        let probe_executable =
            std::env::var("FFPROBE_BIN").unwrap_or_else(|_| "ffprobe".into());
        let workspace = std::env::var("MEDIA_WORKSPACE")
            .map(PathBuf::from)
            .map_err(|_| {
                MediaError::InvalidJob("MEDIA_WORKSPACE is required for FFmpeg execution".into())
            })?;
        let timeout_seconds = std::env::var("FFMPEG_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(900);
        let mut executor = Self::new(executable, workspace, Duration::from_secs(timeout_seconds))?;
        executor.probe_executable = PathBuf::from(probe_executable);
        Ok(executor)
    }

    fn safe_path(&self, relative: &str) -> Result<PathBuf, MediaError> {
        validate_safe_path(relative)?;
        let workspace = self.workspace.canonicalize().map_err(|error| {
            MediaError::InvalidJob(format!("media workspace is unavailable: {error}"))
        })?;
        let candidate = self.workspace.join(relative);

        if candidate.exists() {
            let canonical = candidate.canonicalize().map_err(|error| {
                MediaError::InvalidJob(format!("media path cannot be canonicalized: {error}"))
            })?;
            if !canonical.starts_with(&workspace) {
                return Err(MediaError::InvalidJob(
                    "media path escapes the configured workspace".into(),
                ));
            }
            return Ok(canonical);
        }

        let parent = candidate
            .parent()
            .ok_or_else(|| MediaError::InvalidJob("media path has no parent directory".into()))?;
        let canonical_parent = parent.canonicalize().map_err(|error| {
            MediaError::InvalidJob(format!("media parent cannot be canonicalized: {error}"))
        })?;
        if !canonical_parent.starts_with(&workspace) {
            return Err(MediaError::InvalidJob(
                "media path parent escapes the configured workspace".into(),
            ));
        }
        Ok(canonical_parent.join(
            candidate
                .file_name()
                .ok_or_else(|| MediaError::InvalidJob("media path has no filename".into()))?,
        ))
    }
}

impl FfmpegExecutor {
    pub fn probe(&self, relative_path: &str) -> Result<MediaProbe, MediaError> {
        let path = self.safe_path(relative_path)?;
        if !path.exists() {
            return Err(MediaError::InvalidJob("output media does not exist".into()));
        }

        let mut child = Command::new(&self.probe_executable)
            .current_dir(&self.workspace)
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "format=duration",
                "-show_entries",
                "stream=codec_name,width,height,r_frame_rate",
                "-of",
                "json",
                relative_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| MediaError::InvalidJob(format!("failed to start ffprobe: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| MediaError::InvalidJob("ffprobe stdout is unavailable".into()))?;

        let max_probe_bytes = 1_048_576usize;
        let reader = std::thread::spawn(move || {
            use std::io::Read;
            let mut buffer = Vec::new();
            let mut limited = stdout.take((max_probe_bytes + 1) as u64);
            let read_result = limited.read_to_end(&mut buffer);
            (buffer, read_result)
        });

        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let (buffer, read_result) = reader
                        .join()
                        .map_err(|_| MediaError::InvalidJob("ffprobe reader thread panicked".into()))?;
                    read_result
                        .map_err(|e| MediaError::InvalidJob(format!("ffprobe output read failed: {e}")))?;
                    if buffer.len() > max_probe_bytes {
                        return Err(MediaError::InvalidJob(
                            "ffprobe output exceeds safety limit".into(),
                        ));
                    }
                    if !status.success() {
                        return Err(MediaError::InvalidJob(format!(
                            "ffprobe exited with status {status}"
                        )));
                    }
                    return parse_probe(
                        &buffer,
                        std::fs::metadata(path)
                            .map_err(|e| MediaError::InvalidJob(format!("cannot stat output media: {e}")))?
                            .len(),
                    );
                }
                Ok(None) => {
                    if started.elapsed() >= self.timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        let _ = reader.join();
                        return Err(MediaError::InvalidJob(
                            "ffprobe execution timed out and was terminated".into(),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = reader.join();
                    return Err(MediaError::InvalidJob(format!(
                        "ffprobe process polling failed: {error}"
                    )));
                }
            }
        }
    }
}

impl MediaExecutor for FfmpegExecutor {
    fn execute(&self, job: &MediaJob) -> Result<(), MediaError> {
        let mut args = ffmpeg_args(job)?;
        let input = self.safe_path(&job.input_path)?;
        let output = self.safe_path(&job.output_path)?;

        if !input.exists() {
            return Err(MediaError::InvalidJob("input media does not exist".into()));
        }
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MediaError::InvalidJob(format!("cannot create output directory: {e}"))
            })?;
        }

        args[4] = input.to_string_lossy().into_owned();
        let last = args.len().saturating_sub(1);
        args[last] = output.to_string_lossy().into_owned();

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

fn parse_probe(bytes: &[u8], file_size_bytes: u64) -> Result<MediaProbe, MediaError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| MediaError::InvalidJob(format!("invalid ffprobe JSON: {e}")))?;
    let streams = value
        .get("streams")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MediaError::InvalidJob("ffprobe stream list missing".into()))?;

    let video = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("video"))
        .ok_or_else(|| MediaError::InvalidJob("video stream missing".into()))?;
    let audio_present = streams
        .iter()
        .any(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio"));

    let duration = value
        .get("format")
        .and_then(|v| v.get("duration"))
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .ok_or_else(|| MediaError::InvalidJob("media duration missing".into()))?;

    let (fps_num, fps_den) = video
        .get("r_frame_rate")
        .and_then(|v| v.as_str())
        .and_then(|value| value.split_once('/'))
        .and_then(|(n, d)| Some((n.parse::<u32>().ok()?, d.parse::<u32>().ok()?)))
        .filter(|(_, d)| *d != 0)
        .unwrap_or((0, 1));

    Ok(MediaProbe {
        duration_millis: (duration * 1000.0).round() as u64,
        width: video.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        height: video.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        fps_num,
        fps_den,
        codec: video
            .get("codec_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned(),
        audio_present,
        file_size_bytes,
    })
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
    fn safe_path_is_lexically_restricted_before_execution() {
        let temp =
            std::env::temp_dir().join(format!("company-agents-media-{}", std::process::id()));
        std::fs::create_dir_all(temp.join("input")).unwrap();
        std::fs::write(temp.join("input").join("source.mp4"), b"test").unwrap();
        let executor = FfmpegExecutor::new("ffmpeg", &temp, Duration::from_secs(10)).unwrap();
        assert!(executor.safe_path("input/source.mp4").is_ok());
        assert!(executor.safe_path("../outside.mp4").is_err());
        let _ = std::fs::remove_dir_all(temp);
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
