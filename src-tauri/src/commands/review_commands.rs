use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::services::review_service::{
    ApplySegmentsResult, FramePreviewResult, LogoFix, ReviewContext, ReviewGatingResult,
    ReviewSegment, ReviewService, ReviewVideoContext, SubtitleFix,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewContextResponse {
    pub context: ReviewContext,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoReviewContextResponse {
    pub context: ReviewVideoContext,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySegmentsResponse {
    pub result: ApplySegmentsResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSegmentResponse {
    pub segment: ReviewSegment,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewGatingResponse {
    pub result: ReviewGatingResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FramePreviewResponse {
    pub result: FramePreviewResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyLogoFixPayload {
    pub segment_ids: Vec<String>,
    pub logo_fix: LogoFix,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySubtitleFixPayload {
    pub segment_ids: Vec<String>,
    pub subtitle_fix: SubtitleFix,
}

#[tauri::command]
pub fn get_review_context(
    app_handle: AppHandle,
    job_id: String,
    selected_video_id: Option<String>,
    show_all_videos: bool,
) -> Result<ReviewContextResponse, String> {
    Ok(ReviewContextResponse {
        context: ReviewService::get_review_context(
            &app_handle,
            &job_id,
            selected_video_id.as_deref(),
            show_all_videos,
        )?,
    })
}

#[tauri::command]
pub fn get_video_preview(
    app_handle: AppHandle,
    job_id: String,
    video_id: String,
) -> Result<VideoReviewContextResponse, String> {
    Ok(VideoReviewContextResponse {
        context: ReviewService::get_video_preview(&app_handle, &job_id, &video_id)?,
    })
}

#[tauri::command]
pub fn apply_logo_fix(
    app_handle: AppHandle,
    job_id: String,
    payload: ApplyLogoFixPayload,
) -> Result<ApplySegmentsResponse, String> {
    Ok(ApplySegmentsResponse {
        result: ReviewService::apply_logo_fix(
            &app_handle,
            &job_id,
            &payload.segment_ids,
            &payload.logo_fix,
        )?,
    })
}

#[tauri::command]
pub fn apply_subtitle_fix(
    app_handle: AppHandle,
    job_id: String,
    payload: ApplySubtitleFixPayload,
) -> Result<ApplySegmentsResponse, String> {
    Ok(ApplySegmentsResponse {
        result: ReviewService::apply_subtitle_fix(
            &app_handle,
            &job_id,
            &payload.segment_ids,
            &payload.subtitle_fix,
        )?,
    })
}

#[tauri::command]
pub fn mark_segment_accepted(
    app_handle: AppHandle,
    job_id: String,
    segment_id: String,
) -> Result<ReviewSegmentResponse, String> {
    Ok(ReviewSegmentResponse {
        segment: ReviewService::mark_segment_accepted(&app_handle, &job_id, &segment_id)?,
    })
}

#[tauri::command]
pub fn reset_logo_fix(
    app_handle: AppHandle,
    job_id: String,
    segment_id: String,
) -> Result<ReviewSegmentResponse, String> {
    Ok(ReviewSegmentResponse {
        segment: ReviewService::reset_logo_fix(&app_handle, &job_id, &segment_id)?,
    })
}

#[tauri::command]
pub fn reset_subtitle_fix(
    app_handle: AppHandle,
    job_id: String,
    segment_id: String,
) -> Result<ReviewSegmentResponse, String> {
    Ok(ReviewSegmentResponse {
        segment: ReviewService::reset_subtitle_fix(&app_handle, &job_id, &segment_id)?,
    })
}

#[tauri::command]
pub fn check_video_review_gating(
    app_handle: AppHandle,
    job_id: String,
    video_id: String,
) -> Result<ReviewGatingResponse, String> {
    Ok(ReviewGatingResponse {
        result: ReviewService::check_video_review_gating(&app_handle, &job_id, &video_id)?,
    })
}

#[tauri::command]
pub fn mark_video_ready(
    app_handle: AppHandle,
    job_id: String,
    video_id: String,
) -> Result<ReviewGatingResponse, String> {
    Ok(ReviewGatingResponse {
        result: ReviewService::mark_video_ready(&app_handle, &job_id, &video_id)?,
    })
}

#[tauri::command]
pub fn get_frame_preview(
    app_handle: AppHandle,
    job_id: String,
    segment_id: String,
    time_seconds: f64,
    pending_logo_fix: Option<LogoFix>,
    pending_subtitle_fix: Option<SubtitleFix>,
) -> Result<FramePreviewResponse, String> {
    Ok(FramePreviewResponse {
        result: ReviewService::get_frame_preview(
            &app_handle,
            &job_id,
            &segment_id,
            time_seconds,
            pending_logo_fix.as_ref(),
            pending_subtitle_fix.as_ref(),
        )?,
    })
}
