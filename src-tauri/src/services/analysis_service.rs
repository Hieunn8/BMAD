use std::{
    fs,
    path::Path,
    process::Command,
};

use image::{GenericImageView, ImageReader};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    commands::app::{resolve_ffmpeg_path, resolve_ffprobe_path},
    domain::{job::Job, preset::Preset, video_item::VideoItem},
    services::{logging_service::LoggingService, risk_service::RiskService},
};

const SAMPLE_PERCENTAGES: [f64; 5] = [0.1, 0.25, 0.5, 0.75, 0.9];

// RGB delta above which a corner is considered too noisy to be a reliable logo region.
// 42 ≈ 16% of the 0–255 scale — empirically chosen for heuristic stability detection.
const MAX_MEANINGFUL_RGB_DELTA: f64 = 42.0;

// Sentinel returned by average_delta_for_corner when there is insufficient frame data.
// Forces confidence toward the minimum — equivalent to "no information".
const DELTA_NO_DATA: f64 = f64::MAX / 2.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoSegment {
    pub issue_type: String,
    pub risk_level: String,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
    pub confidence: f32,
    pub message: String,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoDetectionResult {
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub risk_level: String,
    pub matched: bool,
    pub sampled_frame_count: usize,
    pub matched_corner: String,
    pub segments: Vec<LogoSegment>,
}

#[derive(Debug)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl Corner {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomRight => "bottom-right",
            Self::BottomLeft => "bottom-left",
        }
    }
}

// Bottom-of-frame ratios for subtitle region scanning (most subtitles appear here).
const SUBTITLE_BAND_TOP_RATIO: f64 = 0.75;
const SUBTITLE_BAND_BOTTOM_RATIO: f64 = 0.95;

// Per-frame mean brightness variance threshold above which a bottom region is
// considered to contain changing subtitle text.
const SUBTITLE_VARIANCE_THRESHOLD: f64 = 8.0;

#[derive(Debug)]
struct RegionSummary {
    corner: Corner,
    mean_rgb: [f64; 3],
}

