use std::{
    fs,
    path::PathBuf,
    process::Command,
};

use serde::Serialize;
use tauri::AppHandle;

use crate::{
    commands::app::resolve_ffmpeg_path,
    constants::WORKING_DIR,
    domain::{job::Job, preset::Preset, video_item::VideoItem},
    services::{
        analysis_service::{sanitize_for_path, BoundingBox, SubtitleRegion},
        logging_service::LoggingService,
    },
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoOverlayResult {
    pub output_path: String,
}

#[derive(Debug, Default)]
pub struct RenderService;

impl RenderService {
    pub fn extract_preview_clip(
        app_handle: &AppHandle,
        input_video_path: &str,
        output_path: &PathBuf,
        time_seconds: f64,
        duration_seconds: f64,
    ) -> Result<(), String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Khong tao duoc preview output dir: {error}"))?;
        }

        let args = vec![
            "-ss".to_string(),
            format!("{time_seconds:.3}"),
            "-i".to_string(),
            input_video_path.to_string(),
            "-t".to_string(),
            format!("{:.3}", duration_seconds.max(0.5)),
            "-an".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-crf".to_string(),
            "20".to_string(),
            "-movflags".to_string(),
            "+faststart".to_string(),
            "-y".to_string(),
            output_path.display().to_string(),
        ];

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg preview extract: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "FFmpeg preview extract that bai: {}",
                stderr.chars().take(240).collect::<String>()
            ));
        }

        Ok(())
    }

    pub fn overlay_logo(
        app_handle: &AppHandle,
        job: &Job,
        video: &VideoItem,
        input_video_path: &str,
        logo_path: &str,
        bounding_box: &BoundingBox,
    ) -> Result<LogoOverlayResult, String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        let output_path = working_output_path(job, video);
        let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);
        fs::create_dir_all(&working_dir).map_err(|error| format!("Khong tao duoc working dir: {error}"))?;

        // Scale logo to bounding box dimensions (derived from logo's own aspect ratio via
        // estimate_logo_size, already clamped to video bounds in build_bounding_box).
        let filter = format!(
            "[1:v]scale={}:{}[logo];[0:v][logo]overlay={}:{}:format=auto[outv]",
            bounding_box.width, bounding_box.height, bounding_box.x, bounding_box.y
        );
        let args = vec![
            "-i".to_string(),
            input_video_path.to_string(),
            "-i".to_string(),
            logo_path.to_string(),
            "-filter_complex".to_string(),
            filter,
            "-map".to_string(),
            "[outv]".to_string(),
            "-map".to_string(),
            "0:a?".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-crf".to_string(),
            "18".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-y".to_string(),
            output_path.display().to_string(),
        ];

        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "logo-overlay: videoId={}; logoSourcePath={logo_path}; command={} {}",
                video.video_id,
                ffmpeg_path.display(),
                args.join(" ")
            ),
        )?;

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg overlay: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_snippet = stderr.chars().take(240).collect::<String>();
            return Err(format!("FFmpeg overlay logo that bai: {stderr_snippet}"));
        }

        LoggingService::append_video_log(
            job,
            video,
            &format!("logo-overlay-success: output={}", output_path.display()),
        )?;

        Ok(LogoOverlayResult {
            output_path: output_path.display().to_string(),
        })
    }

    pub fn overlay_logo_to_output(
        app_handle: &AppHandle,
        input_video_path: &str,
        output_path: &PathBuf,
        logo_path: &str,
        bounding_box: &BoundingBox,
    ) -> Result<(), String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Khong tao duoc output dir cho logo overlay: {error}"))?;
        }

        let filter = format!(
            "[1:v]scale={}:{}[logo];[0:v][logo]overlay={}:{}:format=auto[outv]",
            bounding_box.width, bounding_box.height, bounding_box.x, bounding_box.y
        );
        let args = vec![
            "-i".to_string(),
            input_video_path.to_string(),
            "-i".to_string(),
            logo_path.to_string(),
            "-filter_complex".to_string(),
            filter,
            "-map".to_string(),
            "[outv]".to_string(),
            "-map".to_string(),
            "0:a?".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-crf".to_string(),
            "18".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-y".to_string(),
            output_path.display().to_string(),
        ];

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg overlay: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_snippet = stderr.chars().take(240).collect::<String>();
            return Err(format!("FFmpeg overlay logo that bai: {stderr_snippet}"));
        }

        Ok(())
    }

    /// Removes hardcoded subtitle from the detected region using the mode specified in the
    /// preset (default: boxblur). V1: one mode applied to the entire video duration.
    pub fn remove_subtitle(
        app_handle: &AppHandle,
        job: &Job,
        video: &VideoItem,
        input_video_path: &str,
        region: &SubtitleRegion,
        preset: Option<&Preset>,
    ) -> Result<SubtitleRemovalResult, String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        let output_path = subtitle_removal_output_path(job, video);
        let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);
        fs::create_dir_all(&working_dir).map_err(|error| format!("Khong tao duoc working dir: {error}"))?;

        let mode = removal_mode_from_preset(preset);
        let filter = build_removal_filter(&mode, region);
        // drawbox is a simple filter (no named labels) → -vf works.
        // boxblur uses [0:v]split…[outv] named labels → must use -filter_complex.
        let args = if mode == "drawbox" {
            vec![
                "-i".to_string(), input_video_path.to_string(),
                "-vf".to_string(), filter.clone(),
                "-c:v".to_string(), "libx264".to_string(),
                "-preset".to_string(), "veryfast".to_string(),
                "-crf".to_string(), "18".to_string(),
                "-c:a".to_string(), "copy".to_string(),
                "-y".to_string(), output_path.display().to_string(),
            ]
        } else {
            vec![
                "-i".to_string(), input_video_path.to_string(),
                "-filter_complex".to_string(), filter.clone(),
                "-map".to_string(), "[outv]".to_string(),
                "-map".to_string(), "0:a?".to_string(),
                "-c:v".to_string(), "libx264".to_string(),
                "-preset".to_string(), "veryfast".to_string(),
                "-crf".to_string(), "18".to_string(),
                "-c:a".to_string(), "copy".to_string(),
                "-y".to_string(), output_path.display().to_string(),
            ]
        };
        let vf_filter = filter;

        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "subtitle-removal: videoId={}; mode={mode}; filter={vf_filter}; command={} {}",
                video.video_id,
                ffmpeg_path.display(),
                args.join(" "),
            ),
        )?;

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg subtitle removal: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "FFmpeg subtitle removal that bai: {}",
                stderr.chars().take(240).collect::<String>()
            ));
        }

        LoggingService::append_video_log(
            job,
            video,
            &format!("subtitle-removal-success: output={}", output_path.display()),
        )?;

        Ok(SubtitleRemovalResult {
            output_path: output_path.display().to_string(),
            mode_applied: mode,
        })
    }

    pub fn remove_subtitle_to_output(
        app_handle: &AppHandle,
        input_video_path: &str,
        output_path: &PathBuf,
        region: &SubtitleRegion,
        mode: &str,
    ) -> Result<(), String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Khong tao duoc output dir cho subtitle removal: {error}"))?;
        }

        let drawbox_mode = matches!(mode, "drawbox" | "mask" | "fill");
        let filter = if drawbox_mode {
            format!(
                "drawbox=x={}:y={}:w={}:h={}:color=black:t=fill",
                region.x, region.y, region.width, region.height
            )
        } else {
            build_removal_filter(mode, region)
        };

        let args = if drawbox_mode {
            vec![
                "-i".to_string(), input_video_path.to_string(),
                "-vf".to_string(), filter,
                "-c:v".to_string(), "libx264".to_string(),
                "-preset".to_string(), "veryfast".to_string(),
                "-crf".to_string(), "18".to_string(),
                "-c:a".to_string(), "copy".to_string(),
                "-y".to_string(), output_path.display().to_string(),
            ]
        } else {
            vec![
                "-i".to_string(), input_video_path.to_string(),
                "-filter_complex".to_string(), filter,
                "-map".to_string(), "[outv]".to_string(),
                "-map".to_string(), "0:a?".to_string(),
                "-c:v".to_string(), "libx264".to_string(),
                "-preset".to_string(), "veryfast".to_string(),
                "-crf".to_string(), "18".to_string(),
                "-c:a".to_string(), "copy".to_string(),
                "-y".to_string(), output_path.display().to_string(),
            ]
        };

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg subtitle removal: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "FFmpeg subtitle removal that bai: {}",
                stderr.chars().take(240).collect::<String>()
            ));
        }

        Ok(())
    }

    /// Burns a new SRT subtitle into the video using FFmpeg's `subtitles` filter (libass).
    /// Output file: `{job}/working/{videoId}_subtitle_rendered.mp4`.
    pub fn render_subtitle(
        app_handle: &AppHandle,
        job: &Job,
        video: &VideoItem,
        input_video_path: &str,
        srt_path: &str,
        preset: Option<&Preset>,
    ) -> Result<SubtitleRenderResult, String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        let output_path = subtitle_render_output_path(job, video);
        let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);
        fs::create_dir_all(&working_dir).map_err(|error| format!("Khong tao duoc working dir: {error}"))?;

        validate_srt_encoding(srt_path, job, video)?;

        let force_style = subtitle_style_from_preset(preset);
        // Escape colons in the SRT path for FFmpeg filter syntax (Windows paths contain C:\...)
        let escaped_srt = srt_path.replace('\\', "/").replace(':', "\\:");
        // Escape single quotes inside the force_style value so they don't break the enclosing '...'
        let escaped_force_style = force_style.replace('\'', "\\'");
        let vf_filter = format!("subtitles={escaped_srt}:force_style='{escaped_force_style}'");
        let args = vec![
            "-i".to_string(),
            input_video_path.to_string(),
            "-vf".to_string(),
            vf_filter.clone(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-crf".to_string(),
            "18".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-y".to_string(),
            output_path.display().to_string(),
        ];

        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "subtitle-render: videoId={}; srtPath={srt_path}; filter={vf_filter}; command={} {}",
                video.video_id,
                ffmpeg_path.display(),
                args.join(" "),
            ),
        )?;

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg subtitle render: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "FFmpeg subtitle render that bai: {}",
                stderr.chars().take(240).collect::<String>()
            ));
        }

        LoggingService::append_video_log(
            job,
            video,
            &format!("subtitle-render-success: output={}", output_path.display()),
        )?;

        Ok(SubtitleRenderResult {
            output_path: output_path.display().to_string(),
        })
    }

    pub fn render_subtitle_to_output(
        app_handle: &AppHandle,
        input_video_path: &str,
        output_path: &PathBuf,
        srt_path: &str,
        force_style: &str,
    ) -> Result<(), String> {
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Khong tao duoc output dir cho subtitle render: {error}"))?;
        }

        let escaped_srt = srt_path.replace('\\', "/").replace(':', "\\:");
        let escaped_force_style = force_style.replace('\'', "\\'");
        let vf_filter = format!("subtitles={escaped_srt}:force_style='{escaped_force_style}'");
        let args = vec![
            "-i".to_string(),
            input_video_path.to_string(),
            "-vf".to_string(),
            vf_filter,
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-crf".to_string(),
            "18".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-y".to_string(),
            output_path.display().to_string(),
        ];

        let output = Command::new(&ffmpeg_path)
            .args(&args)
            .output()
            .map_err(|error| format!("Khong the chay FFmpeg subtitle render: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "FFmpeg subtitle render that bai: {}",
                stderr.chars().take(240).collect::<String>()
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleRemovalResult {
    pub output_path: String,
    pub mode_applied: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleRenderResult {
    pub output_path: String,
}

fn working_output_path(job: &Job, video: &VideoItem) -> PathBuf {
    let safe_id = sanitize_for_path(&video.video_id);
    PathBuf::from(&job.output_folder)
        .join(WORKING_DIR)
        .join(format!("{safe_id}_logo_replaced.mp4"))
}

fn subtitle_removal_output_path(job: &Job, video: &VideoItem) -> PathBuf {
    let safe_id = sanitize_for_path(&video.video_id);
    PathBuf::from(&job.output_folder)
        .join(WORKING_DIR)
        .join(format!("{safe_id}_subtitle_removed.mp4"))
}

fn subtitle_render_output_path(job: &Job, video: &VideoItem) -> PathBuf {
    let safe_id = sanitize_for_path(&video.video_id);
    PathBuf::from(&job.output_folder)
        .join(WORKING_DIR)
        .join(format!("{safe_id}_subtitle_rendered.mp4"))
}

/// Parses removal mode from preset.subtitle_style_preset.
/// Looks for keyword "drawbox" or "blackfill"; defaults to "boxblur".
fn removal_mode_from_preset(preset: Option<&Preset>) -> String {
    let style = preset
        .map(|p| p.subtitle_style_preset.to_ascii_lowercase())
        .unwrap_or_default();

    if style.contains("drawbox") || style.contains("blackfill") {
        "drawbox".to_string()
    } else {
        "boxblur".to_string()
    }
}

/// Builds an FFmpeg -vf filter to remove/obscure the subtitle region.
/// `boxblur`: crop the band, blur it, overlay back. `drawbox`: fill with black.
fn build_removal_filter(mode: &str, region: &SubtitleRegion) -> String {
    if mode == "drawbox" {
        format!(
            "drawbox=x={}:y={}:w={}:h={}:color=black:t=fill",
            region.x, region.y, region.width, region.height
        )
    } else {
        // boxblur: crop the subtitle band, blur it, overlay back at original position.
        format!(
            "[0:v]split[orig][copy];[copy]crop={}:{}:{}:{},boxblur=10:1[blurred];[orig][blurred]overlay={}:{}[outv]",
            region.width, region.height, region.x, region.y,
            region.x, region.y
        )
    }
}

/// Extracts force_style string from preset.subtitle_style_preset.
/// If the preset already contains "FontName=", treat as raw FFmpeg style.
/// Otherwise use a sensible default.
fn subtitle_style_from_preset(preset: Option<&Preset>) -> String {
    let style = preset
        .map(|p| p.subtitle_style_preset.as_str())
        .unwrap_or("");

    if style.contains("FontName=") || style.contains("Fontname=") {
        style.to_string()
    } else {
        "FontName=Arial,FontSize=24,PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,Outline=1".to_string()
    }
}

/// Validates SRT file encoding. Logs a warning if not UTF-8 but does NOT fail the job.
fn validate_srt_encoding(srt_path: &str, job: &Job, video: &VideoItem) -> Result<(), String> {
    match fs::read(srt_path) {
        Ok(bytes) => {
            if std::str::from_utf8(&bytes).is_err() {
                let _ = LoggingService::append_video_log(
                    job,
                    video,
                    &format!("subtitle-warning: SRT file `{srt_path}` khong phai UTF-8; co the bi loi ky tu"),
                );
            }
            Ok(())
        }
        Err(error) => Err(format!("Khong doc duoc SRT file `{srt_path}`: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn builds_logo_output_path_in_working_directory() {
        let job = Job {
            job_id: "job-1".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some("replace-logo".to_string()),
            preset_id: None,
            output_folder: "D:/jobs/job-1".to_string(),
            export_output_folder: None,
            status: "Draft".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };
        let video = VideoItem {
            video_id: "video-1".to_string(),
            source_path: "D:/clips/video-1.mp4".to_string(),
            source_metadata: None,
            mapped_logo_path: None,
            mapped_audio_path: None,
            mapped_srt_path: None,
            status: "Imported".to_string(),
        };

        let output = working_output_path(&job, &video);
        assert_eq!(output, Path::new("D:/jobs/job-1/working/video-1_logo_replaced.mp4"));
    }

    #[test]
    fn sanitizes_video_id_with_separators_in_output_path() {
        let job = Job {
            job_id: "job-1".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: None,
            preset_id: None,
            output_folder: "D:/jobs/job-1".to_string(),
            export_output_folder: None,
            status: "Draft".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };
        let video = VideoItem {
            video_id: "bad/video\\id".to_string(),
            source_path: "".to_string(),
            source_metadata: None,
            mapped_logo_path: None,
            mapped_audio_path: None,
            mapped_srt_path: None,
            status: "Imported".to_string(),
        };

        let output = working_output_path(&job, &video);
        let filename = output.file_name().unwrap().to_str().unwrap();
        assert!(!filename.contains('/'));
        assert!(!filename.contains('\\'));
    }

    // ─── Subtitle helper tests ─────────────────────────────────────────────────

    #[test]
    fn subtitle_render_output_path_uses_subtitle_rendered_suffix() {
        let job = Job {
            job_id: "j".to_string(),
            created_at: "".to_string(),
            selected_task: None,
            preset_id: None,
            output_folder: "D:/jobs/j".to_string(),
            export_output_folder: None,
            status: "".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };
        let video = VideoItem {
            video_id: "vid-1".to_string(),
            source_path: "".to_string(),
            source_metadata: None,
            mapped_logo_path: None,
            mapped_audio_path: None,
            mapped_srt_path: None,
            status: "".to_string(),
        };
        let out = subtitle_render_output_path(&job, &video);
        assert!(out.to_str().unwrap().ends_with("vid-1_subtitle_rendered.mp4"));
    }

    #[test]
    fn removal_mode_defaults_to_boxblur() {
        assert_eq!(removal_mode_from_preset(None), "boxblur");
    }

    #[test]
    fn removal_mode_drawbox_from_preset_keyword() {
        let preset = Preset {
            preset_id: "p".to_string(),
            brand_name: "B".to_string(),
            default_logo_path: "".to_string(),
            audio_replacement_policy: "".to_string(),
            subtitle_style_preset: "drawbox style".to_string(),
            layout_rules: "".to_string(),
            export_preset: "".to_string(),
            notes: "".to_string(),
        };
        assert_eq!(removal_mode_from_preset(Some(&preset)), "drawbox");
    }

    #[test]
    fn boxblur_removal_filter_contains_expected_fragments() {
        let region = SubtitleRegion { x: 0, y: 810, width: 1920, height: 216, confidence: 0.8 };
        let filter = build_removal_filter("boxblur", &region);
        assert!(filter.contains("boxblur=10:1"));
        assert!(filter.contains("overlay=0:810"));
    }

    #[test]
    fn drawbox_removal_filter_uses_black_fill() {
        let region = SubtitleRegion { x: 0, y: 810, width: 1920, height: 216, confidence: 0.8 };
        let filter = build_removal_filter("drawbox", &region);
        assert!(filter.contains("drawbox=x=0:y=810"));
        assert!(filter.contains("color=black:t=fill"));
    }

    #[test]
    fn subtitle_style_falls_back_to_arial_when_preset_is_generic() {
        let style = subtitle_style_from_preset(None);
        assert!(style.contains("FontName=Arial"));
    }

    #[test]
    fn subtitle_style_uses_preset_when_it_contains_font_name() {
        let preset = Preset {
            preset_id: "p".to_string(),
            brand_name: "B".to_string(),
            default_logo_path: "".to_string(),
            audio_replacement_policy: "".to_string(),
            subtitle_style_preset: "FontName=Roboto,FontSize=28".to_string(),
            layout_rules: "".to_string(),
            export_preset: "".to_string(),
            notes: "".to_string(),
        };
        let style = subtitle_style_from_preset(Some(&preset));
        assert_eq!(style, "FontName=Roboto,FontSize=28");
    }
}
