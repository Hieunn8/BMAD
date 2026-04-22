use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::{
    commands::app::resolve_ffmpeg_path,
    constants::{REPORTS_DIR, SEGMENT_STATE_DIR, VIDEO_STATE_DIR, WORKING_DIR},
    domain::{job::Job, preset::Preset, video_item::VideoItem},
    services::{
        analysis_service::{sanitize_for_path, AnalysisService, BoundingBox, VideoMetadata},
        audio_policy_service::AudioPolicyService,
        job_orchestrator::load_job,
        logging_service::LoggingService,
        persistence_service::{make_video_state, PersistedEncodeSummary, PersistenceService, VideoProcessingState},
        preset_service::PresetService,
        render_service::RenderService,
    },
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPresetSummary {
    pub label: String,
    pub codec: String,
    pub crf: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReadyVideo {
    pub video_id: String,
    pub video_name: String,
    pub status: String,
    pub audio_summary: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBlockedVideo {
    pub video_id: String,
    pub video_name: String,
    pub status: String,
    pub reason: String,
    pub audio_summary: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReadiness {
    pub ready_videos: Vec<ExportReadyVideo>,
    pub blocked_videos: Vec<ExportBlockedVideo>,
    pub output_folder: String,
    pub preset_summary: Option<ExportPresetSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoExportStartedEvent {
    pub video_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgressEvent {
    pub video_id: String,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoExportCompletedEvent {
    pub video_id: String,
    pub success: bool,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchExportCompletedEvent {
    pub job_id: String,
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceSummary {
    pub policy: String,
    pub audio_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentStats {
    pub total: usize,
    pub flagged: usize,
    pub modified: usize,
    pub accepted: usize,
    pub high_risk_remaining: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotCheckThumbnail {
    pub segment_id: String,
    pub before_path: Option<String>,
    pub after_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoReport {
    pub video_id: String,
    pub video_name: String,
    pub final_status: String,
    pub encode_summary: Option<PersistedEncodeSummary>,
    pub audio_source: AudioSourceSummary,
    pub segment_stats: SegmentStats,
    pub spot_check_thumbnails: Vec<SpotCheckThumbnail>,
    pub output_path: Option<String>,
    pub report_generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobExportSummaryReport {
    pub job_id: String,
    pub total_videos: usize,
    pub success: usize,
    pub failed: usize,
    pub total_output_size_mb: f64,
    pub reports: Vec<VideoReport>,
    pub generated_at: String,
}

#[derive(Debug, Clone)]
pub struct StartExportResult {
    pub job: Job,
    pub started: bool,
}

#[derive(Debug, Default)]
pub struct ExportService;

impl ExportService {
    pub fn get_export_readiness(app_handle: &AppHandle, job_id: &str) -> Result<ExportReadiness, String> {
        let job = load_job(app_handle, job_id)?;
        let preset = match job.preset_id.as_deref() {
            Some(preset_id) => Some(PresetService::get_preset(app_handle, preset_id)?),
            None => None,
        };

        let mut ready_videos = Vec::new();
        let mut blocked_videos = Vec::new();

        for video in &job.video_items {
            let audio_summary = audio_summary_for_video(&job, preset.as_ref(), video.mapped_audio_path.as_deref());
            let video_name = video_name(video.source_path.as_str());

            if video.status == "ReadyToExport" {
                ready_videos.push(ExportReadyVideo {
                    video_id: video.video_id.clone(),
                    video_name,
                    status: video.status.clone(),
                    audio_summary,
                });
            } else {
                blocked_videos.push(ExportBlockedVideo {
                    video_id: video.video_id.clone(),
                    video_name,
                    status: video.status.clone(),
                    reason: blocked_reason(&video.status),
                    audio_summary,
                });
            }
        }

        Ok(ExportReadiness {
            ready_videos,
            blocked_videos,
            output_folder: effective_output_folder(&job),
            preset_summary: preset.as_ref().map(preset_summary),
        })
    }

    pub fn set_export_output_folder(
        app_handle: &AppHandle,
        job_id: &str,
        output_folder: &str,
    ) -> Result<Job, String> {
        let mut job = load_job(app_handle, job_id)?;
        let normalized = normalize_folder_path(output_folder);

        for video in &job.video_items {
            let source_parent = PathBuf::from(&video.source_path)
                .parent()
                .map(|value| normalize_folder_path(value.to_string_lossy().as_ref()));

            if source_parent.as_deref() == Some(normalized.as_str()) {
                return Err("Output folder khong duoc trung voi thu muc chua source video".to_string());
            }
        }

        job.export_output_folder = Some(output_folder.to_string());
        PersistenceService::persist_job(app_handle, &job)?;
        Ok(job)
    }

    pub fn start_export(app_handle: &AppHandle, job_id: &str) -> Result<StartExportResult, String> {
        let mut job = load_job(app_handle, job_id)?;
        if job.status == "Exporting" {
            return Ok(StartExportResult { job, started: false });
        }

        let ready_video_ids = job
            .video_items
            .iter()
            .filter(|video| video.status == "ReadyToExport")
            .map(|video| video.video_id.clone())
            .collect::<Vec<_>>();
        if ready_video_ids.is_empty() {
            return Err("Khong co video nao o trang thai ReadyToExport".to_string());
        }

        let output_folder = effective_output_folder(&job);
        fs::create_dir_all(&output_folder)
            .map_err(|error| format!("Khong tao duoc export output folder `{output_folder}`: {error}"))?;

        job.export_output_folder = Some(output_folder.clone());
        job.status = "Exporting".to_string();
        PersistenceService::persist_job(app_handle, &job)?;
        emit_job_updated(app_handle, job.clone())?;

        let runner_handle = app_handle.clone();
        let runner_job_id = job.job_id.clone();
        tauri::async_runtime::spawn(async move {
            let _ = process_export_queue(runner_handle, runner_job_id).await;
        });

        Ok(StartExportResult { job, started: true })
    }

    pub fn generate_video_report(
        app_handle: &AppHandle,
        job_id: &str,
        video_id: &str,
        regenerate: bool,
    ) -> Result<VideoReport, String> {
        let job = load_job(app_handle, job_id)?;
        generate_video_report_internal(app_handle, &job, video_id, regenerate)
    }

    pub fn get_report(app_handle: &AppHandle, job_id: &str, video_id: &str) -> Result<VideoReport, String> {
        let job = load_job(app_handle, job_id)?;
        let report_path = video_report_path(&job, video_id);
        if report_path.exists() {
            return read_video_report(&report_path);
        }
        generate_video_report_internal(app_handle, &job, video_id, false)
    }

    pub fn get_job_summary_report(app_handle: &AppHandle, job_id: &str) -> Result<JobExportSummaryReport, String> {
        let job = load_job(app_handle, job_id)?;
        let summary_path = job_summary_report_path(&job);
        if summary_path.exists() {
            return read_job_summary_report(&summary_path);
        }
        generate_job_summary_report(app_handle, &job, false)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct PersistedQuickFixState {
    #[serde(default)]
    logo_fix: Option<PersistedLogoFix>,
    #[serde(default)]
    subtitle_fix: Option<PersistedSubtitleFix>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedLogoFix {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleRegionFix {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    mode: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitlePositionFix {
    x: u32,
    y: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleFix {
    #[serde(default)]
    old_region: Option<PersistedSubtitleRegionFix>,
    #[serde(default)]
    new_position: Option<PersistedSubtitlePositionFix>,
    #[serde(default)]
    new_scale: Option<f32>,
    #[serde(default)]
    style_preset: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedLogoSegment {
    issue_type: String,
    risk_level: String,
    start_ms: u64,
    end_ms: Option<u64>,
    bounding_box: BoundingBox,
    #[serde(default)]
    review_status: Option<String>,
    #[serde(default)]
    quick_fix_state: Option<PersistedQuickFixState>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleSegment {
    issue_type: String,
    risk_level: String,
    start_ms: u64,
    end_ms: Option<u64>,
    confidence: f32,
    region: BoundingBox,
    #[serde(default)]
    review_status: Option<String>,
    #[serde(default)]
    quick_fix_state: Option<PersistedQuickFixState>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedLogoReview {
    video_id: String,
    segments: Vec<PersistedLogoSegment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleReview {
    video_id: String,
    segments: Vec<PersistedSubtitleSegment>,
}

#[derive(Debug, Clone)]
struct ExportVideoConfig {
    logo_box: Option<BoundingBox>,
    subtitle_region: Option<PersistedSubtitleRegionFix>,
    subtitle_fix: Option<PersistedSubtitleFix>,
    subtitle_confidence: f32,
}

#[derive(Debug, Clone)]
struct ReportSegmentCandidate {
    segment_id: String,
    issue_type: String,
    risk_level: String,
    review_status: String,
    start_ms: u64,
    end_ms: Option<u64>,
}

async fn process_export_queue(app_handle: AppHandle, job_id: String) -> Result<(), String> {
    let mut job = load_job(&app_handle, &job_id)?;
    let preset = load_preset(&app_handle, job.preset_id.as_deref())?;
    let output_folder = effective_output_folder(&job);

    let queued_indexes = job
        .video_items
        .iter()
        .enumerate()
        .filter_map(|(index, video)| (video.status == "ReadyToExport").then_some(index))
        .collect::<Vec<_>>();

    let total = queued_indexes.len();
    let mut success_count = 0usize;
    let mut failed_count = 0usize;

    for index in queued_indexes {
        let video = job.video_items[index].clone();
        let video_id = video.video_id.clone();

        job.video_items[index].status = "Exporting".to_string();
        PersistenceService::persist_job(&app_handle, &job)?;
        let exporting_state = make_export_state(&job, &video, "Exporting", "exporting", None, None);
        PersistenceService::persist_video_state(&job, &exporting_state)?;
        emit_job_updated(&app_handle, job.clone())?;
        emit_video_export_started(&app_handle, &video_id)?;

        let export_result = export_video(&app_handle, &job, &video, preset.as_ref(), &output_folder);
        match export_result {
            Ok(output_path) => {
                job.video_items[index].status = "Exported".to_string();
                PersistenceService::persist_job(&app_handle, &job)?;

                let encode_summary = build_encode_summary(&app_handle, &output_path, preset.as_ref())?;
                let exported_state = make_export_state(
                    &job,
                    &video,
                    "Exported",
                    "exported",
                    Some(output_path.clone()),
                    Some(encode_summary.clone()),
                );
                PersistenceService::persist_video_state(&job, &exported_state)?;
                emit_job_updated(&app_handle, job.clone())?;

                if let Err(error) = generate_video_report_internal(&app_handle, &job, &video_id, true) {
                    let _ = LoggingService::append_video_log(
                        &job,
                        &video,
                        &format!("report-warning: khong tao duoc report sau export: {error}"),
                    );
                }

                emit_video_export_completed(&app_handle, &video_id, true, Some(output_path), None)?;
                success_count += 1;
            }
            Err(error) => {
                job.video_items[index].status = "Failed".to_string();
                PersistenceService::persist_job(&app_handle, &job)?;

                let failed_state = make_export_state(&job, &video, "Failed", "export-failed", None, None);
                PersistenceService::persist_video_state(&job, &failed_state)?;
                emit_job_updated(&app_handle, job.clone())?;

                if let Err(report_error) = generate_video_report_internal(&app_handle, &job, &video_id, true) {
                    let _ = LoggingService::append_video_log(
                        &job,
                        &video,
                        &format!("report-warning: khong tao duoc report failure snapshot: {report_error}"),
                    );
                }

                emit_video_export_completed(&app_handle, &video_id, false, None, Some(error.clone()))?;
                failed_count += 1;
            }
        }
    }

    job.status = if failed_count == 0 {
        "Exported".to_string()
    } else if success_count > 0 {
        "ExportedWithFailures".to_string()
    } else {
        "ExportFailed".to_string()
    };
    PersistenceService::persist_job(&app_handle, &job)?;
    emit_job_updated(&app_handle, job.clone())?;

    if let Err(error) = generate_job_summary_report(&app_handle, &job, true) {
        let _ = LoggingService::append_video_log(
            &job,
            job.video_items.first().unwrap_or(&VideoItem {
                video_id: "job".to_string(),
                source_path: "".to_string(),
                source_metadata: None,
                mapped_logo_path: None,
                mapped_audio_path: None,
                mapped_srt_path: None,
                status: "".to_string(),
            }),
            &format!("report-warning: khong tao duoc job export summary: {error}"),
        );
    }

    emit_batch_export_completed(&app_handle, &job.job_id, total, success_count, failed_count)?;

    Ok(())
}

fn export_video(
    app_handle: &AppHandle,
    job: &Job,
    video: &VideoItem,
    preset: Option<&Preset>,
    output_folder: &str,
) -> Result<String, String> {
    let config = load_export_video_config(job, &video.video_id)?;
    let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);
    fs::create_dir_all(&working_dir).map_err(|error| format!("Khong tao duoc working dir export: {error}"))?;

    let safe_id = sanitize_for_path(&video.video_id);
    let stage_root = working_dir.join(format!("{safe_id}_export_stage"));
    fs::create_dir_all(&stage_root).map_err(|error| format!("Khong tao duoc export stage dir: {error}"))?;

    let mut current_input = resolve_initial_export_input(job, video, &config);

    if let Some(logo_box) = config.logo_box.as_ref() {
        let logo_path = resolve_logo_source(video, preset)?;
        let staged_logo = stage_root.join("logo.mp4");
        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "export-stage-logo: input={}; logo={}; x={}; y={}; w={}; h={}",
                current_input, logo_path, logo_box.x, logo_box.y, logo_box.width, logo_box.height
            ),
        )?;
        RenderService::overlay_logo_to_output(app_handle, &current_input, &staged_logo, &logo_path, logo_box)?;
        current_input = staged_logo.display().to_string();
    }

    if let Some(subtitle_region) = config.subtitle_region.as_ref() {
        let staged_removed = stage_root.join("subtitle_removed.mp4");
        let subtitle_region_payload = crate::services::analysis_service::SubtitleRegion {
            x: subtitle_region.x,
            y: subtitle_region.y,
            width: subtitle_region.width,
            height: subtitle_region.height,
            confidence: config.subtitle_confidence,
        };
        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "export-stage-subtitle-remove: input={}; x={}; y={}; w={}; h={}; mode={}",
                current_input,
                subtitle_region.x,
                subtitle_region.y,
                subtitle_region.width,
                subtitle_region.height,
                subtitle_region.mode
            ),
        )?;
        RenderService::remove_subtitle_to_output(
            app_handle,
            &current_input,
            &staged_removed,
            &subtitle_region_payload,
            &subtitle_region.mode,
        )?;
        current_input = staged_removed.display().to_string();

        if let Some(srt_path) = video.mapped_srt_path.as_deref() {
            let metadata = AnalysisService::probe_metadata(app_handle, &current_input)?;
            let subtitle_fix = config.subtitle_fix.clone().unwrap_or(PersistedSubtitleFix {
                old_region: Some(subtitle_region.clone()),
                new_position: None,
                new_scale: Some(1.0),
                style_preset: preset.map(|value| value.subtitle_style_preset.clone()),
            });
            let force_style = subtitle_force_style_from_fix(preset, &subtitle_fix, &metadata);
            let staged_rendered = stage_root.join("subtitle_rendered.mp4");
            LoggingService::append_video_log(
                job,
                video,
                &format!(
                    "export-stage-subtitle-render: input={}; srt={}; forceStyle={}",
                    current_input, srt_path, force_style
                ),
            )?;
            RenderService::render_subtitle_to_output(
                app_handle,
                &current_input,
                &staged_rendered,
                srt_path,
                &force_style,
            )?;
            current_input = staged_rendered.display().to_string();
        }
    }

    let output_path = final_output_path(output_folder, video);
    let crf = preset.map(|value| parse_crf(&value.export_preset)).unwrap_or_else(|| "20".to_string());
    transcode_with_progress(app_handle, job, video, &current_input, &output_path, &crf)?;
    Ok(output_path.display().to_string())
}

fn generate_video_report_internal(
    app_handle: &AppHandle,
    job: &Job,
    video_id: &str,
    overwrite: bool,
) -> Result<VideoReport, String> {
    let report_path = video_report_path(job, video_id);
    if report_path.exists() && !overwrite {
        return read_video_report(&report_path);
    }

    let video = find_video(job, video_id)?;
    let video_state = load_video_state(job, video_id)?;
    let segment_candidates = load_report_segments(job, video_id)?;
    let segment_stats = build_segment_stats(&segment_candidates);
    let output_path = video_state.output_path.clone().filter(|path| !path.trim().is_empty());
    let spot_check_thumbnails = generate_spot_check_thumbnails(
        app_handle,
        job,
        video,
        output_path.as_deref(),
        &segment_candidates,
    )?;

    let report = VideoReport {
        video_id: video.video_id.clone(),
        video_name: video_name(&video.source_path),
        final_status: video_state.status.clone(),
        encode_summary: video_state.encode_summary.clone(),
        audio_source: AudioSourceSummary {
            policy: audio_policy_label(job, video),
            audio_file_path: video_state.audio_source_path.clone().or_else(|| video.mapped_audio_path.clone()),
        },
        segment_stats,
        spot_check_thumbnails,
        output_path,
        report_generated_at: Utc::now().to_rfc3339(),
    };

    persist_video_report(job, &report)?;
    Ok(report)
}

fn generate_job_summary_report(
    app_handle: &AppHandle,
    job: &Job,
    overwrite: bool,
) -> Result<JobExportSummaryReport, String> {
    let summary_path = job_summary_report_path(job);
    if summary_path.exists() && !overwrite {
        return read_job_summary_report(&summary_path);
    }

    let mut reports = Vec::new();
    for video in &job.video_items {
        if matches!(video.status.as_str(), "Exported" | "Failed") {
            reports.push(generate_video_report_internal(app_handle, job, &video.video_id, overwrite)?);
        }
    }

    let success = reports.iter().filter(|report| report.final_status == "Exported").count();
    let failed = reports.iter().filter(|report| report.final_status == "Failed").count();
    let total_output_size_mb = reports
        .iter()
        .filter_map(|report| report.encode_summary.as_ref().map(|summary| summary.output_size_mb))
        .sum::<f64>();

    let summary = JobExportSummaryReport {
        job_id: job.job_id.clone(),
        total_videos: reports.len(),
        success,
        failed,
        total_output_size_mb: round2(total_output_size_mb),
        reports,
        generated_at: Utc::now().to_rfc3339(),
    };

    persist_job_summary_report(job, &summary)?;
    Ok(summary)
}

fn generate_spot_check_thumbnails(
    app_handle: &AppHandle,
    job: &Job,
    video: &VideoItem,
    output_path: Option<&str>,
    segments: &[ReportSegmentCandidate],
) -> Result<Vec<SpotCheckThumbnail>, String> {
    let report_dir = reports_thumbnail_dir(job);
    fs::create_dir_all(&report_dir).map_err(|error| format!("Khong tao duoc report thumbnails dir: {error}"))?;

    let selected = select_spot_check_segments(segments);
    let mut thumbnails = Vec::new();
    for segment in selected {
        let before_path = report_dir.join(format!("{}-before.jpg", sanitize_for_path(&segment.segment_id)));
        let after_path = report_dir.join(format!("{}-after.jpg", sanitize_for_path(&segment.segment_id)));
        let midpoint = midpoint_seconds(segment.start_ms, segment.end_ms);

        extract_thumbnail_frame(app_handle, &video.source_path, &before_path, midpoint)?;
        let after_result = if let Some(path) = output_path {
            if Path::new(path).exists() {
                extract_thumbnail_frame(app_handle, path, &after_path, midpoint)?;
                Some(after_path.display().to_string())
            } else {
                None
            }
        } else {
            None
        };

        thumbnails.push(SpotCheckThumbnail {
            segment_id: segment.segment_id.clone(),
            before_path: Some(before_path.display().to_string()),
            after_path: after_result,
        });
    }

    Ok(thumbnails)
}

fn extract_thumbnail_frame(
    app_handle: &AppHandle,
    input_path: &str,
    output_path: &Path,
    time_seconds: f64,
) -> Result<(), String> {
    let ffmpeg_path = resolve_ffmpeg_path(app_handle)
        .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("Khong tao duoc report thumbnail dir: {error}"))?;
    }

    let args = vec![
        "-ss".to_string(),
        format!("{time_seconds:.3}"),
        "-i".to_string(),
        input_path.to_string(),
        "-vframes".to_string(),
        "1".to_string(),
        "-q:v".to_string(),
        "2".to_string(),
        "-y".to_string(),
        output_path.display().to_string(),
    ];

    let output = Command::new(&ffmpeg_path)
        .args(&args)
        .output()
        .map_err(|error| format!("Khong the chay FFmpeg thumbnail extract: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "FFmpeg thumbnail extract that bai: {}",
            stderr.chars().take(240).collect::<String>()
        ));
    }

    Ok(())
}

fn load_report_segments(job: &Job, video_id: &str) -> Result<Vec<ReportSegmentCandidate>, String> {
    let mut segments = Vec::new();
    let segments_dir = PathBuf::from(&job.output_folder).join(SEGMENT_STATE_DIR);
    let safe_id = sanitize_for_path(video_id);

    let logo_path = segments_dir.join(format!("{safe_id}_logo.json"));
    if logo_path.exists() {
        let payload = fs::read_to_string(&logo_path)
            .map_err(|error| format!("Khong doc duoc logo segment report `{}`: {error}", logo_path.display()))?;
        let parsed: PersistedLogoReview = serde_json::from_str(&payload)
            .map_err(|error| format!("Khong parse duoc logo segment report `{}`: {error}", logo_path.display()))?;
        segments.extend(parsed.segments.into_iter().enumerate().map(|(index, segment)| ReportSegmentCandidate {
            segment_id: format!("{}:logo:{index}", parsed.video_id),
            issue_type: segment.issue_type,
            risk_level: segment.risk_level,
            review_status: segment.review_status.unwrap_or_else(|| "Unreviewed".to_string()),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
        }));
    }

    let subtitle_path = segments_dir.join(format!("{safe_id}_subtitle.json"));
    if subtitle_path.exists() {
        let payload = fs::read_to_string(&subtitle_path)
            .map_err(|error| format!("Khong doc duoc subtitle segment report `{}`: {error}", subtitle_path.display()))?;
        let parsed: PersistedSubtitleReview = serde_json::from_str(&payload)
            .map_err(|error| format!("Khong parse duoc subtitle segment report `{}`: {error}", subtitle_path.display()))?;
        segments.extend(parsed.segments.into_iter().enumerate().map(|(index, segment)| ReportSegmentCandidate {
            segment_id: format!("{}:subtitle:{index}", parsed.video_id),
            issue_type: segment.issue_type,
            risk_level: segment.risk_level,
            review_status: segment.review_status.unwrap_or_else(|| "Unreviewed".to_string()),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
        }));
    }

    Ok(segments)
}

fn build_segment_stats(segments: &[ReportSegmentCandidate]) -> SegmentStats {
    SegmentStats {
        total: segments.len(),
        flagged: segments
            .iter()
            .filter(|segment| matches!(segment.risk_level.as_str(), "High" | "Medium"))
            .count(),
        modified: segments
            .iter()
            .filter(|segment| segment.review_status == "Modified")
            .count(),
        accepted: segments
            .iter()
            .filter(|segment| segment.review_status == "Accepted")
            .count(),
        high_risk_remaining: segments
            .iter()
            .filter(|segment| segment.risk_level == "High" && segment.review_status == "Unreviewed")
            .count(),
    }
}

fn select_spot_check_segments(segments: &[ReportSegmentCandidate]) -> Vec<ReportSegmentCandidate> {
    let mut ranked = segments.to_vec();
    ranked.sort_by_key(|segment| {
        let priority = if segment.risk_level == "High" && segment.review_status == "Modified" {
            0
        } else if segment.risk_level == "High" && segment.review_status == "Accepted" {
            1
        } else if segment.risk_level == "Medium" && segment.review_status == "Modified" {
            2
        } else if segment.risk_level == "Medium" {
            3
        } else {
            4
        };
        (priority, segment.start_ms)
    });
    ranked.into_iter().take(3).collect()
}

fn midpoint_seconds(start_ms: u64, end_ms: Option<u64>) -> f64 {
    let end = end_ms.unwrap_or(start_ms + 1000);
    let midpoint = start_ms + (end.saturating_sub(start_ms) / 2);
    midpoint as f64 / 1000.0
}

fn make_export_state(
    job: &Job,
    video: &VideoItem,
    status: &str,
    step: &str,
    output_path: Option<String>,
    encode_summary: Option<PersistedEncodeSummary>,
) -> VideoProcessingState {
    let mut state = make_video_state(&video.video_id, status, step, &["review-complete".to_string()]);
    state.audio_replacement_applied = Some(should_replace_audio(job, video));
    state.audio_source_path = video.mapped_audio_path.clone();
    state.output_path = output_path;
    state.encode_summary = encode_summary;
    state
}

fn build_encode_summary(
    app_handle: &AppHandle,
    output_path: &str,
    preset: Option<&Preset>,
) -> Result<PersistedEncodeSummary, String> {
    let metadata = AnalysisService::probe_metadata(app_handle, output_path)?;
    let file_size = fs::metadata(output_path)
        .map_err(|error| format!("Khong doc duoc export metadata `{output_path}`: {error}"))?
        .len();
    let output_size_mb = file_size as f64 / (1024.0 * 1024.0);
    let bitrate_kbps = if metadata.duration_seconds > 0.0 {
        Some(round2((file_size as f64 * 8.0 / metadata.duration_seconds) / 1000.0))
    } else {
        None
    };

    Ok(PersistedEncodeSummary {
        codec: "H.264/AAC".to_string(),
        crf: preset.map(|value| parse_crf(&value.export_preset)).unwrap_or_else(|| "20".to_string()),
        output_size_mb: round2(output_size_mb),
        duration_seconds: round2(metadata.duration_seconds),
        bitrate_kbps,
    })
}

fn load_export_video_config(job: &Job, video_id: &str) -> Result<ExportVideoConfig, String> {
    let segments_dir = PathBuf::from(&job.output_folder).join(SEGMENT_STATE_DIR);
    let safe_id = sanitize_for_path(video_id);
    let logo_path = segments_dir.join(format!("{safe_id}_logo.json"));
    let subtitle_path = segments_dir.join(format!("{safe_id}_subtitle.json"));

    let mut logo_box = None;
    if logo_path.exists() {
        let payload = fs::read_to_string(&logo_path)
            .map_err(|error| format!("Khong doc duoc logo segment state `{}`: {error}", logo_path.display()))?;
        let parsed: PersistedLogoReview = serde_json::from_str(&payload)
            .map_err(|error| format!("Khong parse duoc logo segment state `{}`: {error}", logo_path.display()))?;
        if let Some(segment) = parsed.segments.first() {
            logo_box = Some(
                segment
                    .quick_fix_state
                    .as_ref()
                    .and_then(|state| state.logo_fix.as_ref())
                    .map(|fix| BoundingBox {
                        x: fix.x,
                        y: fix.y,
                        width: fix.width,
                        height: fix.height,
                    })
                    .unwrap_or_else(|| segment.bounding_box.clone()),
            );
        }
    }

    let mut subtitle_region = None;
    let mut subtitle_fix = None;
    let mut subtitle_confidence = 0.8f32;
    if subtitle_path.exists() {
        let payload = fs::read_to_string(&subtitle_path)
            .map_err(|error| format!("Khong doc duoc subtitle segment state `{}`: {error}", subtitle_path.display()))?;
        let parsed: PersistedSubtitleReview = serde_json::from_str(&payload)
            .map_err(|error| format!("Khong parse duoc subtitle segment state `{}`: {error}", subtitle_path.display()))?;
        if let Some(segment) = parsed.segments.first() {
            subtitle_confidence = segment.confidence;
            let effective_fix = segment.quick_fix_state.as_ref().and_then(|state| state.subtitle_fix.clone());
            subtitle_region = effective_fix
                .as_ref()
                .and_then(|fix| fix.old_region.clone())
                .or_else(|| {
                    Some(PersistedSubtitleRegionFix {
                        x: segment.region.x,
                        y: segment.region.y,
                        width: segment.region.width,
                        height: segment.region.height,
                        mode: "blur".to_string(),
                    })
                });
            subtitle_fix = effective_fix;
        }
    }

    Ok(ExportVideoConfig {
        logo_box,
        subtitle_region,
        subtitle_fix,
        subtitle_confidence,
    })
}

fn resolve_initial_export_input(job: &Job, video: &VideoItem, config: &ExportVideoConfig) -> String {
    let safe_id = sanitize_for_path(&video.video_id);
    let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);

    if config.logo_box.is_some() || config.subtitle_region.is_some() {
        return working_dir
            .join(format!("{safe_id}_audio_replaced.mp4"))
            .exists()
            .then_some(working_dir.join(format!("{safe_id}_audio_replaced.mp4")))
            .unwrap_or_else(|| PathBuf::from(&video.source_path))
            .display()
            .to_string();
    }

    let candidates = [
        working_dir.join(format!("{safe_id}_subtitle_rendered.mp4")),
        working_dir.join(format!("{safe_id}_logo_replaced.mp4")),
        working_dir.join(format!("{safe_id}_audio_replaced.mp4")),
        PathBuf::from(&video.source_path),
    ];

    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from(&video.source_path))
        .display()
        .to_string()
}

fn transcode_with_progress(
    app_handle: &AppHandle,
    job: &Job,
    video: &VideoItem,
    input_path: &str,
    output_path: &Path,
    crf: &str,
) -> Result<(), String> {
    let ffmpeg_path = resolve_ffmpeg_path(app_handle)
        .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Khong tao duoc export output dir `{}`: {error}", parent.display()))?;
    }

    let metadata = AnalysisService::probe_metadata(app_handle, input_path)?;
    let total_microseconds = (metadata.duration_seconds.max(0.1) * 1_000_000.0) as u64;
    let args = vec![
        "-v".to_string(),
        "error".to_string(),
        "-i".to_string(),
        input_path.to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "slow".to_string(),
        "-crf".to_string(),
        crf.to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        "-y".to_string(),
        output_path.display().to_string(),
    ];

    LoggingService::append_video_log(
        job,
        video,
        &format!(
            "export-start: input={}; output={}; command={} {}",
            input_path,
            output_path.display(),
            ffmpeg_path.display(),
            args.join(" ")
        ),
    )?;

    let mut child = Command::new(&ffmpeg_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Khong the chay FFmpeg export: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Khong lay duoc stdout cua FFmpeg export".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Khong lay duoc stderr cua FFmpeg export".to_string())?;

    emit_export_progress(app_handle, &video.video_id, 0)?;
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line.map_err(|error| format!("Khong doc duoc FFmpeg progress line: {error}"))?;
        if let Some(value) = line.strip_prefix("out_time_ms=") {
            if let Ok(out_time_ms) = value.trim().parse::<u64>() {
                let percent = ((out_time_ms as f64 / total_microseconds as f64) * 100.0)
                    .round()
                    .clamp(0.0, 99.0) as u8;
                emit_export_progress(app_handle, &video.video_id, percent)?;
            }
        } else if line.trim() == "progress=end" {
            emit_export_progress(app_handle, &video.video_id, 100)?;
        }
    }

    let mut stderr_output = String::new();
    let _ = stderr.read_to_string(&mut stderr_output);
    let status = child
        .wait()
        .map_err(|error| format!("Khong doi duoc FFmpeg export finish: {error}"))?;

    if !status.success() {
        let stderr_snippet = stderr_output.chars().take(400).collect::<String>();
        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "export-failed: exitCode={:?}; stderr={}",
                status.code(),
                stderr_snippet
            ),
        )?;
        return Err(format!(
            "FFmpeg export that bai (exit {:?}): {}",
            status.code(),
            if stderr_snippet.is_empty() {
                "Khong co stderr"
            } else {
                stderr_snippet.as_str()
            }
        ));
    }

    LoggingService::append_video_log(
        job,
        video,
        &format!("export-success: output={}", output_path.display()),
    )?;
    Ok(())
}

fn blocked_reason(status: &str) -> String {
    match status {
        "ReviewNeeded" => "Con doan High Risk chua xu ly".to_string(),
        "Failed" => "Video bi loi trong qua trinh xu ly".to_string(),
        "Processing" => "Video dang duoc xu ly".to_string(),
        "Queued" => "Video dang cho xu ly".to_string(),
        "Imported" | "Matched" => "Video chua qua review gating".to_string(),
        "Exporting" => "Video dang duoc export".to_string(),
        "Exported" => "Video da export xong".to_string(),
        "ExportedWithFailures" => "Batch export co video that bai".to_string(),
        "ExportFailed" => "Batch export that bai".to_string(),
        other => format!("Video chua san sang export ({other})"),
    }
}

fn effective_output_folder(job: &Job) -> String {
    job.export_output_folder
        .clone()
        .unwrap_or_else(|| PathBuf::from(&job.output_folder).join("output").display().to_string())
}

fn audio_summary_for_video(job: &Job, preset: Option<&Preset>, mapped_audio_path: Option<&str>) -> String {
    let should_replace_audio = matches!(job.selected_task.as_deref(), Some("replace-audio" | "replace-all"));
    let policy_allows_audio = preset
        .map(|value| AudioPolicyService::should_replace_audio(&value.audio_replacement_policy))
        .unwrap_or(should_replace_audio);

    if !should_replace_audio || !policy_allows_audio {
        return "Audio: Giu nguyen".to_string();
    }

    match mapped_audio_path {
        Some(path) => {
            let audio_path = PathBuf::from(path);
            let filename = audio_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(path);
            format!("Audio: {filename}")
        }
        None => "Audio: Chua co mapping".to_string(),
    }
}

fn preset_summary(preset: &Preset) -> ExportPresetSummary {
    ExportPresetSummary {
        label: preset.export_preset.clone(),
        codec: "H.264".to_string(),
        crf: parse_crf(&preset.export_preset),
    }
}

fn parse_crf(value: &str) -> String {
    if value.contains("CRF18") || value.contains("CRF 18") {
        "18".to_string()
    } else if value.contains("CRF23") || value.contains("CRF 23") {
        "23".to_string()
    } else if value.contains("CRF20") || value.contains("CRF 20") {
        "20".to_string()
    } else {
        "20".to_string()
    }
}

fn normalize_folder_path(value: &str) -> String {
    value.replace('\\', "/").trim_end_matches('/').to_ascii_lowercase()
}

fn video_name(source_path: &str) -> String {
    PathBuf::from(source_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(source_path)
        .to_string()
}

fn final_output_path(output_folder: &str, video: &VideoItem) -> PathBuf {
    let stem = Path::new(&video.source_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    PathBuf::from(output_folder).join(format!(
        "{}_{}_rebranded.mp4",
        sanitize_for_path(&video.video_id),
        sanitize_for_path(stem)
    ))
}

fn subtitle_force_style_from_fix(
    preset: Option<&Preset>,
    subtitle_fix: &PersistedSubtitleFix,
    metadata: &VideoMetadata,
) -> String {
    let mut style = preset
        .map(|value| value.subtitle_style_preset.clone())
        .filter(|value| value.contains("FontName=") || value.contains("Fontname="))
        .unwrap_or_else(|| {
            "FontName=Arial,FontSize=24,PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,Outline=1".to_string()
        });

    if let Some(custom_style) = &subtitle_fix.style_preset {
        if !custom_style.trim().is_empty() && custom_style.contains("FontName=") {
            style = custom_style.clone();
        }
    }

    let scale = subtitle_fix.new_scale.unwrap_or(1.0).clamp(0.4, 3.0);
    let base_font_size = (24.0 * scale).round() as u32;
    let (alignment, margin_l, margin_r, margin_v) = if let Some(position) = &subtitle_fix.new_position {
        let third = metadata.width / 3;
        if position.x <= third {
            (
                1,
                position.x,
                0,
                metadata.height.saturating_sub(position.y).max(16),
            )
        } else if position.x >= third * 2 {
            (
                3,
                0,
                metadata.width.saturating_sub(position.x).max(16),
                metadata.height.saturating_sub(position.y).max(16),
            )
        } else {
            (
                2,
                0,
                0,
                metadata.height.saturating_sub(position.y).max(16),
            )
        }
    } else {
        (2, 0, 0, ((metadata.height as f64) * 0.08) as u32)
    };

    format!(
        "{style},Alignment={alignment},MarginL={margin_l},MarginR={margin_r},MarginV={margin_v},FontSize={base_font_size}"
    )
}

fn resolve_logo_source(video: &VideoItem, preset: Option<&Preset>) -> Result<String, String> {
    video
        .mapped_logo_path
        .clone()
        .or_else(|| preset.map(|value| value.default_logo_path.clone()))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Video `{}` chua co logo", video.video_id))
}

fn load_preset(app_handle: &AppHandle, preset_id: Option<&str>) -> Result<Option<Preset>, String> {
    match preset_id {
        Some(value) => PresetService::get_preset(app_handle, value).map(Some),
        None => Ok(None),
    }
}

fn find_video<'a>(job: &'a Job, video_id: &str) -> Result<&'a VideoItem, String> {
    job.video_items
        .iter()
        .find(|item| item.video_id == video_id)
        .ok_or_else(|| format!("Khong tim thay video `{video_id}` trong job"))
}

fn load_video_state(job: &Job, video_id: &str) -> Result<VideoProcessingState, String> {
    let path = PathBuf::from(&job.output_folder)
        .join(VIDEO_STATE_DIR)
        .join(format!("{}.json", sanitize_for_path(video_id)));
    let payload = fs::read_to_string(&path)
        .map_err(|error| format!("Khong doc duoc video state `{}`: {error}", path.display()))?;
    serde_json::from_str(&payload)
        .map_err(|error| format!("Khong parse duoc video state `{}`: {error}", path.display()))
}

fn video_report_path(job: &Job, video_id: &str) -> PathBuf {
    PathBuf::from(&job.output_folder)
        .join(REPORTS_DIR)
        .join(format!("{}-report.json", sanitize_for_path(video_id)))
}

fn job_summary_report_path(job: &Job) -> PathBuf {
    PathBuf::from(&job.output_folder)
        .join(REPORTS_DIR)
        .join("export-summary.json")
}

fn reports_thumbnail_dir(job: &Job) -> PathBuf {
    PathBuf::from(&job.output_folder).join(REPORTS_DIR).join("thumbnails")
}

fn persist_video_report(job: &Job, report: &VideoReport) -> Result<(), String> {
    let path = video_report_path(job, &report.video_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("Khong tao duoc report dir: {error}"))?;
    }
    let payload = serde_json::to_string_pretty(report)
        .map_err(|error| format!("Khong serialize duoc video report: {error}"))?;
    fs::write(&path, payload).map_err(|error| format!("Khong ghi duoc video report `{}`: {error}", path.display()))
}

fn persist_job_summary_report(job: &Job, report: &JobExportSummaryReport) -> Result<(), String> {
    let path = job_summary_report_path(job);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("Khong tao duoc report dir: {error}"))?;
    }
    let payload = serde_json::to_string_pretty(report)
        .map_err(|error| format!("Khong serialize duoc job export summary: {error}"))?;
    fs::write(&path, payload)
        .map_err(|error| format!("Khong ghi duoc job export summary `{}`: {error}", path.display()))
}

fn read_video_report(path: &Path) -> Result<VideoReport, String> {
    let payload = fs::read_to_string(path)
        .map_err(|error| format!("Khong doc duoc video report `{}`: {error}", path.display()))?;
    serde_json::from_str(&payload).map_err(|error| format!("Khong parse duoc video report `{}`: {error}", path.display()))
}

fn read_job_summary_report(path: &Path) -> Result<JobExportSummaryReport, String> {
    let payload = fs::read_to_string(path)
        .map_err(|error| format!("Khong doc duoc job export summary `{}`: {error}", path.display()))?;
    serde_json::from_str(&payload)
        .map_err(|error| format!("Khong parse duoc job export summary `{}`: {error}", path.display()))
}

fn emit_job_updated(app_handle: &AppHandle, job: Job) -> Result<(), String> {
    app_handle
        .emit("jobUpdated", serde_json::json!({ "job": job }))
        .map_err(|error| format!("Khong emit duoc jobUpdated: {error}"))
}

fn emit_video_export_started(app_handle: &AppHandle, video_id: &str) -> Result<(), String> {
    app_handle
        .emit(
            "videoExportStarted",
            VideoExportStartedEvent {
                video_id: video_id.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc videoExportStarted: {error}"))
}

fn emit_export_progress(app_handle: &AppHandle, video_id: &str, percent: u8) -> Result<(), String> {
    app_handle
        .emit(
            "exportProgress",
            ExportProgressEvent {
                video_id: video_id.to_string(),
                percent,
            },
        )
        .map_err(|error| format!("Khong emit duoc exportProgress: {error}"))
}

fn emit_video_export_completed(
    app_handle: &AppHandle,
    video_id: &str,
    success: bool,
    output_path: Option<String>,
    error_message: Option<String>,
) -> Result<(), String> {
    app_handle
        .emit(
            "videoExportCompleted",
            VideoExportCompletedEvent {
                video_id: video_id.to_string(),
                success,
                output_path,
                error_message,
            },
        )
        .map_err(|error| format!("Khong emit duoc videoExportCompleted: {error}"))
}

fn emit_batch_export_completed(
    app_handle: &AppHandle,
    job_id: &str,
    total: usize,
    success: usize,
    failed: usize,
) -> Result<(), String> {
    app_handle
        .emit(
            "batchExportCompleted",
            BatchExportCompletedEvent {
                job_id: job_id.to_string(),
                total,
                success,
                failed,
            },
        )
        .map_err(|error| format!("Khong emit duoc batchExportCompleted: {error}"))
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn should_replace_audio(job: &Job, video: &VideoItem) -> bool {
    matches!(job.selected_task.as_deref(), Some("replace-audio" | "replace-all")) && video.mapped_audio_path.is_some()
}

fn audio_policy_label(job: &Job, video: &VideoItem) -> String {
    if should_replace_audio(job, video) {
        "replace".to_string()
    } else {
        "keep".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_output_path_sanitizes_video_id_and_stem() {
        let video = VideoItem {
            video_id: "video/1".to_string(),
            source_path: "D:/clips/My Demo.mp4".to_string(),
            source_metadata: None,
            mapped_logo_path: None,
            mapped_audio_path: None,
            mapped_srt_path: None,
            status: "ReadyToExport".to_string(),
        };

        let output = final_output_path("D:/exports", &video);
        assert_eq!(output, PathBuf::from("D:/exports").join("video_1_My_Demo_rebranded.mp4"));
    }

    #[test]
    fn export_readiness_blocks_exported_video() {
        assert_eq!(blocked_reason("Exported"), "Video da export xong");
        assert_eq!(blocked_reason("Exporting"), "Video dang duoc export");
    }

    #[test]
    fn subtitle_force_style_uses_position_and_scale() {
        let fix = PersistedSubtitleFix {
            old_region: None,
            new_position: Some(PersistedSubtitlePositionFix { x: 100, y: 900 }),
            new_scale: Some(1.5),
            style_preset: Some("FontName=Roboto".to_string()),
        };
        let metadata = VideoMetadata {
            width: 1920,
            height: 1080,
            duration_seconds: 20.0,
        };

        let style = subtitle_force_style_from_fix(None, &fix, &metadata);
        assert!(style.contains("FontName=Roboto"));
        assert!(style.contains("Alignment=1"));
        assert!(style.contains("FontSize=36"));
    }

    #[test]
    fn segment_stats_counts_modified_and_remaining_high_risk() {
        let stats = build_segment_stats(&[
            ReportSegmentCandidate {
                segment_id: "a".to_string(),
                issue_type: "LogoPosition".to_string(),
                risk_level: "High".to_string(),
                review_status: "Modified".to_string(),
                start_ms: 0,
                end_ms: Some(1000),
            },
            ReportSegmentCandidate {
                segment_id: "b".to_string(),
                issue_type: "SubtitleRegion".to_string(),
                risk_level: "High".to_string(),
                review_status: "Unreviewed".to_string(),
                start_ms: 1000,
                end_ms: Some(2000),
            },
            ReportSegmentCandidate {
                segment_id: "c".to_string(),
                issue_type: "SubtitleStyle".to_string(),
                risk_level: "Medium".to_string(),
                review_status: "Accepted".to_string(),
                start_ms: 2000,
                end_ms: Some(3000),
            },
        ]);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.flagged, 3);
        assert_eq!(stats.modified, 1);
        assert_eq!(stats.accepted, 1);
        assert_eq!(stats.high_risk_remaining, 1);
    }

    #[test]
    fn spot_check_selection_prioritizes_high_modified_then_high_accepted() {
        let selected = select_spot_check_segments(&[
            ReportSegmentCandidate {
                segment_id: "m1".to_string(),
                issue_type: "LogoPosition".to_string(),
                risk_level: "Medium".to_string(),
                review_status: "Modified".to_string(),
                start_ms: 3000,
                end_ms: Some(4000),
            },
            ReportSegmentCandidate {
                segment_id: "h1".to_string(),
                issue_type: "LogoPosition".to_string(),
                risk_level: "High".to_string(),
                review_status: "Accepted".to_string(),
                start_ms: 1000,
                end_ms: Some(2000),
            },
            ReportSegmentCandidate {
                segment_id: "h0".to_string(),
                issue_type: "SubtitleRegion".to_string(),
                risk_level: "High".to_string(),
                review_status: "Modified".to_string(),
                start_ms: 0,
                end_ms: Some(1000),
            },
        ]);

        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].segment_id, "h0");
        assert_eq!(selected[1].segment_id, "h1");
        assert_eq!(selected[2].segment_id, "m1");
    }

    // ─── Integration: filesystem round-trips ──────────────────────────────────

    fn make_test_job(root: &std::path::Path) -> Job {
        Job {
            job_id: "job-1".to_string(),
            created_at: "2026-04-22T00:00:00Z".to_string(),
            selected_task: Some("replace-all".to_string()),
            preset_id: None,
            output_folder: root.display().to_string(),
            export_output_folder: None,
            status: "ReadyToExport".to_string(),
            video_items: vec![],
            imported_files: vec![],
        }
    }

    #[test]
    fn load_export_video_config_uses_quick_fix_logo_bbox_when_present() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let segments_dir = temp_dir.path().join("segments");
        fs::create_dir_all(&segments_dir).expect("create segments dir");
        fs::write(
            segments_dir.join("video-1_logo.json"),
            r#"{"videoId":"video-1","segments":[{"issueType":"LogoPosition","riskLevel":"High","startMs":0,"endMs":1000,"boundingBox":{"x":10,"y":20,"width":100,"height":50},"reviewStatus":"Modified","quickFixState":{"logoFix":{"x":55,"y":66,"width":120,"height":60},"subtitleFix":null}}]}"#,
        ).expect("write logo segment");

        let config = load_export_video_config(&job, "video-1").expect("load config");

        let logo_box = config.logo_box.expect("logo_box present");
        assert_eq!(logo_box.x, 55);
        assert_eq!(logo_box.y, 66);
        assert_eq!(logo_box.width, 120);
        assert_eq!(logo_box.height, 60);
    }

    #[test]
    fn load_export_video_config_falls_back_to_segment_bbox_when_no_fix() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let segments_dir = temp_dir.path().join("segments");
        fs::create_dir_all(&segments_dir).expect("create segments dir");
        fs::write(
            segments_dir.join("video-1_logo.json"),
            r#"{"videoId":"video-1","segments":[{"issueType":"LogoPosition","riskLevel":"Medium","startMs":500,"endMs":2000,"boundingBox":{"x":10,"y":20,"width":100,"height":50},"reviewStatus":null,"quickFixState":null}]}"#,
        ).expect("write logo segment");

        let config = load_export_video_config(&job, "video-1").expect("load config");

        let logo_box = config.logo_box.expect("logo_box present");
        assert_eq!(logo_box.x, 10);
        assert_eq!(logo_box.y, 20);
        assert_eq!(logo_box.width, 100);
        assert_eq!(logo_box.height, 50);
    }

    #[test]
    fn load_export_video_config_extracts_subtitle_region_from_segment() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let segments_dir = temp_dir.path().join("segments");
        fs::create_dir_all(&segments_dir).expect("create segments dir");
        fs::write(
            segments_dir.join("video-1_subtitle.json"),
            r#"{"videoId":"video-1","segments":[{"issueType":"SubtitleRegion","riskLevel":"Medium","startMs":0,"endMs":3000,"confidence":0.9,"region":{"x":0,"y":960,"width":1920,"height":80},"reviewStatus":null,"quickFixState":null}]}"#,
        ).expect("write subtitle segment");

        let config = load_export_video_config(&job, "video-1").expect("load config");

        let region = config.subtitle_region.expect("subtitle_region present");
        assert_eq!(region.x, 0);
        assert_eq!(region.y, 960);
        assert_eq!(region.width, 1920);
        assert_eq!(region.height, 80);
        assert_eq!(region.mode, "blur");
        assert!((config.subtitle_confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn load_export_video_config_uses_subtitle_quick_fix_region() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let segments_dir = temp_dir.path().join("segments");
        fs::create_dir_all(&segments_dir).expect("create segments dir");
        fs::write(
            segments_dir.join("video-1_subtitle.json"),
            r#"{"videoId":"video-1","segments":[{"issueType":"SubtitleRegion","riskLevel":"High","startMs":0,"endMs":5000,"confidence":0.7,"region":{"x":0,"y":950,"width":1920,"height":80},"reviewStatus":"Modified","quickFixState":{"logoFix":null,"subtitleFix":{"oldRegion":{"x":0,"y":950,"width":1920,"height":80,"mode":"inpaint"},"newPosition":null,"newScale":1.2,"stylePreset":null}}}]}"#,
        ).expect("write subtitle segment");

        let config = load_export_video_config(&job, "video-1").expect("load config");

        let region = config.subtitle_region.expect("subtitle_region from quick fix");
        assert_eq!(region.mode, "inpaint");
        assert_eq!(region.y, 950);
        let fix = config.subtitle_fix.expect("subtitle_fix present");
        assert!((fix.new_scale.unwrap_or(0.0) - 1.2).abs() < 0.01);
    }

    #[test]
    fn load_report_segments_combines_logo_and_subtitle_segments() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let segments_dir = temp_dir.path().join("segments");
        fs::create_dir_all(&segments_dir).expect("create segments dir");
        fs::write(
            segments_dir.join("video-1_logo.json"),
            r#"{"videoId":"video-1","segments":[{"issueType":"LogoPosition","riskLevel":"High","startMs":0,"endMs":1000,"boundingBox":{"x":10,"y":20,"width":100,"height":50}}]}"#,
        ).expect("write logo segment");
        fs::write(
            segments_dir.join("video-1_subtitle.json"),
            r#"{"videoId":"video-1","segments":[{"issueType":"SubtitleRegion","riskLevel":"Medium","startMs":2000,"endMs":4000,"confidence":0.85,"region":{"x":0,"y":900,"width":1920,"height":80}}]}"#,
        ).expect("write subtitle segment");

        let segments = load_report_segments(&job, "video-1").expect("load segments");

        assert_eq!(segments.len(), 2);
        let issue_types: Vec<&str> = segments.iter().map(|s| s.issue_type.as_str()).collect();
        assert!(issue_types.contains(&"LogoPosition"));
        assert!(issue_types.contains(&"SubtitleRegion"));
    }

    #[test]
    fn load_report_segments_returns_empty_when_no_files_exist() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());

        let segments = load_report_segments(&job, "video-1").expect("load segments");

        assert!(segments.is_empty());
    }

    #[test]
    fn video_report_persists_and_reads_back_correctly() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let report = VideoReport {
            video_id: "video-1".to_string(),
            video_name: "clip.mp4".to_string(),
            final_status: "Exported".to_string(),
            encode_summary: Some(PersistedEncodeSummary {
                codec: "H.264/AAC".to_string(),
                crf: "20".to_string(),
                output_size_mb: 45.5,
                duration_seconds: 120.0,
                bitrate_kbps: Some(3000.0),
            }),
            audio_source: AudioSourceSummary {
                policy: "replace".to_string(),
                audio_file_path: Some("D:/audio/clip.mp3".to_string()),
            },
            segment_stats: SegmentStats { total: 2, flagged: 1, modified: 1, accepted: 0, high_risk_remaining: 0 },
            spot_check_thumbnails: vec![],
            output_path: Some("D:/exports/clip_rebranded.mp4".to_string()),
            report_generated_at: "2026-04-22T00:00:00Z".to_string(),
        };

        persist_video_report(&job, &report).expect("persist report");

        let path = video_report_path(&job, "video-1");
        let loaded = read_video_report(&path).expect("read report");
        assert_eq!(loaded.video_id, "video-1");
        assert_eq!(loaded.final_status, "Exported");
        assert_eq!(loaded.segment_stats.total, 2);
        assert_eq!(loaded.segment_stats.modified, 1);
        assert_eq!(loaded.encode_summary.unwrap().crf, "20");
        assert_eq!(loaded.output_path.unwrap(), "D:/exports/clip_rebranded.mp4");
    }

    #[test]
    fn job_summary_report_persists_and_reads_back_correctly() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = make_test_job(temp_dir.path());
        let summary = JobExportSummaryReport {
            job_id: "job-1".to_string(),
            total_videos: 3,
            success: 2,
            failed: 1,
            total_output_size_mb: 123.45,
            reports: vec![],
            generated_at: "2026-04-22T00:00:00Z".to_string(),
        };

        persist_job_summary_report(&job, &summary).expect("persist summary");

        let path = job_summary_report_path(&job);
        let loaded = read_job_summary_report(&path).expect("read summary");
        assert_eq!(loaded.job_id, "job-1");
        assert_eq!(loaded.total_videos, 3);
        assert_eq!(loaded.success, 2);
        assert_eq!(loaded.failed, 1);
        assert!((loaded.total_output_size_mb - 123.45).abs() < 0.01);
    }
}