// ─── Subtitle detection types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleSegment {
    pub issue_type: String,
    pub risk_level: String,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
    pub confidence: f32,
    pub message: String,
    pub region: SubtitleRegion,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleDetectionResult {
    pub regions: Vec<SubtitleRegion>,
    pub confidence: f32,
    pub detected: bool,
    pub sampled_frame_count: usize,
    pub segments: Vec<SubtitleSegment>,
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct AnalysisService;

impl AnalysisService {
    pub fn detect_logo(
        app_handle: &AppHandle,
        job: &Job,
        preset: Option<&Preset>,
        video: &VideoItem,
        input_video_path: &str,
    ) -> Result<LogoDetectionResult, String> {
        let logo_source = resolve_logo_source(video, preset)?;
        let ffprobe_path = resolve_ffprobe_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFprobe binary da bundle".to_string())?;
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        let metadata = probe_video_metadata(&ffprobe_path, input_video_path)?;
        let preferred_corner = preferred_corner_from_rules(preset);
        let (logo_width, logo_height) = estimate_logo_size(&logo_source, &metadata);

        let sample_dir = std::env::temp_dir()
            .join(format!("logo-detect-{}", sanitize_for_path(&video.video_id)));
        recreate_directory(&sample_dir)?;

        // Run detection in an inner closure so temp dir is always cleaned up.
        let inner_result = detect_logo_inner(
            &ffmpeg_path,
            input_video_path,
            &metadata,
            preferred_corner,
            logo_width,
            logo_height,
            &sample_dir,
        );
        let _ = fs::remove_dir_all(&sample_dir);
        let (best_corner, confidence, frame_count) = inner_result?;

        let bounding_box = build_bounding_box(best_corner, metadata.width, metadata.height, logo_width, logo_height);
        let risk_level = RiskService::from_confidence(confidence);
        let segments = build_segments(&bounding_box, confidence, risk_level.as_str());
        let segment_count = segments.len();

        let result = LogoDetectionResult {
            bounding_box: bounding_box.clone(),
            confidence,
            risk_level: risk_level.as_str().to_string(),
            matched: confidence >= 0.5,
            sampled_frame_count: frame_count,
            matched_corner: best_corner.as_str().to_string(),
            segments,
        };

        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "logo-detection: videoId={}; input={input_video_path}; corner={}; confidence={:.2}; bbox={},{},{},{}; segmentCount={}; riskLevel={}",
                video.video_id,
                result.matched_corner,
                result.confidence,
                result.bounding_box.x,
                result.bounding_box.y,
                result.bounding_box.width,
                result.bounding_box.height,
                segment_count,
                result.risk_level,
            ),
        )?;

        Ok(result)
    }

    /// Returns a bounding box derived purely from preset layout rules — used as fallback
    /// when detection confidence is too low to trust the heuristic result.
    pub fn default_bounding_box_from_preset(
        preset: Option<&Preset>,
        metadata: &VideoMetadata,
        logo_width: u32,
        logo_height: u32,
    ) -> BoundingBox {
        let corner = preferred_corner_from_rules(preset);
        build_bounding_box(corner, metadata.width, metadata.height, logo_width, logo_height)
    }

    pub fn probe_metadata(
        app_handle: &AppHandle,
        input_video_path: &str,
    ) -> Result<VideoMetadata, String> {
        let ffprobe_path = resolve_ffprobe_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFprobe binary da bundle".to_string())?;
        probe_video_metadata(&ffprobe_path, input_video_path)
    }

    pub fn estimate_logo_size_for_video(logo_path: &str, metadata: &VideoMetadata) -> (u32, u32) {
        estimate_logo_size(logo_path, metadata)
    }

    /// Detects hardcoded subtitle regions in the bottom band of a video by comparing
    /// pixel variance across sampled frames. High frame-to-frame variation in the bottom
    /// region indicates changing subtitle text.
    pub fn detect_subtitle_regions(
        app_handle: &AppHandle,
        job: &Job,
        video: &VideoItem,
        input_video_path: &str,
    ) -> Result<SubtitleDetectionResult, String> {
        let ffprobe_path = resolve_ffprobe_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFprobe binary da bundle".to_string())?;
        let ffmpeg_path = resolve_ffmpeg_path(app_handle)
            .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())?;
        let metadata = probe_video_metadata(&ffprobe_path, input_video_path)?;

        let sample_dir = std::env::temp_dir()
            .join(format!("sub-detect-{}", sanitize_for_path(&video.video_id)));
        recreate_directory(&sample_dir)?;

        let inner_result = detect_subtitle_inner(&ffmpeg_path, input_video_path, &metadata, &sample_dir);
        let _ = fs::remove_dir_all(&sample_dir);
        let (confidence, frame_count) = inner_result?;

        let y_top = ((metadata.height as f64) * SUBTITLE_BAND_TOP_RATIO) as u32;
        let y_bottom = ((metadata.height as f64) * SUBTITLE_BAND_BOTTOM_RATIO) as u32;
        let band_height = y_bottom.saturating_sub(y_top).max(1);

        let region = SubtitleRegion {
            x: 0,
            y: y_top,
            width: metadata.width,
            height: band_height,
            confidence,
        };

        let segments = build_subtitle_segments(&region, confidence);

        let result = SubtitleDetectionResult {
            regions: vec![region],
            confidence,
            detected: confidence >= 0.5,
            sampled_frame_count: frame_count,
            segments,
        };

        LoggingService::append_video_log(
            job,
            video,
            &format!(
                "subtitle-detection: videoId={}; input={input_video_path}; confidence={:.2}; regionY={y_top}; regionH={band_height}; segmentCount={}",
                video.video_id,
                result.confidence,
                result.segments.len(),
            ),
        )?;

        Ok(result)
    }
}

fn detect_logo_inner(
    ffmpeg_path: &Path,
    input_video_path: &str,
    metadata: &VideoMetadata,
    preferred_corner: Corner,
    logo_width: u32,
    logo_height: u32,
    sample_dir: &Path,
) -> Result<(Corner, f32, usize), String> {
    let timestamps = sample_timestamps(metadata.duration_seconds);
    let frame_count = timestamps.len();
    let mut frame_summaries = Vec::new();

    for (index, timestamp) in timestamps.iter().enumerate() {
        let frame_path = sample_dir.join(format!("frame-{index}.png"));
        extract_frame(ffmpeg_path, input_video_path, *timestamp, &frame_path)?;
        frame_summaries.push(summarize_corners(
            &frame_path,
            metadata.width,
            metadata.height,
            logo_width,
            logo_height,
        )?);
    }

    let best_corner = choose_corner(&frame_summaries, preferred_corner);
    let average_delta = average_delta_for_corner(&frame_summaries, best_corner);
    let confidence = confidence_from_delta(average_delta, best_corner == preferred_corner);

    Ok((best_corner, confidence, frame_count))
}

