use std::fs;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

use crate::{
    constants::JOB_MANIFEST,
    domain::{job::Job, preset::Preset},
    services::{
        analysis_service::AnalysisService,
        audio_policy_service::AudioPolicyService,
        audio_replacement_service::AudioReplacementService,
        logging_service::LoggingService,
        persistence_service::{make_video_state, PersistedJobState, PersistenceService, VideoProcessingState},
        preset_service::PresetService,
        render_service::RenderService,
        risk_service::RiskService,
    },
};

// ─── Public domain types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VideoReadiness {
    pub video_id: String,
    pub is_ready: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobReadiness {
    pub is_ready: bool,
    pub blockers: Vec<String>,
    pub videos: Vec<VideoReadiness>,
}

// ─── Event payloads ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStartedEvent {
    pub job: Job,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobUpdatedEvent {
    pub job: Job,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProcessingEvent {
    pub job_id: String,
    pub video_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioReplacementStartedEvent {
    pub video_id: String,
    pub audio_source_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioReplacementCompletedEvent {
    pub video_id: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoDetectionCompletedEvent {
    pub video_id: String,
    pub confidence: f32,
    pub segment_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoReplacementCompletedEvent {
    pub video_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleDetectionCompletedEvent {
    pub video_id: String,
    pub region_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleRenderCompletedEvent {
    pub video_id: String,
}

/// Emitted before/after each pipeline step so the UI log panel can stream progress.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingStepUpdateEvent {
    pub video_id: String,
    pub step: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskDistribution {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Final per-video outcome emitted when all pipeline steps finish for a video.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProcessingCompletedEvent {
    pub video_id: String,
    pub outcome: String,
    pub segment_count: usize,
    pub risk_distribution: RiskDistribution,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub total: usize,
    pub review_needed: usize,
    pub ready_to_export: usize,
    pub failed: usize,
}

/// Emitted once after the full job queue finishes processing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProcessingCompletedEvent {
    pub job_id: String,
    pub summary: JobSummary,
}

// ─── Internal result type ─────────────────────────────────────────────────────

struct VideoProcessingOutcome {
    status: String,
    total_segments: usize,
    high_risk_count: usize,
    medium_risk_count: usize,
}

// ─── Orchestrator ─────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct JobOrchestrator;

impl JobOrchestrator {
    pub fn validate_readiness(app_handle: &AppHandle, job_id: &str) -> Result<JobReadiness, String> {
        let job = load_job(app_handle, job_id)?;
        let preset = load_preset(app_handle, job.preset_id.as_deref())?;
        Ok(validate_job_readiness(&job, preset.as_ref()))
    }

    pub fn start_job(app_handle: &AppHandle, job_id: &str) -> Result<(Job, JobReadiness, bool), String> {
        let mut job = load_job(app_handle, job_id)?;

        if job.status == "Processing" {
            let readiness = Self::validate_readiness(app_handle, job_id)?;
            return Ok((job, readiness, false));
        }

        if !matches!(job.status.as_str(), "Draft" | "ReadyToRun") {
            return Err(format!("Khong the bat dau job o trang thai `{}`", job.status));
        }

        let readiness = Self::validate_readiness(app_handle, job_id)?;
        if !readiness.is_ready {
            return Ok((job, readiness, false));
        }

        job.status = "Processing".to_string();
        for video in &mut job.video_items {
            if should_queue_video(video.status.as_str()) {
                video.status = "Queued".to_string();
            }
        }

        PersistenceService::persist_job(app_handle, &job)?;
        emit_job_started(app_handle, job.clone())?;
        emit_job_updated(app_handle, job.clone())?;

        let runner_handle = app_handle.clone();
        let runner_job_id = job.job_id.clone();
        tauri::async_runtime::spawn(async move {
            let _ = process_job_queue(runner_handle, runner_job_id).await;
        });

        Ok((job, readiness, true))
    }

    pub fn restore_from_checkpoint(
        app_handle: &AppHandle,
        mut state: PersistedJobState,
    ) -> Result<(PersistedJobState, Option<String>), String> {
        let warning_message = if state.job.status == "Processing" {
            state.job.status = "ReadyToRun".to_string();
            for video in &mut state.job.video_items {
                if matches!(video.status.as_str(), "Processing" | "Queued") {
                    video.status = "Imported".to_string();
                }
            }
            normalize_processing_states(&state.job, &mut state.video_states);
            PersistenceService::persist_job(app_handle, &state.job)?;
            persist_video_states(&state.job, &state.video_states)?;
            Some("Job bi gian doan khi dang xu ly, reset ve trang thai san sang chay".to_string())
        } else if state.job.status == "Exporting" {
            state.job.status = "ReadyToExport".to_string();
            for video in &mut state.job.video_items {
                if video.status == "Exporting" {
                    video.status = "ReadyToExport".to_string();
                }
            }
            normalize_exporting_states(&mut state.video_states);
            PersistenceService::persist_job(app_handle, &state.job)?;
            persist_video_states(&state.job, &state.video_states)?;
            Some("Job bi gian doan khi dang export, reset cac video dang xuat ve ReadyToExport".to_string())
        } else {
            None
        };

        Ok((state, warning_message))
    }
}

// ─── Job loading ──────────────────────────────────────────────────────────────

pub fn load_job(app_handle: &AppHandle, job_id: &str) -> Result<Job, String> {
    let job_dir = PersistenceService::job_directory_path(app_handle, job_id)?;
    let manifest_path = job_dir.join(JOB_MANIFEST);
    let payload = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Khong doc duoc job manifest: {error}"))?;

    serde_json::from_str(&payload).map_err(|error| format!("Khong parse duoc job manifest: {error}"))
}

fn load_preset(app_handle: &AppHandle, preset_id: Option<&str>) -> Result<Option<Preset>, String> {
    match preset_id {
        Some(value) => PresetService::get_preset(app_handle, value).map(Some),
        None => Ok(None),
    }
}

fn normalize_processing_states(job: &Job, states: &mut Vec<VideoProcessingState>) {
    for video in &job.video_items {
        if let Some(state) = states.iter_mut().find(|state| state.video_id == video.video_id) {
            if matches!(state.status.as_str(), "Processing" | "Queued") {
                *state = make_video_state(&video.video_id, "Imported", "ready-to-run", &Vec::new());
            }
        } else {
            states.push(make_video_state(&video.video_id, "Imported", "ready-to-run", &Vec::new()));
        }
    }
}

fn normalize_exporting_states(states: &mut [VideoProcessingState]) {
    for state in states.iter_mut() {
        if state.status == "Exporting" {
            *state = make_video_state(
                &state.video_id,
                "ReadyToExport",
                "ready-to-export",
                &state.completed_steps,
            );
        }
    }
}

fn should_queue_video(status: &str) -> bool {
    !matches!(status, "ReviewNeeded" | "ReadyToExport" | "Failed")
}

fn persist_video_states(job: &Job, states: &[VideoProcessingState]) -> Result<(), String> {
    for state in states {
        PersistenceService::persist_video_state(job, state)?;
    }
    Ok(())
}

// ─── Readiness validation ─────────────────────────────────────────────────────

fn validate_job_readiness(job: &Job, preset: Option<&Preset>) -> JobReadiness {
    let videos = job
        .video_items
        .iter()
        .map(|video| {
            let mut blockers = Vec::new();

            match job.selected_task.as_deref() {
                Some("replace-logo") => {
                    if !has_logo(video.mapped_logo_path.as_deref(), preset) {
                        blockers.push("Chua co logo".to_string());
                    }
                }
                Some("replace-audio") => {
                    if video.mapped_audio_path.is_none() {
                        blockers.push("Chua co audio".to_string());
                    }
                }
                Some("replace-subtitle") => {
                    if video.mapped_srt_path.is_none() {
                        blockers.push("Chua co SRT".to_string());
                    }
                }
                Some("replace-all") => {
                    if !has_logo(video.mapped_logo_path.as_deref(), preset) {
                        blockers.push("Chua co logo".to_string());
                    }
                    if video.mapped_audio_path.is_none() {
                        blockers.push("Chua co audio".to_string());
                    }
                    if video.mapped_srt_path.is_none() {
                        blockers.push("Chua co SRT".to_string());
                    }
                }
                _ => blockers.push("Chua chon task cho job".to_string()),
            }

            if matches!(video.status.as_str(), "Input Needs Review" | "NeedsReview") {
                blockers.push("Mapping chua duoc xac nhan".to_string());
            }

            VideoReadiness {
                video_id: video.video_id.clone(),
                is_ready: blockers.is_empty(),
                blockers,
            }
        })
        .collect::<Vec<_>>();

    let blockers = if videos.iter().any(|video| !video.is_ready) {
        vec!["Job chua san sang; van con video dang bi block.".to_string()]
    } else {
        Vec::new()
    };

    JobReadiness {
        is_ready: blockers.is_empty(),
        blockers,
        videos,
    }
}

// ─── Processing queue ─────────────────────────────────────────────────────────

async fn process_job_queue(app_handle: AppHandle, job_id: String) -> Result<(), String> {
    let mut job = load_job(&app_handle, &job_id)?;
    let preset = load_preset(&app_handle, job.preset_id.as_deref())?;

    let queued_indexes = job
        .video_items
        .iter()
        .enumerate()
        .filter_map(|(index, video)| {
            if matches!(video.status.as_str(), "Queued" | "Imported") {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let total = queued_indexes.len();
    let mut outcomes: Vec<String> = Vec::new();

    for index in queued_indexes {
        let video_id = job.video_items[index].video_id.clone();
        job.video_items[index].status = "Processing".to_string();
        PersistenceService::persist_job(&app_handle, &job)?;
        emit_job_updated(&app_handle, job.clone())?;
        emit_video_processing_started(&app_handle, &job.job_id, &video_id)?;

        let processing_result = process_video(&app_handle, &job, index, preset.as_ref());

        sleep(Duration::from_millis(150)).await;

        match processing_result {
            Ok(outcome) => {
                let outcome_status = outcome.status.clone();
                job.video_items[index].status = outcome_status.clone();
                outcomes.push(outcome_status.clone());

                emit_video_processing_completed(
                    &app_handle,
                    &video_id,
                    &outcome_status,
                    outcome.total_segments,
                    outcome.high_risk_count,
                    outcome.medium_risk_count,
                )?;
                emit_video_processing_done(&app_handle, &job.job_id, &video_id)?;
            }
            Err(error) => {
                job.video_items[index].status = "Failed".to_string();
                outcomes.push("Failed".to_string());
                let _ = PersistenceService::persist_video_state(
                    &job,
                    &make_video_state(&video_id, "Failed", "failed", &Vec::new()),
                );
                let _ = emit_step_update(&app_handle, &video_id, "pipeline", "failed", &error);
                emit_audio_replacement_completed(&app_handle, &video_id, false, Some(error.clone()))?;
                emit_video_processing_completed(
                    &app_handle,
                    &video_id,
                    "Failed",
                    0,
                    0,
                    0,
                )?;
            }
        }

        PersistenceService::persist_job(&app_handle, &job)?;
        emit_job_updated(&app_handle, job.clone())?;
    }

    // Aggregate job outcome and emit summary.
    let summary = compute_job_summary(&outcomes);
    job.status = determine_job_status(&summary);
    PersistenceService::persist_job(&app_handle, &job)?;
    emit_job_updated(&app_handle, job.clone())?;
    emit_job_processing_completed(&app_handle, &job_id, total, &outcomes)?;

    Ok(())
}

// ─── Per-video processing ─────────────────────────────────────────────────────

fn process_video(
    app_handle: &AppHandle,
    job: &Job,
    index: usize,
    preset: Option<&Preset>,
) -> Result<VideoProcessingOutcome, String> {
    let video = &job.video_items[index];
    let should_replace_audio = matches!(job.selected_task.as_deref(), Some("replace-audio" | "replace-all"));
    let should_replace_logo = matches!(job.selected_task.as_deref(), Some("replace-logo" | "replace-all"));
    let should_replace_subtitle = matches!(job.selected_task.as_deref(), Some("replace-subtitle" | "replace-all"));

    let mut working_input = video.source_path.clone();
    let mut completed_steps: Vec<String> = Vec::new();
    let mut total_segments: usize = 0;
    let mut high_risk_count: usize = 0;
    let mut medium_risk_count: usize = 0;

    // Persist initial state before any work starts.
    let _ = PersistenceService::persist_video_state(
        job,
        &make_video_state(&video.video_id, "Processing", "init", &completed_steps),
    );

    // ─── Audio replacement ────────────────────────────────────────────────────
    if should_replace_audio {
        if let Some(audio_source_path) = video.mapped_audio_path.clone() {
            let policy_allows_audio = preset
                .map(|value| AudioPolicyService::should_replace_audio(&value.audio_replacement_policy))
                .unwrap_or(true);

            emit_step_update(app_handle, &video.video_id, "replace-audio", "started", "Dang thay audio track")?;

            if policy_allows_audio {
                emit_audio_replacement_started(app_handle, &video.video_id, &audio_source_path)?;
            }

            match AudioReplacementService::replace_audio(app_handle, job, preset, video) {
                Ok(result) => {
                    if let Some(output_path) = result.output_path {
                        working_input = output_path;
                    }
                    emit_audio_replacement_completed(app_handle, &video.video_id, true, None)?;
                    completed_steps.push("replace-audio".to_string());
                    let _ = PersistenceService::persist_video_state(
                        job,
                        &make_video_state(&video.video_id, "Processing", "replace-audio", &completed_steps),
                    );
                    emit_step_update(app_handle, &video.video_id, "replace-audio", "completed", "Audio da duoc thay")?;
                }
                Err(error) => return Err(error),
            }
        } else {
            // Missing mapping — log and skip (not a failure).
            emit_step_update(
                app_handle,
                &video.video_id,
                "replace-audio",
                "skipped",
                "Bo qua buoc thay audio: chua co audio mapping",
            )?;
        }
    }

    // ─── Logo detection + overlay ─────────────────────────────────────────────
    if should_replace_logo {
        let logo_source_path = resolve_logo_source(video, preset)?;

        emit_step_update(app_handle, &video.video_id, "detect-logo", "started", "Dang phat hien vung logo")?;
        let detection = AnalysisService::detect_logo(app_handle, job, preset, video, &working_input)?;

        for seg in &detection.segments {
            total_segments += 1;
            match seg.risk_level.as_str() {
                "High" => high_risk_count += 1,
                "Medium" => medium_risk_count += 1,
                _ => {}
            }
        }

        let effective_bbox = if !detection.matched {
            let metadata = AnalysisService::probe_metadata(app_handle, &working_input)?;
            let (logo_width, logo_height) =
                AnalysisService::estimate_logo_size_for_video(&logo_source_path, &metadata);
            AnalysisService::default_bounding_box_from_preset(preset, &metadata, logo_width, logo_height)
        } else {
            detection.bounding_box.clone()
        };

        let segment_count = detection.segments.len();
        let _ = RiskService::persist_logo_segments(job, video, &detection)?;
        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "logo-segments-persisted: segmentCount={segment_count}; riskLevel={}",
                detection.risk_level
            ),
        )?;
        emit_logo_detection_completed(app_handle, &video.video_id, detection.confidence, segment_count)?;
        emit_step_update(
            app_handle,
            &video.video_id,
            "detect-logo",
            "completed",
            &format!("Phat hien logo: confidence={:.2}", detection.confidence),
        )?;

        emit_step_update(app_handle, &video.video_id, "replace-logo", "started", "Dang overlay logo moi")?;
        let overlay_result = RenderService::overlay_logo(
            app_handle,
            job,
            video,
            &working_input,
            &logo_source_path,
            &effective_bbox,
        )?;
        working_input = overlay_result.output_path;
        emit_logo_replacement_completed(app_handle, &video.video_id)?;

        completed_steps.push("replace-logo".to_string());
        let _ = PersistenceService::persist_video_state(
            job,
            &make_video_state(&video.video_id, "Processing", "replace-logo", &completed_steps),
        );
        emit_step_update(app_handle, &video.video_id, "replace-logo", "completed", "Logo da duoc thay")?;
    }

    // ─── Subtitle detection + removal + render ────────────────────────────────
    if should_replace_subtitle {
        let srt_path = video
            .mapped_srt_path
            .clone()
            .ok_or_else(|| format!("Video `{}` chua co SRT mapping", video.video_id))?;

        emit_step_update(app_handle, &video.video_id, "detect-subtitle", "started", "Dang phat hien vung subtitle")?;
        let detection = AnalysisService::detect_subtitle_regions(app_handle, job, video, &working_input)?;

        for seg in &detection.segments {
            total_segments += 1;
            match seg.risk_level.as_str() {
                "High" => high_risk_count += 1,
                "Medium" => medium_risk_count += 1,
                _ => {}
            }
        }

        let region_count = detection.regions.len();
        let _ = RiskService::persist_subtitle_segments(job, video, &detection)?;
        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "subtitle-segments-persisted: segmentCount={}; detected={}",
                detection.segments.len(),
                detection.detected,
            ),
        )?;
        emit_subtitle_detection_completed(app_handle, &video.video_id, region_count)?;
        emit_step_update(
            app_handle,
            &video.video_id,
            "detect-subtitle",
            "completed",
            &format!("Phat hien subtitle: confidence={:.2}", detection.confidence),
        )?;

        if detection.detected && !detection.regions.is_empty() {
            emit_step_update(app_handle, &video.video_id, "remove-subtitle", "started", "Dang xoa subtitle cu")?;
            let removal_result = RenderService::remove_subtitle(
                app_handle,
                job,
                video,
                &working_input,
                &detection.regions[0],
                preset,
            )?;
            working_input = removal_result.output_path;
            emit_step_update(app_handle, &video.video_id, "remove-subtitle", "completed", "Subtitle cu da duoc xoa")?;
        }

        emit_step_update(app_handle, &video.video_id, "render-subtitle", "started", "Dang render subtitle moi")?;
        let render_result = RenderService::render_subtitle(app_handle, job, video, &working_input, &srt_path, preset)?;
        working_input = render_result.output_path;
        emit_subtitle_render_completed(app_handle, &video.video_id)?;

        completed_steps.push("replace-subtitle".to_string());
        let _ = PersistenceService::persist_video_state(
            job,
            &make_video_state(&video.video_id, "Processing", "replace-subtitle", &completed_steps),
        );
        emit_step_update(app_handle, &video.video_id, "render-subtitle", "completed", "Subtitle moi da duoc render")?;
    }

    let _ = &working_input;

    // Determine and persist final outcome.
    let outcome_status = determine_video_outcome(high_risk_count);
    let _ = PersistenceService::persist_video_state(
        job,
        &make_video_state(&video.video_id, outcome_status, "done", &completed_steps),
    );

    Ok(VideoProcessingOutcome {
        status: outcome_status.to_string(),
        total_segments,
        high_risk_count,
        medium_risk_count,
    })
}

// ─── Outcome helpers ──────────────────────────────────────────────────────────

/// Maps high-risk segment count to per-video outcome status.
fn determine_video_outcome(high_risk_count: usize) -> &'static str {
    if high_risk_count > 0 { "ReviewNeeded" } else { "ReadyToExport" }
}

fn compute_job_summary(outcomes: &[String]) -> JobSummary {
    JobSummary {
        total: outcomes.len(),
        review_needed: outcomes.iter().filter(|o| o.as_str() == "ReviewNeeded").count(),
        ready_to_export: outcomes.iter().filter(|o| o.as_str() == "ReadyToExport").count(),
        failed: outcomes.iter().filter(|o| o.as_str() == "Failed").count(),
    }
}

fn determine_job_status(summary: &JobSummary) -> String {
    if summary.review_needed > 0 {
        "ReviewPending".to_string()
    } else if summary.failed > 0 {
        "ProcessedWithFailures".to_string()
    } else {
        "ReadyToExport".to_string()
    }
}

// ─── Emit helpers ─────────────────────────────────────────────────────────────

fn emit_step_update(
    app_handle: &AppHandle,
    video_id: &str,
    step: &str,
    status: &str,
    message: &str,
) -> Result<(), String> {
    app_handle
        .emit(
            "processingStepUpdate",
            ProcessingStepUpdateEvent {
                video_id: video_id.to_string(),
                step: step.to_string(),
                status: status.to_string(),
                message: message.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc processingStepUpdate: {error}"))
}

fn emit_video_processing_completed(
    app_handle: &AppHandle,
    video_id: &str,
    outcome: &str,
    segment_count: usize,
    high: usize,
    medium: usize,
) -> Result<(), String> {
    app_handle
        .emit(
            "videoProcessingCompleted",
            VideoProcessingCompletedEvent {
                video_id: video_id.to_string(),
                outcome: outcome.to_string(),
                segment_count,
                risk_distribution: RiskDistribution {
                    high,
                    medium,
                    low: segment_count.saturating_sub(high + medium),
                },
            },
        )
        .map_err(|error| format!("Khong emit duoc videoProcessingCompleted: {error}"))
}

fn emit_job_processing_completed(
    app_handle: &AppHandle,
    job_id: &str,
    total: usize,
    outcomes: &[String],
) -> Result<(), String> {
    let summary = compute_job_summary(outcomes);
    let _ = total; // already captured in summary.total
    app_handle
        .emit(
            "jobProcessingCompleted",
            JobProcessingCompletedEvent {
                job_id: job_id.to_string(),
                summary,
            },
        )
        .map_err(|error| format!("Khong emit duoc jobProcessingCompleted: {error}"))
}

fn emit_job_started(app_handle: &AppHandle, job: Job) -> Result<(), String> {
    app_handle
        .emit("jobStarted", JobStartedEvent { job })
        .map_err(|error| format!("Khong emit duoc jobStarted: {error}"))
}

fn emit_job_updated(app_handle: &AppHandle, job: Job) -> Result<(), String> {
    app_handle
        .emit("jobUpdated", JobUpdatedEvent { job })
        .map_err(|error| format!("Khong emit duoc jobUpdated: {error}"))
}

fn emit_video_processing_started(app_handle: &AppHandle, job_id: &str, video_id: &str) -> Result<(), String> {
    app_handle
        .emit(
            "videoProcessingStarted",
            VideoProcessingEvent {
                job_id: job_id.to_string(),
                video_id: video_id.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc videoProcessingStarted: {error}"))
}

fn emit_video_processing_done(app_handle: &AppHandle, job_id: &str, video_id: &str) -> Result<(), String> {
    app_handle
        .emit(
            "videoProcessingDone",
            VideoProcessingEvent {
                job_id: job_id.to_string(),
                video_id: video_id.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc videoProcessingDone: {error}"))
}

fn emit_audio_replacement_started(
    app_handle: &AppHandle,
    video_id: &str,
    audio_source_path: &str,
) -> Result<(), String> {
    app_handle
        .emit(
            "audioReplacementStarted",
            AudioReplacementStartedEvent {
                video_id: video_id.to_string(),
                audio_source_path: audio_source_path.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc audioReplacementStarted: {error}"))
}

fn emit_audio_replacement_completed(
    app_handle: &AppHandle,
    video_id: &str,
    success: bool,
    error_message: Option<String>,
) -> Result<(), String> {
    app_handle
        .emit(
            "audioReplacementCompleted",
            AudioReplacementCompletedEvent {
                video_id: video_id.to_string(),
                success,
                error_message,
            },
        )
        .map_err(|error| format!("Khong emit duoc audioReplacementCompleted: {error}"))
}

fn emit_logo_detection_completed(
    app_handle: &AppHandle,
    video_id: &str,
    confidence: f32,
    segment_count: usize,
) -> Result<(), String> {
    app_handle
        .emit(
            "logoDetectionCompleted",
            LogoDetectionCompletedEvent {
                video_id: video_id.to_string(),
                confidence,
                segment_count,
            },
        )
        .map_err(|error| format!("Khong emit duoc logoDetectionCompleted: {error}"))
}

fn emit_logo_replacement_completed(app_handle: &AppHandle, video_id: &str) -> Result<(), String> {
    app_handle
        .emit(
            "logoReplacementCompleted",
            LogoReplacementCompletedEvent {
                video_id: video_id.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc logoReplacementCompleted: {error}"))
}

fn emit_subtitle_detection_completed(
    app_handle: &AppHandle,
    video_id: &str,
    region_count: usize,
) -> Result<(), String> {
    app_handle
        .emit(
            "subtitleDetectionCompleted",
            SubtitleDetectionCompletedEvent {
                video_id: video_id.to_string(),
                region_count,
            },
        )
        .map_err(|error| format!("Khong emit duoc subtitleDetectionCompleted: {error}"))
}

fn emit_subtitle_render_completed(app_handle: &AppHandle, video_id: &str) -> Result<(), String> {
    app_handle
        .emit(
            "subtitleRenderCompleted",
            SubtitleRenderCompletedEvent {
                video_id: video_id.to_string(),
            },
        )
        .map_err(|error| format!("Khong emit duoc subtitleRenderCompleted: {error}"))
}

fn resolve_logo_source(video: &crate::domain::video_item::VideoItem, preset: Option<&Preset>) -> Result<String, String> {
    video
        .mapped_logo_path
        .clone()
        .or_else(|| preset.map(|value| value.default_logo_path.clone()))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Video `{}` chua co logo", video.video_id))
}

fn has_logo(mapped_logo_path: Option<&str>, preset: Option<&Preset>) -> bool {
    mapped_logo_path.is_some() || preset.is_some_and(|value| !value.default_logo_path.is_empty())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::video_item::VideoItem,
        services::mapping_service::{AcceptedFile, FileRole},
    };

    fn sample_job(task: &str) -> Job {
        Job {
            job_id: "job-1".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some(task.to_string()),
            preset_id: Some("preset-1".to_string()),
            output_folder: "D:/jobs/job-1".to_string(),
            export_output_folder: None,
            status: "Draft".to_string(),
            video_items: vec![VideoItem {
                video_id: "video-1".to_string(),
                source_path: "D:/clips/video-1.mp4".to_string(),
                source_metadata: None,
                mapped_logo_path: Some("D:/logos/default.png".to_string()),
                mapped_audio_path: Some("D:/audio/video-1.mp3".to_string()),
                mapped_srt_path: Some("D:/srt/video-1.srt".to_string()),
                status: "Matched".to_string(),
            }],
            imported_files: vec![AcceptedFile {
                file_name: "video-1.mp4".to_string(),
                path: "D:/clips/video-1.mp4".to_string(),
                role: FileRole::Video,
            }],
        }
    }

    fn sample_preset() -> Preset {
        Preset {
            preset_id: "preset-1".to_string(),
            brand_name: "Brand".to_string(),
            default_logo_path: "D:/logos/default.png".to_string(),
            audio_replacement_policy: "policy".to_string(),
            subtitle_style_preset: "style".to_string(),
            layout_rules: "rules".to_string(),
            export_preset: "export".to_string(),
            notes: "notes".to_string(),
        }
    }

    #[test]
    fn marks_job_blocked_when_audio_missing_for_audio_task() {
        let mut job = sample_job("replace-audio");
        job.video_items[0].mapped_audio_path = None;
        job.video_items[0].status = "Input Needs Review".to_string();

        let readiness = validate_job_readiness(&job, Some(&sample_preset()));

        assert!(!readiness.is_ready);
        assert_eq!(readiness.videos[0].blockers[0], "Chua co audio");
    }

    #[test]
    fn marks_job_ready_when_required_inputs_exist() {
        let readiness = validate_job_readiness(&sample_job("replace-all"), Some(&sample_preset()));

        assert!(readiness.is_ready);
        assert!(readiness.videos[0].is_ready);
    }

    #[test]
    fn keeps_processing_job_idempotent_in_readiness_layer() {
        let mut job = sample_job("replace-all");
        job.status = "Processing".to_string();

        let readiness = validate_job_readiness(&job, Some(&sample_preset()));
        assert!(readiness.is_ready);
    }

    #[test]
    fn readiness_reports_per_video_blockers() {
        let mut job = sample_job("replace-all");
        job.video_items[0].mapped_audio_path = None;
        job.video_items[0].mapped_srt_path = None;
        job.video_items[0].status = "Input Needs Review".to_string();

        let readiness = validate_job_readiness(&job, Some(&sample_preset()));

        assert_eq!(readiness.videos[0].blockers.len(), 3);
        assert!(readiness.videos[0].blockers.contains(&"Chua co audio".to_string()));
        assert!(readiness.videos[0].blockers.contains(&"Chua co SRT".to_string()));
        assert!(readiness
            .videos[0]
            .blockers
            .contains(&"Mapping chua duoc xac nhan".to_string()));
    }

    // ─── Outcome determination ─────────────────────────────────────────────────

    #[test]
    fn high_risk_segments_produce_review_needed_outcome() {
        assert_eq!(determine_video_outcome(1), "ReviewNeeded");
        assert_eq!(determine_video_outcome(3), "ReviewNeeded");
    }

    #[test]
    fn no_high_risk_segments_produce_ready_to_export_outcome() {
        assert_eq!(determine_video_outcome(0), "ReadyToExport");
    }

    #[test]
    fn job_summary_counts_outcomes_correctly() {
        let outcomes = vec![
            "ReviewNeeded".to_string(),
            "ReadyToExport".to_string(),
            "ReadyToExport".to_string(),
            "Failed".to_string(),
        ];
        let summary = compute_job_summary(&outcomes);
        assert_eq!(summary.total, 4);
        assert_eq!(summary.review_needed, 1);
        assert_eq!(summary.ready_to_export, 2);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn job_status_is_review_pending_when_any_video_needs_review() {
        let summary = JobSummary { total: 2, review_needed: 1, ready_to_export: 1, failed: 0 };
        assert_eq!(determine_job_status(&summary), "ReviewPending");
    }

    #[test]
    fn job_status_is_ready_to_export_when_all_videos_pass() {
        let summary = JobSummary { total: 2, review_needed: 0, ready_to_export: 2, failed: 0 };
        assert_eq!(determine_job_status(&summary), "ReadyToExport");
    }

    #[test]
    fn job_status_is_processed_with_failures_when_only_failures_and_exports() {
        let summary = JobSummary { total: 2, review_needed: 0, ready_to_export: 1, failed: 1 };
        assert_eq!(determine_job_status(&summary), "ProcessedWithFailures");
    }

    #[test]
    fn normalize_processing_states_resets_unfinished_video_states() {
        let job = sample_job("replace-all");
        let mut states = vec![make_video_state("video-1", "Processing", "replace-logo", &["replace-audio".to_string()])];

        normalize_processing_states(&job, &mut states);

        assert_eq!(states[0].status, "Imported");
        assert_eq!(states[0].current_step, "ready-to-run");
        assert!(states[0].completed_steps.is_empty());
    }

    #[test]
    fn normalize_exporting_states_keeps_completed_steps_and_resets_status() {
        let mut states = vec![make_video_state(
            "video-1",
            "Exporting",
            "exporting",
            &["replace-audio".to_string(), "replace-logo".to_string()],
        )];

        normalize_exporting_states(&mut states);

        assert_eq!(states[0].status, "ReadyToExport");
        assert_eq!(states[0].current_step, "ready-to-export");
        assert_eq!(states[0].completed_steps.len(), 2);
    }

    #[test]
    fn should_queue_video_only_for_unfinished_states() {
        assert!(should_queue_video("Imported"));
        assert!(should_queue_video("Matched"));
        assert!(should_queue_video("Processing"));
        assert!(!should_queue_video("ReviewNeeded"));
        assert!(!should_queue_video("ReadyToExport"));
        assert!(!should_queue_video("Failed"));
    }

    #[test]
    fn risk_distribution_low_count_does_not_underflow() {
        // low = total - high - medium; guard against underflow when high+medium > total
        let high = 3usize;
        let medium = 2usize;
        let total = 4usize;
        let low = total.saturating_sub(high + medium);
        assert_eq!(low, 0);
    }
}
