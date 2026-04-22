use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    constants::{APP_DATA_ROOT_DIR, JOB_MANIFEST, SEGMENT_STATE_DIR, VIDEO_STATE_DIR},
    domain::job::Job,
    services::analysis_service::sanitize_for_path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedEncodeSummary {
    pub codec: String,
    pub crf: String,
    pub output_size_mb: f64,
    pub duration_seconds: f64,
    pub bitrate_kbps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProcessingState {
    pub video_id: String,
    pub status: String,
    pub current_step: String,
    pub completed_steps: Vec<String>,
    pub timestamp: String,
    #[serde(default)]
    pub audio_replacement_applied: Option<bool>,
    #[serde(default)]
    pub audio_source_path: Option<String>,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub encode_summary: Option<PersistedEncodeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentStateFile {
    pub file_name: String,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct PersistedJobListing {
    pub job: Job,
    pub last_modified: String,
}

#[derive(Debug, Clone)]
pub struct PersistedJobState {
    pub job: Job,
    pub video_states: Vec<VideoProcessingState>,
    pub segment_files: Vec<SegmentStateFile>,
    pub last_modified: String,
}

#[derive(Debug, Default)]
pub struct PersistenceService;

impl PersistenceService {
    pub fn persist_job(app_handle: &AppHandle, job: &Job) -> Result<PathBuf, String> {
        let job_dir = Self::job_directory_path(app_handle, &job.job_id)?;
        persist_job_to_directory(&job_dir, job)
    }

    pub fn job_directory_path(app_handle: &AppHandle, job_id: &str) -> Result<PathBuf, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|error| format!("Không lấy được app data dir: {error}"))?;

        Ok(job_directory_path_from_root(&app_data_dir, job_id))
    }

    pub fn list_jobs(app_handle: &AppHandle) -> Result<Vec<PersistedJobListing>, String> {
        let jobs_root = jobs_root_path(app_handle)?;
        if !jobs_root.exists() {
            return Ok(Vec::new());
        }

        let mut jobs = fs::read_dir(&jobs_root)
            .map_err(|error| format!("Khong doc duoc jobs dir: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() && path.join(JOB_MANIFEST).exists())
            .map(|job_dir| read_job_listing(&job_dir))
            .collect::<Result<Vec<_>, _>>()?;

        jobs.sort_by(|left, right| right.last_modified.cmp(&left.last_modified));
        Ok(jobs)
    }

    pub fn load_job_state(app_handle: &AppHandle, job_id: &str) -> Result<PersistedJobState, String> {
        let job_dir = Self::job_directory_path(app_handle, job_id)?;
        read_job_state(&job_dir)
    }

    /// Writes `{job.output_folder}/videos/{videoId}.json` with the current per-video
    /// processing step. Called before and after each pipeline step so the job can be
    /// resumed from the last stable checkpoint (Story 4.1).
    pub fn persist_video_state(job: &Job, state: &VideoProcessingState) -> Result<PathBuf, String> {
        let videos_dir = PathBuf::from(&job.output_folder).join(VIDEO_STATE_DIR);
        fs::create_dir_all(&videos_dir)
            .map_err(|error| format!("Khong tao duoc videos dir: {error}"))?;
        let safe_id = sanitize_for_path(&state.video_id);
        let path = videos_dir.join(format!("{safe_id}.json"));
        let json = serde_json::to_string_pretty(state)
            .map_err(|error| format!("Khong serialize duoc video state: {error}"))?;
        fs::write(&path, json)
            .map_err(|error| format!("Khong ghi duoc video state `{}`: {error}", path.display()))?;
        Ok(path)
    }
}

pub fn make_video_state(video_id: &str, status: &str, step: &str, completed: &[String]) -> VideoProcessingState {
    VideoProcessingState {
        video_id: video_id.to_string(),
        status: status.to_string(),
        current_step: step.to_string(),
        completed_steps: completed.to_vec(),
        timestamp: Utc::now().to_rfc3339(),
        audio_replacement_applied: None,
        audio_source_path: None,
        output_path: None,
        encode_summary: None,
    }
}

fn job_directory_path_from_root(root: &Path, job_id: &str) -> PathBuf {
    root.join(APP_DATA_ROOT_DIR).join(job_id)
}

fn jobs_root_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|error| format!("Khong lay duoc app data dir: {error}"))?;
    Ok(app_data_dir.join(APP_DATA_ROOT_DIR))
}

fn persist_job_to_directory(job_dir: &Path, job: &Job) -> Result<PathBuf, String> {
    fs::create_dir_all(job_dir)
        .map_err(|error| format!("Không tạo được thư mục job: {error}"))?;

    let manifest_path = job_dir.join(JOB_MANIFEST);
    let payload = serde_json::to_string_pretty(job)
        .map_err(|error| format!("Không serialize được job manifest: {error}"))?;

    fs::write(&manifest_path, payload)
        .map_err(|error| format!("Không ghi được job manifest: {error}"))?;

    Ok(manifest_path)
}

fn read_job_listing(job_dir: &Path) -> Result<PersistedJobListing, String> {
    let manifest_path = job_dir.join(JOB_MANIFEST);
    let payload = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Khong doc duoc job manifest `{}`: {error}", manifest_path.display()))?;
    let job: Job = serde_json::from_str(&payload)
        .map_err(|error| format!("Khong parse duoc job manifest `{}`: {error}", manifest_path.display()))?;
    Ok(PersistedJobListing {
        job,
        last_modified: newest_modified_timestamp(job_dir)?,
    })
}

fn read_job_state(job_dir: &Path) -> Result<PersistedJobState, String> {
    let manifest_path = job_dir.join(JOB_MANIFEST);
    let payload = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Khong doc duoc job manifest `{}`: {error}", manifest_path.display()))?;
    let job: Job = serde_json::from_str(&payload)
        .map_err(|error| format!("Khong parse duoc job manifest `{}`: {error}", manifest_path.display()))?;
    let last_modified = newest_modified_timestamp(job_dir)?;

    let videos_dir = job_dir.join(VIDEO_STATE_DIR);
    let mut video_states = if videos_dir.exists() {
        fs::read_dir(&videos_dir)
            .map_err(|error| format!("Khong doc duoc videos dir `{}`: {error}", videos_dir.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
            .map(|path| {
                let payload = fs::read_to_string(&path)
                    .map_err(|error| format!("Khong doc duoc video state `{}`: {error}", path.display()))?;
                serde_json::from_str::<VideoProcessingState>(&payload)
                    .map_err(|error| format!("Khong parse duoc video state `{}`: {error}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    video_states.sort_by(|left, right| left.video_id.cmp(&right.video_id));

    let segments_dir = job_dir.join(SEGMENT_STATE_DIR);
    let mut segment_files = if segments_dir.exists() {
        fs::read_dir(&segments_dir)
            .map_err(|error| format!("Khong doc duoc segments dir `{}`: {error}", segments_dir.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
            .map(|path| {
                let payload = fs::read_to_string(&path)
                    .map_err(|error| format!("Khong doc duoc segment state `{}`: {error}", path.display()))?;
                let parsed = serde_json::from_str::<Value>(&payload)
                    .map_err(|error| format!("Khong parse duoc segment state `{}`: {error}", path.display()))?;
                Ok(SegmentStateFile {
                    file_name: path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    payload: parsed,
                })
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        Vec::new()
    };
    segment_files.sort_by(|left, right| left.file_name.cmp(&right.file_name));

    Ok(PersistedJobState {
        job,
        video_states,
        segment_files,
        last_modified,
    })
}

fn modified_timestamp(path: &Path) -> Result<String, String> {
    let modified = fs::metadata(path)
        .map_err(|error| format!("Khong doc duoc metadata `{}`: {error}", path.display()))?
        .modified()
        .map_err(|error| format!("Khong lay duoc modified time `{}`: {error}", path.display()))?;
    Ok(chrono::DateTime::<Utc>::from(modified).to_rfc3339())
}

fn newest_modified_timestamp(root: &Path) -> Result<String, String> {
    let mut latest = fs::metadata(root)
        .map_err(|error| format!("Khong doc duoc metadata `{}`: {error}", root.display()))?
        .modified()
        .map_err(|error| format!("Khong lay duoc modified time `{}`: {error}", root.display()))?;

    collect_newest_modified(root, &mut latest)?;

    Ok(chrono::DateTime::<Utc>::from(latest).to_rfc3339())
}

fn collect_newest_modified(path: &Path, latest: &mut std::time::SystemTime) -> Result<(), String> {
    for entry in fs::read_dir(path)
        .map_err(|error| format!("Khong doc duoc thu muc `{}`: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("Khong doc duoc entry trong `{}`: {error}", path.display()))?;
        let entry_path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| format!("Khong doc duoc metadata `{}`: {error}", entry_path.display()))?;
        let modified = metadata
            .modified()
            .map_err(|error| format!("Khong lay duoc modified time `{}`: {error}", entry_path.display()))?;

        if modified > *latest {
            *latest = modified;
        }

        if metadata.is_dir() {
            collect_newest_modified(&entry_path, latest)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mapping_service::{AcceptedFile, FileRole};

    #[test]
    fn persists_video_state_to_expected_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = Job {
            job_id: "job-v".to_string(),
            created_at: "".to_string(),
            selected_task: None,
            preset_id: None,
            output_folder: temp_dir.path().display().to_string(),
            export_output_folder: None,
            status: "Processing".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };
        let state = make_video_state("vid-1", "Processing", "replace-audio", &["replace-audio".to_string()]);
        let path = PersistenceService::persist_video_state(&job, &state).expect("persist state");

        assert!(path.exists());
        let content = fs::read_to_string(&path).expect("read state");
        assert!(content.contains("\"videoId\": \"vid-1\""));
        assert!(content.contains("\"status\": \"Processing\""));
        assert!(content.contains("\"currentStep\": \"replace-audio\""));
    }

    #[test]
    fn persist_video_state_sanitizes_video_id_in_filename() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job = Job {
            job_id: "j".to_string(),
            created_at: "".to_string(),
            selected_task: None,
            preset_id: None,
            output_folder: temp_dir.path().display().to_string(),
            export_output_folder: None,
            status: "".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };
        let state = make_video_state("bad/video\\id", "Processing", "init", &[]);
        let path = PersistenceService::persist_video_state(&job, &state).expect("persist state");
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(!filename.contains('/'));
        assert!(!filename.contains('\\'));
    }

    #[test]
    fn persists_job_manifest_to_expected_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job_dir = job_directory_path_from_root(temp_dir.path(), "job-123");
        let job = Job {
            job_id: "job-123".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some("replace-logo".to_string()),
            preset_id: None,
            output_folder: job_dir.display().to_string(),
            export_output_folder: None,
            status: "Draft".to_string(),
            video_items: Vec::new(),
            imported_files: vec![AcceptedFile {
                file_name: "demo.mp4".to_string(),
                path: "C:/demo.mp4".to_string(),
                role: FileRole::Video,
            }],
        };

        let manifest_path = persist_job_to_directory(&job_dir, &job).expect("persist job");

        assert!(manifest_path.exists());
        let payload = fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(payload.contains("\"jobId\": \"job-123\""));
        assert!(payload.contains("\"selectedTask\": \"replace-logo\""));
    }

    #[test]
    fn reads_job_state_with_video_and_segment_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job_dir = job_directory_path_from_root(temp_dir.path(), "job-123");
        let job = Job {
            job_id: "job-123".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some("replace-all".to_string()),
            preset_id: Some("preset-1".to_string()),
            output_folder: job_dir.display().to_string(),
            export_output_folder: None,
            status: "ReviewPending".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };

        persist_job_to_directory(&job_dir, &job).expect("persist job");
        fs::create_dir_all(job_dir.join(VIDEO_STATE_DIR)).expect("videos dir");
        fs::create_dir_all(job_dir.join(SEGMENT_STATE_DIR)).expect("segments dir");
        fs::write(
            job_dir.join(VIDEO_STATE_DIR).join("video-1.json"),
            serde_json::to_string_pretty(&make_video_state("video-1", "ReadyToExport", "done", &[])).unwrap(),
        )
        .expect("write video state");
        fs::write(
            job_dir.join(SEGMENT_STATE_DIR).join("video-1_logo.json"),
            r#"{"videoId":"video-1","segments":[{"id":"seg-1"}]}"#,
        )
        .expect("write segment state");

        let state = read_job_state(&job_dir).expect("read job state");

        assert_eq!(state.job.job_id, "job-123");
        assert_eq!(state.video_states.len(), 1);
        assert_eq!(state.segment_files.len(), 1);
        assert_eq!(state.segment_files[0].file_name, "video-1_logo.json");
    }

    #[test]
    fn read_job_listing_ignores_invalid_segment_payloads() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job_dir = job_directory_path_from_root(temp_dir.path(), "job-123");
        let job = Job {
            job_id: "job-123".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some("replace-all".to_string()),
            preset_id: None,
            output_folder: job_dir.display().to_string(),
            export_output_folder: None,
            status: "ReviewPending".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };

        persist_job_to_directory(&job_dir, &job).expect("persist job");
        fs::create_dir_all(job_dir.join(SEGMENT_STATE_DIR)).expect("segments dir");
        fs::write(job_dir.join(SEGMENT_STATE_DIR).join("broken.json"), "{not-json").expect("write broken segment");

        let listing = read_job_listing(&job_dir).expect("read listing");

        assert_eq!(listing.job.job_id, "job-123");
    }

    #[test]
    fn read_job_listing_uses_latest_file_activity_for_last_modified() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let job_dir = job_directory_path_from_root(temp_dir.path(), "job-123");
        let job = Job {
            job_id: "job-123".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some("replace-all".to_string()),
            preset_id: None,
            output_folder: job_dir.display().to_string(),
            export_output_folder: None,
            status: "ReviewPending".to_string(),
            video_items: Vec::new(),
            imported_files: Vec::new(),
        };

        let manifest_path = persist_job_to_directory(&job_dir, &job).expect("persist job");
        std::thread::sleep(std::time::Duration::from_millis(5));
        fs::create_dir_all(job_dir.join(SEGMENT_STATE_DIR)).expect("segments dir");
        let segment_path = job_dir.join(SEGMENT_STATE_DIR).join("video-1_logo.json");
        fs::write(&segment_path, r#"{"videoId":"video-1","segments":[]}"#).expect("write segment");

        let listing = read_job_listing(&job_dir).expect("read listing");
        let manifest_modified = modified_timestamp(&manifest_path).expect("manifest ts");
        let segment_modified = modified_timestamp(&segment_path).expect("segment ts");

        assert_eq!(listing.last_modified, segment_modified);
        assert_ne!(listing.last_modified, manifest_modified);
    }
}
