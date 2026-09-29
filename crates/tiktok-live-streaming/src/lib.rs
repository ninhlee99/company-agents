#![forbid(unsafe_code)]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

#[derive(Debug, Clone)]
pub struct LiveStreamConfig {
    pub ffmpeg_bin: PathBuf,
    pub input_path: PathBuf,
    pub audio_path: Option<PathBuf>,
    pub overlay_path: PathBuf,
    pub destination: String,
    pub fps: u32,
}

impl LiveStreamConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=60).contains(&self.fps) {
            return Err("LIVE fps must be between 1 and 60".into());
        }
        if !(self.destination.starts_with("rtmp://")
            || self.destination.starts_with("rtmps://"))
        {
            return Err("LIVE destination must use RTMP or RTMPS".into());
        }
        if self.input_path.as_os_str().is_empty() {
            return Err("LIVE input path is required".into());
        }
        if self.overlay_path.as_os_str().is_empty() {
            return Err("LIVE overlay path is required".into());
        }
        Ok(())
    }
}

pub struct LiveStreamController {
    config: LiveStreamConfig,
    child: Mutex<Option<Child>>,
}

impl LiveStreamController {
    pub fn new(config: LiveStreamConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            config,
            child: Mutex::new(None),
        })
    }

    pub fn from_env() -> Result<Self, String> {
        let input_path = std::env::var("TIKTOK_LIVE_INPUT_PATH")
            .map(PathBuf::from)
            .map_err(|_| "TIKTOK_LIVE_INPUT_PATH is required".to_string())?;
        let overlay_path = std::env::var("TIKTOK_LIVE_OVERLAY_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| input_path.with_extension("live-overlay.txt"));
        let destination = std::env::var("TIKTOK_LIVE_STREAM_DESTINATION")
            .map_err(|_| "TIKTOK_LIVE_STREAM_DESTINATION is required".to_string())?;
        let audio_path = std::env::var("TIKTOK_LIVE_AUDIO_PATH")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);
        let fps = std::env::var("TIKTOK_LIVE_FPS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(30);
        Self::new(LiveStreamConfig {
            ffmpeg_bin: PathBuf::from(
                std::env::var("FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".into()),
            ),
            input_path,
            audio_path,
            overlay_path,
            destination,
            fps,
        })
    }

    pub fn args(&self) -> Vec<String> {
        let fps = self.config.fps.to_string();
        let overlay = self.config.overlay_path.to_string_lossy().into_owned();
        let mut args = vec![
            "-hide_banner".into(),
            "-nostdin".into(),
            "-re".into(),
            "-stream_loop".into(),
            "-1".into(),
            "-i".into(),
            self.config.input_path.to_string_lossy().into_owned(),
        ];
        if let Some(audio) = &self.config.audio_path {
            args.extend([
                "-stream_loop".into(),
                "-1".into(),
                "-i".into(),
                audio.to_string_lossy().into_owned(),
            ]);
        }
        args.extend([
            "-vf".into(),
            format!(
                "drawtext=textfile={overlay}:reload=1:fontcolor=white:fontsize=44:box=1:boxcolor=black@0.55:x=(w-text_w)/2:y=h-text_h-40"
            ),
            "-map".into(),
            "0:v:0".into(),
        ]);
        if self.config.audio_path.is_some() {
            args.extend(["-map".into(), "1:a:0".into()]);
        }
        args.extend([
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "veryfast".into(),
            "-tune".into(),
            "zerolatency".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-r".into(),
            fps,
            "-g".into(),
            (self.config.fps * 2).to_string(),
        ]);
        if self.config.audio_path.is_some() {
            args.extend([
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "128k".into(),
            ]);
        } else {
            args.push("-an".into());
        }
        args.extend([
            "-f".into(),
            "flv".into(),
            self.config.destination.clone(),
        ]);
        args
    }

    pub fn start(&self, initial_overlay: &str) -> Result<(), String> {
        let mut guard = self.child.lock().map_err(|_| "LIVE process lock poisoned".to_string())?;
        if guard.as_mut().is_some_and(|child| child.try_wait().ok().flatten().is_none()) {
            return Err("LIVE stream is already running".into());
        }
        write_overlay(&self.config.overlay_path, initial_overlay)?;
        if !self.config.input_path.is_file() {
            return Err("LIVE input media does not exist".into());
        }
        if let Some(audio) = &self.config.audio_path {
            if !audio.is_file() {
                return Err("LIVE audio media does not exist".into());
            }
        }
        if let Some(parent) = self.config.overlay_path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("cannot create overlay directory: {error}"))?;
        }
        let child = Command::new(&self.config.ffmpeg_bin)
            .args(self.args())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to start FFmpeg LIVE stream: {error}"))?;
        *guard = Some(child);
        Ok(())
    }

    pub fn update_overlay(&self, text: &str) -> Result<(), String> {
        write_overlay(&self.config.overlay_path, text)
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut guard = self.child.lock().map_err(|_| "LIVE process lock poisoned".to_string())?;
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn is_running(&self) -> Result<bool, String> {
        let mut guard = self.child.lock().map_err(|_| "LIVE process lock poisoned".to_string())?;
        match guard.as_mut() {
            Some(child) => Ok(child
                .try_wait()
                .map_err(|error| format!("LIVE process status failed: {error}"))?
                .is_none()),
            None => Ok(false),
        }
    }
}

fn write_overlay(path: &Path, text: &str) -> Result<(), String> {
    if text.chars().count() > 2_000 {
        return Err("LIVE overlay text is too long".into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create overlay directory: {error}"))?;
    }
    let tmp = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&tmp)
        .map_err(|error| format!("cannot open overlay temp file: {error}"))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("cannot write overlay: {error}"))?;
    file.sync_data()
        .map_err(|error| format!("cannot sync overlay: {error}"))?;
    fs::rename(&tmp, path).map_err(|error| format!("cannot publish overlay atomically: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_rejects_non_rtmp_destination() {
        let config = LiveStreamConfig {
            ffmpeg_bin: "ffmpeg".into(),
            input_path: "video.mp4".into(),
            audio_path: None,
            overlay_path: "overlay.txt".into(),
            destination: "https://example.invalid".into(),
            fps: 30,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn args_are_looped_and_low_latency() {
        let controller = LiveStreamController::new(LiveStreamConfig {
            ffmpeg_bin: "ffmpeg".into(),
            input_path: "video.mp4".into(),
            audio_path: Some("music.mp3".into()),
            overlay_path: "overlay.txt".into(),
            destination: "rtmps://example.invalid/live".into(),
            fps: 30,
        })
        .unwrap();
        let args = controller.args();
        assert!(args.windows(2).any(|pair| pair == ["-stream_loop", "-1"]));
        assert!(args.iter().any(|value| value == "zerolatency"));
        assert!(args.iter().any(|value| value == "rtmps://example.invalid/live"));
    }

    #[test]
    fn overlay_is_bounded() {
        let config = LiveStreamConfig {
            ffmpeg_bin: "ffmpeg".into(),
            input_path: "video.mp4".into(),
            audio_path: None,
            overlay_path: "overlay.txt".into(),
            destination: "rtmp://example.invalid/live".into(),
            fps: 30,
        };
        let controller = LiveStreamController::new(config).unwrap();
        assert!(controller.update_overlay(&"x".repeat(2_001)).is_err());
    }
}
