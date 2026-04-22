use std::{fs, path::PathBuf};

use serde::Serialize;

use crate::{
    constants::SEGMENT_STATE_DIR,
    domain::{job::Job, video_item::VideoItem},
    services::analysis_service::{
        sanitize_for_path, LogoDetectionResult, LogoSegment,
        SubtitleDetectionResult, SubtitleSegment,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

impl RiskLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedLogoReview {
    video_id: String,
    detection: LogoDetectionResult,
    segments: Vec<LogoSegment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSubtitleReview {
    video_id: String,
    detection: SubtitleDetectionResult,
    segments: Vec<SubtitleSegment>,
}

#[derive(Debug, Default)]
pub struct RiskService;

impl RiskService {
    pub fn from_confidence(confidence: f32) -> RiskLevel {
        if confidence < 0.5 {
            RiskLevel::High
        } else if confidence < 0.8 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    pub fn persist_logo_segments(
        job: &Job,
        video: &VideoItem,
        detection: &LogoDetectionResult,
    ) -> Result<PathBuf, String> {
        let segments_dir = PathBuf::from(&job.output_folder).join(SEGMENT_STATE_DIR);
        fs::create_dir_all(&segments_dir).map_err(|error| format!("Khong tao duoc segments dir: {error}"))?;

        // Use _logo suffix to avoid overwriting subtitle segments written by story 2.4.
        let safe_id = sanitize_for_path(&video.video_id);
        let output_path = segments_dir.join(format!("{safe_id}_logo.json"));
        let payload = PersistedLogoReview {
            video_id: video.video_id.clone(),
            detection: detection.clone(),
            segments: detection.segments.clone(),
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("Khong serialize duoc logo review data: {error}"))?;

        fs::write(&output_path, json)
            .map_err(|error| format!("Khong ghi duoc segment file `{}`: {error}", output_path.display()))?;

        Ok(output_path)
    }

    pub fn persist_subtitle_segments(
        job: &Job,
        video: &VideoItem,
        detection: &SubtitleDetectionResult,
    ) -> Result<PathBuf, String> {
        let segments_dir = PathBuf::from(&job.output_folder).join(SEGMENT_STATE_DIR);
        fs::create_dir_all(&segments_dir).map_err(|error| format!("Khong tao duoc segments dir: {error}"))?;

        let safe_id = sanitize_for_path(&video.video_id);
        let output_path = segments_dir.join(format!("{safe_id}_subtitle.json"));
        let payload = PersistedSubtitleReview {
            video_id: video.video_id.clone(),
            detection: detection.clone(),
            segments: detection.segments.clone(),
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("Khong serialize duoc subtitle review data: {error}"))?;

        fs::write(&output_path, json)
            .map_err(|error| format!("Khong ghi duoc subtitle segment file `{}`: {error}", output_path.display()))?;

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_confidence_to_expected_risk_level() {
        assert_eq!(RiskService::from_confidence(0.3), RiskLevel::High);
        assert_eq!(RiskService::from_confidence(0.65), RiskLevel::Medium);
        assert_eq!(RiskService::from_confidence(0.91), RiskLevel::Low);
    }
}
