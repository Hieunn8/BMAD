use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::{
    constants::{PREVIEW_CACHE_DIR, SEGMENT_STATE_DIR, VIDEO_STATE_DIR, WORKING_DIR},
    domain::{job::Job, preset::Preset, video_item::VideoItem},
    services::{
        analysis_service::{sanitize_for_path, BoundingBox},
        job_orchestrator::load_job,
        persistence_service::{make_video_state, PersistenceService, VideoProcessingState},
        preset_service::PresetService,
        render_service::RenderService,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoFix {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleRegionFix {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitlePositionFix {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleFix {
    pub old_region: Option<SubtitleRegionFix>,
    pub new_position: Option<SubtitlePositionFix>,
    pub new_scale: Option<f32>,
    pub style_preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuickFixState {
    pub logo_fix: Option<LogoFix>,
    pub subtitle_fix: Option<SubtitleFix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSegment {
    pub id: String,
    pub video_id: String,
    pub source: String,
    pub issue_type: String,
    pub risk_level: String,
    pub review_status: String,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
    pub confidence: f32,
    pub message: String,
    pub bounding_box: Option<BoundingBox>,
    pub quick_fix_state: QuickFixState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewVideoSummary {
    pub video_id: String,
    pub video_name: String,
    pub status: String,
    pub segment_count: usize,
    pub review_required: bool,
    pub optional_review: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewVideoContext {
    pub video_id: String,
    pub source_path: String,
    pub segments: Vec<ReviewSegment>,
    pub video_status: VideoProcessingState,
    pub preview_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewContext {
    pub job_id: String,
    pub selected_video_id: Option<String>,
    pub video_list: Vec<ReviewVideoSummary>,
    pub selected_video: Option<ReviewVideoContext>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySegmentsResult {
    pub updated_segments: Vec<ReviewSegment>,
    pub warning_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewGatingBlocker {
    pub segment_id: String,
    pub time_range: String,
    pub issue_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewGatingResult {
    pub can_proceed: bool,
    pub blockers: Vec<ReviewGatingBlocker>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoReadyEvent {
    pub video_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FramePreviewResult {
    pub cache_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedLogoReview {
    video_id: String,
    detection: Option<Value>,
    segments: Vec<PersistedLogoSegment>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleReview {
    video_id: String,
    detection: Option<Value>,
    segments: Vec<PersistedSubtitleSegment>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedLogoSegment {
    issue_type: String,
    risk_level: String,
    start_ms: u64,
    end_ms: Option<u64>,
    confidence: f32,
    message: String,
    bounding_box: BoundingBox,
    review_status: Option<String>,
    quick_fix_state: Option<QuickFixState>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleSegment {
    issue_type: String,
    risk_level: String,
    start_ms: u64,
    end_ms: Option<u64>,
    confidence: f32,
    message: String,
    region: BoundingBox,
    review_status: Option<String>,
    quick_fix_state: Option<QuickFixState>,
}

enum SegmentFile {
    Logo {
        path: PathBuf,
        payload: PersistedLogoReview,
    },
    Subtitle {
        path: PathBuf,
        payload: PersistedSubtitleReview,
    },
}

#[derive(Debug, Default)]
pub struct ReviewService;

impl ReviewService {
    pub fn get_review_context(
        app_handle: &AppHandle,
        job_id: &str,
        selected_video_id: Option<&str>,
        show_all_videos: bool,
    ) -> Result<ReviewContext, String> {
        let job = load_job(app_handle, job_id)?;
        let all_videos = job
            .video_items
            .iter()
            .map(|video| {
                let segments = load_segments(&job, &video.video_id)?;
                Ok(ReviewVideoSummary {
                    video_id: video.video_id.clone(),
                    video_name: video_name(video),
                    status: video.status.clone(),
                    segment_count: segments.len(),
                    review_required: video.status == "ReviewNeeded" || has_high_risk(&segments),
                    optional_review: video.status != "ReviewNeeded" && segments.is_empty(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut filtered_videos = if show_all_videos {
            all_videos.clone()
        } else {
            all_videos
                .iter()
                .filter(|video| video.review_required || video.segment_count > 0)
                .cloned()
                .collect::<Vec<_>>()
        };

        if filtered_videos.is_empty() {
            filtered_videos = all_videos;
        }

        filtered_videos.sort_by(|left, right| {
            right
                .review_required
                .cmp(&left.review_required)
                .then(right.segment_count.cmp(&left.segment_count))
                .then(left.video_name.cmp(&right.video_name))
        });

        let selected_id = selected_video_id
            .filter(|video_id| filtered_videos.iter().any(|video| video.video_id == *video_id))
            .map(ToOwned::to_owned)
            .or_else(|| filtered_videos.first().map(|video| video.video_id.clone()));

        let selected_video = selected_id
            .as_deref()
            .map(|video_id| Self::get_video_preview(app_handle, job_id, video_id))
            .transpose()?;

        Ok(ReviewContext {
            job_id: job.job_id,
            selected_video_id: selected_id,
            video_list: filtered_videos,
            selected_video,
        })
    }

    pub fn get_video_preview(
        app_handle: &AppHandle,
        job_id: &str,
        video_id: &str,
    ) -> Result<ReviewVideoContext, String> {
        let job = load_job(app_handle, job_id)?;
        let video = find_video(&job, video_id)?;

        Ok(ReviewVideoContext {
            video_id: video.video_id.clone(),
            source_path: video.source_path.clone(),
            segments: load_segments(&job, video_id)?,
            video_status: load_video_state(&job, video)?,
            preview_path: resolve_preview_path(&job, video),
        })
    }

    pub fn apply_logo_fix(
        app_handle: &AppHandle,
        job_id: &str,
        segment_ids: &[String],
        logo_fix: &LogoFix,
    ) -> Result<ApplySegmentsResult, String> {
        let job = load_job(app_handle, job_id)?;
        let (compatible, skipped) = compatible_segment_targets(&job, segment_ids, "logo")?;
        let mut updated_segments = Vec::new();

        for target in compatible {
            let mut file = load_segment_file(&job, &target.video_id, &target.source)?;
            match &mut file {
                SegmentFile::Logo { path, payload } => {
                    let segment = payload
                        .segments
                        .get_mut(target.index)
                        .ok_or_else(|| format!("Khong tim thay segment `{}`", target.segment_id))?;
                    segment.bounding_box = BoundingBox {
                        x: logo_fix.x,
                        y: logo_fix.y,
                        width: logo_fix.width,
                        height: logo_fix.height,
                    };
                    segment.review_status = Some("Modified".to_string());
                    let mut quick_fix_state = segment.quick_fix_state.clone().unwrap_or_default();
                    quick_fix_state.logo_fix = Some(logo_fix.clone());
                    segment.quick_fix_state = Some(quick_fix_state);
                    save_logo_file(path, payload)?;
                    let refreshed = load_segments(&job, &target.video_id)?
                        .into_iter()
                        .find(|item| item.id == target.segment_id)
                        .ok_or_else(|| format!("Khong tim thay segment `{}` sau khi update", target.segment_id))?;
                    updated_segments.push(refreshed);
                }
                SegmentFile::Subtitle { .. } => {}
            }
        }

        invalidate_preview_cache(&job, segment_ids)?;

        Ok(ApplySegmentsResult {
            updated_segments,
            warning_message: (!skipped.is_empty()).then(|| {
                "Mot so doan khong the ap dung fix nay".to_string()
            }),
        })
    }

    pub fn apply_subtitle_fix(
        app_handle: &AppHandle,
        job_id: &str,
        segment_ids: &[String],
        subtitle_fix: &SubtitleFix,
    ) -> Result<ApplySegmentsResult, String> {
        let job = load_job(app_handle, job_id)?;
        let (compatible, skipped) = compatible_segment_targets(&job, segment_ids, "subtitle")?;
        let mut updated_segments = Vec::new();

        for target in compatible {
            let mut file = load_segment_file(&job, &target.video_id, &target.source)?;
            if let SegmentFile::Subtitle { path, payload } = &mut file {
                let segment = payload
                    .segments
                    .get_mut(target.index)
                    .ok_or_else(|| format!("Khong tim thay segment `{}`", target.segment_id))?;

                if let Some(old_region) = &subtitle_fix.old_region {
                    segment.region = BoundingBox {
                        x: old_region.x,
                        y: old_region.y,
                        width: old_region.width,
                        height: old_region.height,
                    };
                }

                segment.review_status = Some("Modified".to_string());
                let mut quick_fix_state = segment.quick_fix_state.clone().unwrap_or_default();
                quick_fix_state.subtitle_fix = Some(subtitle_fix.clone());
                segment.quick_fix_state = Some(quick_fix_state);
                save_subtitle_file(path, payload)?;
                let refreshed = load_segments(&job, &target.video_id)?
                    .into_iter()
                    .find(|item| item.id == target.segment_id)
                    .ok_or_else(|| format!("Khong tim thay segment `{}` sau khi update", target.segment_id))?;
                updated_segments.push(refreshed);
            }
        }

        invalidate_preview_cache(&job, segment_ids)?;

        Ok(ApplySegmentsResult {
            updated_segments,
            warning_message: (!skipped.is_empty()).then(|| {
                "Mot so doan khong the ap dung fix nay".to_string()
            }),
        })
    }

    pub fn mark_segment_accepted(
        app_handle: &AppHandle,
        job_id: &str,
        segment_id: &str,
    ) -> Result<ReviewSegment, String> {
        let job = load_job(app_handle, job_id)?;
        let target = find_segment_target(&job, segment_id)?;
        let mut file = load_segment_file(&job, &target.video_id, &target.source)?;

        match &mut file {
            SegmentFile::Logo { path, payload } => {
                payload
                    .segments
                    .get_mut(target.index)
                    .ok_or_else(|| format!("Khong tim thay segment `{segment_id}`"))?
                    .review_status = Some("Accepted".to_string());
                save_logo_file(path, payload)?;
            }
            SegmentFile::Subtitle { path, payload } => {
                payload
                    .segments
                    .get_mut(target.index)
                    .ok_or_else(|| format!("Khong tim thay segment `{segment_id}`"))?
                    .review_status = Some("Accepted".to_string());
                save_subtitle_file(path, payload)?;
            }
        }

        load_segments(&job, &target.video_id)?
            .into_iter()
            .find(|segment| segment.id == segment_id)
            .ok_or_else(|| format!("Khong tim thay segment `{segment_id}` sau khi update"))
    }

    pub fn reset_logo_fix(
        app_handle: &AppHandle,
        job_id: &str,
        segment_id: &str,
    ) -> Result<ReviewSegment, String> {
        let job = load_job(app_handle, job_id)?;
        let target = find_segment_target(&job, segment_id)?;
        let video = find_video(&job, &target.video_id)?;
        let preset = load_preset(app_handle, job.preset_id.as_deref())?;

        let mut file = load_segment_file(&job, &target.video_id, &target.source)?;
        if let SegmentFile::Logo { path, payload } = &mut file {
            let segment = payload
                .segments
                .get_mut(target.index)
                .ok_or_else(|| format!("Khong tim thay segment `{segment_id}`"))?;
            let fallback_box = default_logo_box(app_handle, video, preset.as_ref())?;
            segment.bounding_box = fallback_box;
            segment.review_status = Some("Unreviewed".to_string());
            let mut quick_fix_state = segment.quick_fix_state.clone().unwrap_or_default();
            quick_fix_state.logo_fix = None;
            segment.quick_fix_state = Some(quick_fix_state);
            save_logo_file(path, payload)?;
        }

        invalidate_preview_cache(&job, &[segment_id.to_string()])?;

        load_segments(&job, &target.video_id)?
            .into_iter()
            .find(|segment| segment.id == segment_id)
            .ok_or_else(|| format!("Khong tim thay segment `{segment_id}` sau khi reset"))
    }

    pub fn reset_subtitle_fix(
        app_handle: &AppHandle,
        job_id: &str,
        segment_id: &str,
    ) -> Result<ReviewSegment, String> {
        let job = load_job(app_handle, job_id)?;
        let target = find_segment_target(&job, segment_id)?;
        let video = find_video(&job, &target.video_id)?;
        let preset = load_preset(app_handle, job.preset_id.as_deref())?;
        let mut file = load_segment_file(&job, &target.video_id, &target.source)?;

        if let SegmentFile::Subtitle { path, payload } = &mut file {
            let segment = payload
                .segments
                .get_mut(target.index)
                .ok_or_else(|| format!("Khong tim thay segment `{segment_id}`"))?;
            segment.review_status = Some("Unreviewed".to_string());
            let default_fix = default_subtitle_fix(app_handle, video, preset.as_ref(), &segment.region)?;
            let mut quick_fix_state = segment.quick_fix_state.clone().unwrap_or_default();
            quick_fix_state.subtitle_fix = Some(default_fix);
            segment.quick_fix_state = Some(quick_fix_state);
            save_subtitle_file(path, payload)?;
        }

        invalidate_preview_cache(&job, &[segment_id.to_string()])?;

        load_segments(&job, &target.video_id)?
            .into_iter()
            .find(|segment| segment.id == segment_id)
            .ok_or_else(|| format!("Khong tim thay segment `{segment_id}` sau khi reset"))
    }

    pub fn check_video_review_gating(
        app_handle: &AppHandle,
        job_id: &str,
        video_id: &str,
    ) -> Result<ReviewGatingResult, String> {
        let job = load_job(app_handle, job_id)?;
        let segments = load_segments(&job, video_id)?;
        let blockers = segments
            .into_iter()
            .filter(|segment| segment.risk_level == "High" && segment.review_status == "Unreviewed")
            .map(|segment| ReviewGatingBlocker {
                segment_id: segment.id,
                time_range: format_time_range(segment.start_ms, segment.end_ms),
                issue_type: segment.issue_type,
            })
            .collect::<Vec<_>>();

        Ok(ReviewGatingResult {
            can_proceed: blockers.is_empty(),
            blockers,
        })
    }

    pub fn mark_video_ready(
        app_handle: &AppHandle,
        job_id: &str,
        video_id: &str,
    ) -> Result<ReviewGatingResult, String> {
        let gating = Self::check_video_review_gating(app_handle, job_id, video_id)?;
        if !gating.can_proceed {
            return Ok(gating);
        }

        let mut job = load_job(app_handle, job_id)?;
        let video = job
            .video_items
            .iter_mut()
            .find(|item| item.video_id == video_id)
            .ok_or_else(|| format!("Khong tim thay video `{video_id}` trong job"))?;
        video.status = "ReadyToExport".to_string();

        let ready_count = job
            .video_items
            .iter()
            .filter(|item| item.status == "ReadyToExport" || item.status == "Failed")
            .count();
        if ready_count == job.video_items.len() {
            job.status = "ReadyToExport".to_string();
        }

        PersistenceService::persist_job(app_handle, &job)?;
        PersistenceService::persist_video_state(
            &job,
            &make_video_state(video_id, "ReadyToExport", "review-complete", &["review-complete".to_string()]),
        )?;

        app_handle
            .emit(
                "videoReadyToExport",
                VideoReadyEvent {
                    video_id: video_id.to_string(),
                },
            )
            .map_err(|error| format!("Khong emit duoc videoReadyToExport: {error}"))?;

        Ok(gating)
    }

    pub fn get_frame_preview(
        app_handle: &AppHandle,
        job_id: &str,
        segment_id: &str,
        time_seconds: f64,
        pending_logo_fix: Option<&LogoFix>,
        pending_subtitle_fix: Option<&SubtitleFix>,
    ) -> Result<FramePreviewResult, String> {
        let job = load_job(app_handle, job_id)?;
        let target = find_segment_target_by_id(&job, segment_id)?;
        let video = find_video(&job, &target.video_id)?;

        let cache_dir = PathBuf::from(&job.output_folder).join(PREVIEW_CACHE_DIR);
        fs::create_dir_all(&cache_dir).map_err(|error| format!("Khong tao duoc preview cache dir: {error}"))?;

        let effective_logo_fix = pending_logo_fix.cloned().or_else(|| target.segment.quick_fix_state.logo_fix.clone());
        let effective_subtitle_fix =
            pending_subtitle_fix.cloned().or_else(|| target.segment.quick_fix_state.subtitle_fix.clone());
        let fix_json = serde_json::to_string(&QuickFixState {
            logo_fix: effective_logo_fix.clone(),
            subtitle_fix: effective_subtitle_fix.clone(),
        })
        .unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        format!("{segment_id}:{fix_json}").hash(&mut hasher);
        let hash = hasher.finish();
        let cache_path = cache_dir.join(format!("{}-{:x}.mp4", sanitize_for_path(segment_id), hash));

        if !cache_path.exists() {
            let temp_source_clip = cache_dir.join(format!("{}-{:x}-source.mp4", sanitize_for_path(segment_id), hash));
            let base_input = preview_base_path_for_segment(&job, video, &target.segment, effective_logo_fix.is_some());

            RenderService::extract_preview_clip(
                app_handle,
                &base_input,
                &temp_source_clip,
                time_seconds,
                preview_duration_for_segment(&target.segment),
            )?;

            if let Some(logo_fix) = effective_logo_fix {
                let preset = load_preset(app_handle, job.preset_id.as_deref())?;
                let logo_source = video
                    .mapped_logo_path
                    .clone()
                    .or_else(|| preset.map(|value| value.default_logo_path))
                    .ok_or_else(|| format!("Video `{}` chua co logo source", video.video_id))?;
                let target_box = BoundingBox {
                    x: logo_fix.x,
                    y: logo_fix.y,
                    width: logo_fix.width,
                    height: logo_fix.height,
                };
                RenderService::overlay_logo_to_output(
                    app_handle,
                    temp_source_clip.to_str().unwrap_or_default(),
                    &cache_path,
                    &logo_source,
                    &target_box,
                )?;
            } else if let Some(subtitle_fix) = effective_subtitle_fix {
                let region = subtitle_fix
                    .old_region
                    .clone()
                    .or_else(|| {
                        target.segment.bounding_box.clone().map(|bbox| SubtitleRegionFix {
                            x: bbox.x,
                            y: bbox.y,
                            width: bbox.width,
                            height: bbox.height,
                            mode: "blur".to_string(),
                        })
                    })
                    .ok_or_else(|| format!("Segment `{segment_id}` khong co subtitle region de preview"))?;
                let subtitle_region = crate::services::analysis_service::SubtitleRegion {
                    x: region.x,
                    y: region.y,
                    width: region.width,
                    height: region.height,
                    confidence: target.segment.confidence,
                };
                RenderService::remove_subtitle_to_output(
                    app_handle,
                    temp_source_clip.to_str().unwrap_or_default(),
                    &cache_path,
                    &subtitle_region,
                    &region.mode,
                )?;
                if let Some(srt_path) = video.mapped_srt_path.clone() {
                    let metadata = crate::services::analysis_service::AnalysisService::probe_metadata(
                        app_handle,
                        temp_source_clip.to_str().unwrap_or(video.source_path.as_str()),
                    )?;
                    let preset = load_preset(app_handle, job.preset_id.as_deref())?;
                    let force_style = subtitle_force_style_from_fix(
                        preset.as_ref(),
                        &subtitle_fix,
                        &metadata,
                    );
                    let rendered_path = cache_dir.join(format!("{}-{:x}-rendered.mp4", sanitize_for_path(segment_id), hash));
                    RenderService::render_subtitle_to_output(
                        app_handle,
                        cache_path.to_str().unwrap_or_default(),
                        &rendered_path,
                        &srt_path,
                        &force_style,
                    )?;
                    fs::rename(&rendered_path, &cache_path)
                        .or_else(|_| {
                            fs::copy(&rendered_path, &cache_path).map(|_| ())
                        })
                        .map_err(|error| format!("Khong thay file preview subtitle rendered: {error}"))?;
                    let _ = fs::remove_file(rendered_path);
                }
            } else {
                fs::copy(&temp_source_clip, &cache_path)
                    .map_err(|error| format!("Khong copy duoc preview clip cache: {error}"))?;
            }

            let _ = fs::remove_file(temp_source_clip);
        }

        Ok(FramePreviewResult {
            cache_path: cache_path.display().to_string(),
        })
    }
}

#[derive(Clone)]
struct SegmentTarget {
    segment_id: String,
    video_id: String,
    source: String,
    index: usize,
    segment: ReviewSegment,
}

fn find_video<'a>(job: &'a Job, video_id: &str) -> Result<&'a VideoItem, String> {
    job.video_items
        .iter()
        .find(|item| item.video_id == video_id)
        .ok_or_else(|| format!("Khong tim thay video `{video_id}` trong job"))
}

fn load_preset(app_handle: &AppHandle, preset_id: Option<&str>) -> Result<Option<Preset>, String> {
    match preset_id {
        Some(value) => PresetService::get_preset(app_handle, value).map(Some),
        None => Ok(None),
    }
}

fn load_segments(job: &Job, video_id: &str) -> Result<Vec<ReviewSegment>, String> {
    let mut segments = Vec::new();

    if let SegmentFile::Logo { payload, .. } = load_segment_file(job, video_id, "logo")? {
        segments.extend(payload.segments.into_iter().enumerate().map(|(index, segment)| ReviewSegment {
            id: format!("{}:logo:{index}", payload.video_id),
            video_id: payload.video_id.clone(),
            source: "logo".to_string(),
            issue_type: segment.issue_type,
            risk_level: segment.risk_level,
            review_status: segment.review_status.unwrap_or_else(|| "Unreviewed".to_string()),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            confidence: segment.confidence,
            message: segment.message,
            bounding_box: Some(segment.bounding_box),
            quick_fix_state: segment.quick_fix_state.unwrap_or_default(),
        }));
    }

    if let SegmentFile::Subtitle { payload, .. } = load_segment_file(job, video_id, "subtitle")? {
        segments.extend(payload.segments.into_iter().enumerate().map(|(index, segment)| ReviewSegment {
            id: format!("{}:subtitle:{index}", payload.video_id),
            video_id: payload.video_id.clone(),
            source: "subtitle".to_string(),
            issue_type: segment.issue_type,
            risk_level: segment.risk_level,
            review_status: segment.review_status.unwrap_or_else(|| "Unreviewed".to_string()),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            confidence: segment.confidence,
            message: segment.message,
            bounding_box: Some(segment.region),
            quick_fix_state: segment.quick_fix_state.unwrap_or_default(),
        }));
    }

    Ok(segments)
}

fn load_video_state(job: &Job, video: &VideoItem) -> Result<VideoProcessingState, String> {
    let videos_dir = PathBuf::from(&job.output_folder).join(VIDEO_STATE_DIR);
    let safe_id = sanitize_for_path(&video.video_id);
    let path = videos_dir.join(format!("{safe_id}.json"));

    if path.exists() {
        let payload = fs::read_to_string(&path)
            .map_err(|error| format!("Khong doc duoc video state `{}`: {error}", path.display()))?;
        return serde_json::from_str(&payload)
            .map_err(|error| format!("Khong parse duoc video state `{}`: {error}", path.display()));
    }

    Ok(VideoProcessingState {
        video_id: video.video_id.clone(),
        status: video.status.clone(),
        current_step: "done".to_string(),
        completed_steps: Vec::new(),
        timestamp: job.created_at.clone(),
        audio_replacement_applied: None,
        audio_source_path: None,
        output_path: None,
        encode_summary: None,
    })
}

fn resolve_preview_path(job: &Job, video: &VideoItem) -> String {
    let safe_id = sanitize_for_path(&video.video_id);
    let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);
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

fn preview_base_path_for_segment(job: &Job, video: &VideoItem, segment: &ReviewSegment, needs_logo_apply: bool) -> String {
    let safe_id = sanitize_for_path(&video.video_id);
    let working_dir = PathBuf::from(&job.output_folder).join(WORKING_DIR);

    let candidates = if needs_logo_apply || segment.source == "logo" {
        vec![
            working_dir.join(format!("{safe_id}_audio_replaced.mp4")),
            PathBuf::from(&video.source_path),
        ]
    } else if segment.source == "subtitle" {
        vec![
            working_dir.join(format!("{safe_id}_logo_replaced.mp4")),
            working_dir.join(format!("{safe_id}_audio_replaced.mp4")),
            PathBuf::from(&video.source_path),
        ]
    } else {
        vec![PathBuf::from(resolve_preview_path(job, video))]
    };

    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from(&video.source_path))
        .display()
        .to_string()
}

fn default_subtitle_fix(
    app_handle: &AppHandle,
    video: &VideoItem,
    preset: Option<&Preset>,
    region: &BoundingBox,
) -> Result<SubtitleFix, String> {
    let metadata = crate::services::analysis_service::AnalysisService::probe_metadata(
        app_handle,
        video.source_path.as_str(),
    )?;
    let baseline_ratio = subtitle_baseline_ratio_from_rules(preset).unwrap_or(0.08);
    let position_x = metadata.width / 2;
    let position_y = metadata
        .height
        .saturating_sub(((metadata.height as f64) * baseline_ratio) as u32);

    Ok(SubtitleFix {
        old_region: Some(SubtitleRegionFix {
            x: region.x,
            y: region.y,
            width: region.width,
            height: region.height,
            mode: removal_mode_from_preset_label(preset),
        }),
        new_position: Some(SubtitlePositionFix {
            x: position_x,
            y: position_y,
        }),
        new_scale: Some(1.0),
        style_preset: Some(
            preset
                .map(|value| value.subtitle_style_preset.clone())
                .unwrap_or_else(|| "default".to_string()),
        ),
    })
}

fn subtitle_baseline_ratio_from_rules(preset: Option<&Preset>) -> Option<f64> {
    let rules = preset?.layout_rules.to_ascii_lowercase();
    let index = rules.find("baseline")?;
    let tail = &rules[index..];
    let percent_index = tail.find('%')?;
    let numeric = tail[..percent_index]
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    numeric.parse::<f64>().ok().map(|value| value / 100.0)
}

fn removal_mode_from_preset_label(preset: Option<&Preset>) -> String {
    let style = preset
        .map(|value| value.subtitle_style_preset.to_ascii_lowercase())
        .unwrap_or_default();
    if style.contains("drawbox") || style.contains("fill") || style.contains("mask") {
        "fill".to_string()
    } else {
        "blur".to_string()
    }
}

fn subtitle_force_style_from_fix(
    preset: Option<&Preset>,
    subtitle_fix: &SubtitleFix,
    metadata: &crate::services::analysis_service::VideoMetadata,
) -> String {
    let mut style = preset
        .map(|value| value.subtitle_style_preset.clone())
        .filter(|value| value.contains("FontName=") || value.contains("Fontname="))
        .unwrap_or_else(|| {
            "FontName=Arial,FontSize=24,PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,Outline=1".to_string()
        });

    let scale = subtitle_fix.new_scale.unwrap_or(1.0).clamp(0.4, 3.0);
    let base_font_size = 24.0 * scale;
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

    if let Some(custom_style) = &subtitle_fix.style_preset {
        if !custom_style.trim().is_empty() && custom_style.contains("FontName=") {
            style = custom_style.clone();
        }
    }

    format!(
        "{style},Alignment={alignment},MarginL={margin_l},MarginR={margin_r},MarginV={margin_v},FontSize={}",
        base_font_size.round() as u32
    )
}

fn default_logo_box(
    app_handle: &AppHandle,
    video: &VideoItem,
    preset: Option<&Preset>,
) -> Result<BoundingBox, String> {
    let preview_or_source = PathBuf::from(&video.source_path);
    let metadata = crate::services::analysis_service::AnalysisService::probe_metadata(
        app_handle,
        preview_or_source.to_str().unwrap_or(video.source_path.as_str()),
    )?;
    let logo_source = video
        .mapped_logo_path
        .clone()
        .or_else(|| preset.map(|value| value.default_logo_path.clone()))
        .ok_or_else(|| format!("Video `{}` chua co logo source", video.video_id))?;
    let (width, height) =
        crate::services::analysis_service::AnalysisService::estimate_logo_size_for_video(&logo_source, &metadata);
    Ok(crate::services::analysis_service::AnalysisService::default_bounding_box_from_preset(
        preset,
        &metadata,
        width,
        height,
    ))
}

fn has_high_risk(segments: &[ReviewSegment]) -> bool {
    segments.iter().any(|segment| segment.risk_level == "High")
}

fn video_name(video: &VideoItem) -> String {
    Path::new(&video.source_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(video.source_path.as_str())
        .to_string()
}

fn segment_file_path(job: &Job, video_id: &str, source: &str) -> PathBuf {
    let safe_id = sanitize_for_path(video_id);
    PathBuf::from(&job.output_folder)
        .join(SEGMENT_STATE_DIR)
        .join(format!("{safe_id}_{source}.json"))
}

fn load_segment_file(job: &Job, video_id: &str, source: &str) -> Result<SegmentFile, String> {
    let path = segment_file_path(job, video_id, source);
    if !path.exists() {
        return match source {
            "logo" => Ok(SegmentFile::Logo {
                path,
                payload: PersistedLogoReview {
                    video_id: video_id.to_string(),
                    detection: None,
                    segments: Vec::new(),
                },
            }),
            _ => Ok(SegmentFile::Subtitle {
                path,
                payload: PersistedSubtitleReview {
                    video_id: video_id.to_string(),
                    detection: None,
                    segments: Vec::new(),
                },
            }),
        };
    }

    let payload = fs::read_to_string(&path)
        .map_err(|error| format!("Khong doc duoc segment file `{}`: {error}", path.display()))?;
    match source {
        "logo" => {
            let parsed = serde_json::from_str(&payload)
                .map_err(|error| format!("Khong parse duoc logo segments `{}`: {error}", path.display()))?;
            Ok(SegmentFile::Logo { path, payload: parsed })
        }
        _ => {
            let parsed = serde_json::from_str(&payload)
                .map_err(|error| format!("Khong parse duoc subtitle segments `{}`: {error}", path.display()))?;
            Ok(SegmentFile::Subtitle { path, payload: parsed })
        }
    }
}

fn save_logo_file(path: &Path, payload: &PersistedLogoReview) -> Result<(), String> {
    let json = serde_json::to_string_pretty(payload)
        .map_err(|error| format!("Khong serialize duoc logo review data: {error}"))?;
    fs::write(path, json).map_err(|error| format!("Khong ghi duoc logo review file `{}`: {error}", path.display()))
}

fn save_subtitle_file(path: &Path, payload: &PersistedSubtitleReview) -> Result<(), String> {
    let json = serde_json::to_string_pretty(payload)
        .map_err(|error| format!("Khong serialize duoc subtitle review data: {error}"))?;
    fs::write(path, json)
        .map_err(|error| format!("Khong ghi duoc subtitle review file `{}`: {error}", path.display()))
}

fn compatible_segment_targets(
    job: &Job,
    segment_ids: &[String],
    source: &str,
) -> Result<(Vec<SegmentTarget>, Vec<String>), String> {
    let mut compatible = Vec::new();
    let mut skipped = Vec::new();
    let base_target = segment_ids
        .first()
        .map(|segment_id| find_segment_target(job, segment_id))
        .transpose()?;

    for segment_id in segment_ids.iter() {
        let target = find_segment_target(job, segment_id)?;
        let same_source = target.source == source;
        let same_video = base_target
            .as_ref()
            .map(|base| base.video_id == target.video_id)
            .unwrap_or(true);
        let same_issue = base_target
            .as_ref()
            .map(|base| base.segment.issue_type == target.segment.issue_type)
            .unwrap_or(true);

        if same_source && same_video && same_issue {
            compatible.push(target);
        } else {
            skipped.push(segment_id.clone());
        }
    }

    Ok((compatible, skipped))
}

fn find_segment_target(job: &Job, segment_id: &str) -> Result<SegmentTarget, String> {
    find_segment_target_by_id(job, segment_id)
}

fn find_segment_target_by_id(job: &Job, segment_id: &str) -> Result<SegmentTarget, String> {
    let mut parts = segment_id.rsplitn(3, ':');
    let index = parts
        .next()
        .ok_or_else(|| format!("Segment id khong hop le `{segment_id}`"))?
        .parse::<usize>()
        .map_err(|error| format!("Khong parse duoc segment index `{segment_id}`: {error}"))?;
    let source = parts.next().ok_or_else(|| format!("Segment id khong hop le `{segment_id}`"))?;
    let video_id = parts.next().ok_or_else(|| format!("Segment id khong hop le `{segment_id}`"))?;

    let segments = load_segments(job, video_id)?;
    let segment = segments
        .into_iter()
        .find(|value| value.id == segment_id)
        .ok_or_else(|| format!("Khong tim thay segment `{segment_id}`"))?;

    Ok(SegmentTarget {
        segment_id: segment_id.to_string(),
        video_id: video_id.to_string(),
        source: source.to_string(),
        index,
        segment,
    })
}

fn invalidate_preview_cache(job: &Job, segment_ids: &[String]) -> Result<(), String> {
    let cache_dir = PathBuf::from(&job.output_folder).join(PREVIEW_CACHE_DIR);
    if !cache_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&cache_dir)
        .map_err(|error| format!("Khong doc duoc preview cache dir `{}`: {error}", cache_dir.display()))?
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        if segment_ids
            .iter()
            .any(|segment_id| file_name.starts_with(&sanitize_for_path(segment_id)))
        {
            let _ = fs::remove_file(path);
        }
    }

    Ok(())
}

fn format_time_range(start_ms: u64, end_ms: Option<u64>) -> String {
    format!("{}-{}", format_time(start_ms), format_time(end_ms.unwrap_or(start_ms)))
}

fn format_time(value_ms: u64) -> String {
    let total_seconds = value_ms / 1000;
    format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60)
}

fn preview_duration_for_segment(segment: &ReviewSegment) -> f64 {
    let end_ms = segment.end_ms.unwrap_or(segment.start_ms + 3000);
    (((end_ms.saturating_sub(segment.start_ms)) as f64) / 1000.0).clamp(1.5, 6.0)
}