/// Strips path separators and other dangerous characters from a video ID before using it
/// in file/directory names. Prevents path traversal when IDs come from external manifests.
pub fn sanitize_for_path(id: &str) -> String {
    id.chars()
        .map(|ch| if ch.is_alphanumeric() || ch == '-' || ch == '_' { ch } else { '_' })
        .collect()
}

fn resolve_logo_source(video: &VideoItem, preset: Option<&Preset>) -> Result<String, String> {
    video
        .mapped_logo_path
        .clone()
        .or_else(|| preset.map(|value| value.default_logo_path.clone()))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Video `{}` chua co logo source", video.video_id))
}

fn recreate_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| format!("Khong reset duoc temp dir `{}`: {error}", path.display()))?;
    }

    fs::create_dir_all(path).map_err(|error| format!("Khong tao duoc temp dir `{}`: {error}", path.display()))
}

fn probe_video_metadata(ffprobe_path: &Path, video_path: &str) -> Result<VideoMetadata, String> {
    // Use JSON output to avoid relying on positional line order, which is not stable
    // across all container formats and FFprobe versions.
    let output = Command::new(ffprobe_path)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height:format=duration",
            "-of",
            "json",
            video_path,
        ])
        .output()
        .map_err(|error| format!("Khong the chay FFprobe: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFprobe that bai: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|error| format!("Khong parse duoc JSON tu FFprobe: {error}"))?;

    let width = json["streams"][0]["width"]
        .as_u64()
        .ok_or_else(|| "FFprobe khong tra ve width".to_string())? as u32;
    let height = json["streams"][0]["height"]
        .as_u64()
        .ok_or_else(|| "FFprobe khong tra ve height".to_string())? as u32;
    let duration_seconds = json["format"]["duration"]
        .as_str()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(VideoMetadata {
        width,
        height,
        duration_seconds,
    })
}

fn sample_timestamps(duration_seconds: f64) -> Vec<f64> {
    if duration_seconds <= 0.0 {
        return vec![0.0];
    }

    let mut timestamps = SAMPLE_PERCENTAGES
        .into_iter()
        .map(|ratio| (duration_seconds * ratio).max(0.0))
        .collect::<Vec<_>>();

    timestamps.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    timestamps.dedup_by(|left, right| (*left - *right).abs() < 0.01);
    timestamps
}

fn extract_frame(ffmpeg_path: &Path, video_path: &str, timestamp: f64, output_path: &Path) -> Result<(), String> {
    let output = Command::new(ffmpeg_path)
        .args([
            "-ss",
            &format!("{timestamp:.3}"),
            "-i",
            video_path,
            "-frames:v",
            "1",
            "-y",
            &output_path.display().to_string(),
        ])
        .output()
        .map_err(|error| format!("Khong the trich frame bang FFmpeg: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg extract frame that bai: {}", stderr.trim()));
    }

    Ok(())
}

fn summarize_corners(
    frame_path: &Path,
    video_width: u32,
    video_height: u32,
    region_width: u32,
    region_height: u32,
) -> Result<Vec<RegionSummary>, String> {
    let image = ImageReader::open(frame_path)
        .map_err(|error| format!("Khong mo duoc frame `{}`: {error}", frame_path.display()))?
        .decode()
        .map_err(|error| format!("Khong decode duoc frame `{}`: {error}", frame_path.display()))?;

    let corners = [Corner::TopLeft, Corner::TopRight, Corner::BottomRight, Corner::BottomLeft];

    Ok(corners
        .into_iter()
        .map(|corner| RegionSummary {
            corner,
            mean_rgb: mean_rgb_for_region(
                &image,
                build_bounding_box(corner, video_width, video_height, region_width, region_height),
            ),
        })
        .collect())
}

