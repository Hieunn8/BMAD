use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Serialize;
use tauri::AppHandle;

use crate::{
    commands::app::resolve_ffmpeg_path,
    constants::{LOGS_DIR, WORKING_DIR},
    domain::{job::Job, preset::Preset, video_item::VideoItem},
    services::audio_policy_service::AudioPolicyService,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioReplacementResult {
    pub skipped: bool,
    pub output_path: Option<String>,
}

#[derive(Debug, Default)]
pub struct AudioReplacementService;

impl AudioReplacementService {
    pub fn replace_audio(
        app_handle: &AppHandle,
        job: &Job,
        preset: Option<&Preset>,
        video: &VideoItem,
    ) -> Result<AudioReplacementResult, String> {
        let should_run = preset
            .map(|value| AudioPolicyService::should_replace_audio(&value.audio_replacement_policy))
            .unwrap_or(true);

        if !should_run {
            log_audio_step(job, video, "skip: audio replacement policy = NoReplacement")?;
            return Ok(AudioReplacementResult {
                skipped: true,
                output_path: None,
            });
        }

        let audio_path = video
            .mapped_audio_path
            .as_deref()
            .ok_or_else(|| format!("Video `{}` chua co audio mapping", video.video_id))?;

        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        let output_path = working_output_path(job, video);
        let args = ffmpeg_args(video.source_path.as_str(), audio_path, &output_path);

        log_audio_step(
            job,
            video,
            &format!(
                "start: videoId={}; audioSourcePath={audio_path}; command={} {}",
                video.video_id,
                ffmpeg_path.display(),
                args.join(" ")
            ),
        )?;

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_snippet = stderr.chars().take(240).collect::<String>();
            log_audio_step(
                job,
                video,
                &format!(
                    "error: exit_code={:?}; stderr={stderr_snippet}",
                    output.status.code()
                ),
            )?;

            return Err(format!(
                "FFmpeg thay audio that bai cho video `{}`: {stderr_snippet}",
                video.video_id
            ));
        }

        log_audio_step(
            job,
            video,
            &format!("success: output={}", output_path.display()),
        )?;

        Ok(AudioReplacementResult {
            skipped: false,
            output_path: Some(output_path.display().to_string()),
        })
    }
}

fn ffmpeg_args(video_path: &str, audio_path: &str, output_path: &Path) -> Vec<String> {
    vec![
        "-i".to_string(),
        video_path.to_string(),
        "-i".to_string(),
        audio_path.to_string(),
        "-map".to_string(),
        "0:v:0".to_string(),
        "-map".to_string(),
        "1:a:0".to_string(),
        "-c:v".to_string(),
        "copy".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-shortest".to_string(),
        "-y".to_string(),
        output_path.display().to_string(),
    ]
}

fn working_output_path(job: &Job, video: &VideoItem) -> PathBuf {
    PathBuf::from(&job.output_folder)
        .join(WORKING_DIR)
        .join(format!("{}_audio_replaced.mp4", video.video_id))
}

fn log_audio_step(job: &Job, video: &VideoItem, line: &str) -> Result<(), String> {
    let log_dir = PathBuf::from(&job.output_folder).join(LOGS_DIR);
    fs::create_dir_all(&log_dir).map_err(|error| format!("Khong tao duoc log dir: {error}"))?;

    let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);
    fs::create_dir_all(&working_dir)
        .map_err(|error| format!("Khong tao duoc working dir: {error}"))?;

    let log_path = log_dir.join(format!("{}.log", video.video_id));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| format!("Khong mo duoc log file `{}`: {error}", log_path.display()))?;

    writeln!(file, "{line}").map_err(|error| format!("Khong ghi duoc log file: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_ffmpeg_arguments() {
        let args = ffmpeg_args(
            "D:/clips/input.mp4",
            "D:/audio/new.mp3",
            Path::new("D:/jobs/job-1/working/video-1_audio_replaced.mp4"),
        );

        assert!(args.windows(2).any(|pair| pair == ["-map", "0:v:0"]));
        assert!(args.windows(2).any(|pair| pair == ["-map", "1:a:0"]));
        assert!(args.windows(2).any(|pair| pair == ["-c:v", "copy"]));
        assert!(args.windows(2).any(|pair| pair == ["-c:a", "aac"]));
        assert!(args.contains(&"-shortest".to_string()));
    }
}