fn mean_rgb_for_region(image: &image::DynamicImage, bbox: BoundingBox) -> [f64; 3] {
    let mut totals = [0.0_f64; 3];
    let mut pixel_count = 0.0_f64;

    for y in bbox.y..bbox.y.saturating_add(bbox.height) {
        for x in bbox.x..bbox.x.saturating_add(bbox.width) {
            if x >= image.width() || y >= image.height() {
                continue;
            }

            let pixel = image.get_pixel(x, y).0;
            totals[0] += f64::from(pixel[0]);
            totals[1] += f64::from(pixel[1]);
            totals[2] += f64::from(pixel[2]);
            pixel_count += 1.0;
        }
    }

    if pixel_count == 0.0 {
        return [0.0, 0.0, 0.0];
    }

    [
        totals[0] / pixel_count,
        totals[1] / pixel_count,
        totals[2] / pixel_count,
    ]
}

fn choose_corner(frame_summaries: &[Vec<RegionSummary>], preferred_corner: Corner) -> Corner {
    if frame_summaries.is_empty() {
        // No data — fall back to the preset preferred corner with no detection evidence.
        return preferred_corner;
    }

    let corners = [Corner::TopLeft, Corner::TopRight, Corner::BottomRight, Corner::BottomLeft];
    let mut best_corner = preferred_corner;
    let mut best_delta = f64::MAX;

    for corner in corners {
        let delta = average_delta_for_corner(frame_summaries, corner);
        if delta < best_delta - 1.0 || ((delta - best_delta).abs() < 1.0 && corner == preferred_corner) {
            best_corner = corner;
            best_delta = delta;
        }
    }

    best_corner
}

fn average_delta_for_corner(frame_summaries: &[Vec<RegionSummary>], corner: Corner) -> f64 {
    let means = frame_summaries
        .iter()
        .filter_map(|frame| frame.iter().find(|summary| summary.corner == corner))
        .map(|summary| summary.mean_rgb)
        .collect::<Vec<_>>();

    // < 2 frames means no pairwise comparison is possible — treat as maximum instability
    // so the confidence formula maps this to the minimum confidence, not maximum.
    if means.len() < 2 {
        return DELTA_NO_DATA;
    }

    let mut total_delta = 0.0;
    let mut comparisons = 0.0;
    for pair in means.windows(2) {
        let left = pair[0];
        let right = pair[1];
        total_delta += ((left[0] - right[0]).abs() + (left[1] - right[1]).abs() + (left[2] - right[2]).abs()) / 3.0;
        comparisons += 1.0;
    }

    total_delta / comparisons
}

fn confidence_from_delta(delta: f64, preferred_corner_matched: bool) -> f32 {
    let normalized = (1.0 - (delta / MAX_MEANINGFUL_RGB_DELTA).clamp(0.0, 1.0)) as f32;
    let bonus = if preferred_corner_matched { 0.12 } else { 0.0 };
    (normalized * 0.82 + bonus).clamp(0.18, 0.98)
}

fn preferred_corner_from_rules(preset: Option<&Preset>) -> Corner {
    let rules = preset
        .map(|value| value.layout_rules.to_ascii_lowercase())
        .unwrap_or_default();

    if rules.contains("top-left") {
        Corner::TopLeft
    } else if rules.contains("bottom-right") {
        Corner::BottomRight
    } else if rules.contains("bottom-left") {
        Corner::BottomLeft
    } else {
        Corner::TopRight
    }
}

fn estimate_logo_size(logo_path: &str, metadata: &VideoMetadata) -> (u32, u32) {
    let default_width = ((metadata.width as f32) * 0.18).round() as u32;
    let default_height = ((metadata.height as f32) * 0.12).round() as u32;

    if let Ok((source_width, source_height)) = image::image_dimensions(logo_path) {
        let capped_width = default_width.clamp(48, (metadata.width / 3).max(48));
        let aspect_ratio = if source_width == 0 {
            1.0
        } else {
            source_height as f32 / source_width as f32
        };
        let computed_height = ((capped_width as f32) * aspect_ratio).round() as u32;
        return (
            capped_width,
            computed_height.clamp(32, (metadata.height / 3).max(32)),
        );
    }

    (default_width.max(48), default_height.max(32))
}

pub fn build_bounding_box(corner: Corner, video_width: u32, video_height: u32, width: u32, height: u32) -> BoundingBox {
    let margin_x = ((video_width as f32) * 0.04).round() as u32;
    let margin_y = ((video_height as f32) * 0.04).round() as u32;

    let x = match corner {
        Corner::TopLeft | Corner::BottomLeft => margin_x,
        Corner::TopRight | Corner::BottomRight => video_width.saturating_sub(width + margin_x),
    };
    let y = match corner {
        Corner::TopLeft | Corner::TopRight => margin_y,
        Corner::BottomLeft | Corner::BottomRight => video_height.saturating_sub(height + margin_y),
    };

    // Clamp so the logo region never exceeds the video frame.
    let clamped_width = width.max(1).min(video_width.saturating_sub(x));
    let clamped_height = height.max(1).min(video_height.saturating_sub(y));

    BoundingBox {
        x,
        y,
        width: clamped_width,
        height: clamped_height,
    }
}

fn detect_subtitle_inner(
    ffmpeg_path: &Path,
    input_video_path: &str,
    metadata: &VideoMetadata,
    sample_dir: &Path,
) -> Result<(f32, usize), String> {
    let timestamps = sample_timestamps(metadata.duration_seconds);
    let frame_count = timestamps.len();

    let y_top = ((metadata.height as f64) * SUBTITLE_BAND_TOP_RATIO) as u32;
    let band_height = (((metadata.height as f64) * (SUBTITLE_BAND_BOTTOM_RATIO - SUBTITLE_BAND_TOP_RATIO)) as u32).max(1);

    let mut band_means: Vec<f64> = Vec::new();

    for (index, timestamp) in timestamps.iter().enumerate() {
        let frame_path = sample_dir.join(format!("frame-{index}.png"));
        extract_frame(ffmpeg_path, input_video_path, *timestamp, &frame_path)?;

        let image = ImageReader::open(&frame_path)
            .map_err(|error| format!("Khong mo duoc frame: {error}"))?
            .decode()
            .map_err(|error| format!("Khong decode duoc frame: {error}"))?;

        // Compute mean brightness in the subtitle band (Y channel approximation).
        let mean = mean_brightness_in_band(&image, y_top, band_height);
        band_means.push(mean);
    }

    let confidence = confidence_from_band_variance(&band_means);
    Ok((confidence, frame_count))
}

/// Computes mean luminance of pixels in a horizontal band of the image.
fn mean_brightness_in_band(image: &image::DynamicImage, y_start: u32, height: u32) -> f64 {
    let mut total = 0.0_f64;
    let mut count = 0.0_f64;

    for y in y_start..y_start.saturating_add(height) {
        if y >= image.height() {
            break;
        }
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y).0;
            // Rec.601 luma approximation
            total += 0.299 * f64::from(pixel[0])
                + 0.587 * f64::from(pixel[1])
                + 0.114 * f64::from(pixel[2]);
            count += 1.0;
        }
    }

    if count == 0.0 { 0.0 } else { total / count }
}

/// Maps cross-frame brightness variance to a detection confidence value.
/// High variance → bright/dark alternation → subtitle text changing between frames.
fn confidence_from_band_variance(means: &[f64]) -> f32 {
    if means.len() < 2 {
        return 0.18; // insufficient data → minimum confidence
    }

    let avg = means.iter().sum::<f64>() / means.len() as f64;
    let variance = means.iter().map(|m| (m - avg).powi(2)).sum::<f64>() / means.len() as f64;
    let std_dev = variance.sqrt();

    // Normalize: std_dev of 0 → 0.18, std_dev >= THRESHOLD → 0.92
    let normalized = (std_dev / SUBTITLE_VARIANCE_THRESHOLD).clamp(0.0, 1.0);
    let confidence = 0.18 + normalized as f32 * (0.92 - 0.18);
    confidence.clamp(0.18, 0.92)
}

fn build_subtitle_segments(region: &SubtitleRegion, confidence: f32) -> Vec<SubtitleSegment> {
    // Low confidence: cannot reliably identify the subtitle region.
    if confidence < 0.5 {
        return vec![SubtitleSegment {
            issue_type: "SubtitleRegion".to_string(),
            risk_level: "High".to_string(),
            start_ms: 0,
            end_ms: None,
            confidence,
            message: "Subtitle region detect voi confidence thap; can review thu cong".to_string(),
            region: region.clone(),
        }];
    }

    // High confidence but region is abnormally tall (> 20% of video height).
    // This can happen when background scenes have high brightness variation.
    let video_height_estimate = (region.y as f64 / SUBTITLE_BAND_TOP_RATIO) as u32;
    let region_ratio = region.height as f64 / video_height_estimate.max(1) as f64;
    if region_ratio > 0.20 {
        return vec![SubtitleSegment {
            issue_type: "SubtitleRegion".to_string(),
            risk_level: "Medium".to_string(),
            start_ms: 0,
            end_ms: None,
            confidence,
            message: "Subtitle region lon bat thuong; co the bi blur nhieu hon du kien".to_string(),
            region: region.clone(),
        }];
    }

    Vec::new()
}

fn build_segments(bounding_box: &BoundingBox, confidence: f32, risk_level: &str) -> Vec<LogoSegment> {
    if risk_level == "Low" {
        return Vec::new();
    }

    vec![LogoSegment {
        issue_type: "LogoPosition".to_string(),
        risk_level: risk_level.to_string(),
        start_ms: 0,
        end_ms: None,
        confidence,
        message: "Logo detect chua du on dinh; can review bounding box".to_string(),
        bounding_box: bounding_box.clone(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_top_right_when_rules_are_missing() {
        let preset = Preset {
            preset_id: "preset-1".to_string(),
            brand_name: "Brand".to_string(),
            default_logo_path: "logo.png".to_string(),
            audio_replacement_policy: "ReplaceAll".to_string(),
            subtitle_style_preset: "style".to_string(),
            layout_rules: "safe zone generic".to_string(),
            export_preset: "export".to_string(),
            notes: "notes".to_string(),
        };

        assert_eq!(preferred_corner_from_rules(Some(&preset)).as_str(), "top-right");
    }

    #[test]
    fn creates_review_segment_only_for_non_low_risk() {
        let bbox = BoundingBox {
            x: 10,
            y: 20,
            width: 200,
            height: 80,
        };

        assert_eq!(build_segments(&bbox, 0.92, "Low").len(), 0);
        assert_eq!(build_segments(&bbox, 0.62, "Medium").len(), 1);
    }

    #[test]
    fn single_frame_returns_no_data_delta() {
        let summaries = vec![vec![
            RegionSummary { corner: Corner::TopRight, mean_rgb: [100.0, 100.0, 100.0] },
        ]];
        let delta = average_delta_for_corner(&summaries, Corner::TopRight);
        assert_eq!(delta, DELTA_NO_DATA);
    }

    #[test]
    fn sanitize_strips_path_separators() {
        assert_eq!(sanitize_for_path("video/bad\\id.."), "video_bad_id__");
        assert_eq!(sanitize_for_path("ok-video_01"), "ok-video_01");
    }

    #[test]
    fn bounding_box_clamped_within_video_dimensions() {
        // Logo wider than remaining space should be clamped
        let bbox = build_bounding_box(Corner::TopRight, 1920, 1080, 600, 200);
        assert!(bbox.x + bbox.width <= 1920);
        assert!(bbox.y + bbox.height <= 1080);
    }

    // ─── Subtitle detection tests ─────────────────────────────────────────────

    #[test]
    fn low_variance_means_low_subtitle_confidence() {
        // Stable brightness (no subtitle change) → near minimum confidence
        let means = vec![100.0, 101.0, 100.5, 99.8, 100.2];
        let confidence = confidence_from_band_variance(&means);
        assert!(confidence < 0.5, "expected low confidence for stable band, got {confidence}");
    }

    #[test]
    fn high_variance_means_high_subtitle_confidence() {
        // Large brightness swings (subtitle appears/disappears) → high confidence
        let means = vec![80.0, 150.0, 90.0, 160.0, 85.0];
        let confidence = confidence_from_band_variance(&means);
        assert!(confidence >= 0.5, "expected high confidence for varying band, got {confidence}");
    }

    #[test]
    fn single_mean_returns_minimum_confidence() {
        let means = vec![120.0];
        let confidence = confidence_from_band_variance(&means);
        assert_eq!(confidence, 0.18);
    }

    #[test]
    fn low_confidence_detection_produces_high_risk_segment() {
        let region = SubtitleRegion { x: 0, y: 810, width: 1920, height: 216, confidence: 0.25 };
        let segments = build_subtitle_segments(&region, 0.25);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].risk_level, "High");
        assert_eq!(segments[0].issue_type, "SubtitleRegion");
    }

    #[test]
    fn high_confidence_normal_region_produces_no_segment() {
        // 75% top ratio → 1080-height video → y_top ≈ 810, band ≈ 216 (20% of 1080)
        let region = SubtitleRegion { x: 0, y: 810, width: 1920, height: 200, confidence: 0.85 };
        let segments = build_subtitle_segments(&region, 0.85);
        assert_eq!(segments.len(), 0);
    }
}
