use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;
use uuid::Uuid;

use crate::domain::{job::Job, preset::Preset, video_item::VideoItem};
use crate::services::{
    job_orchestrator::{JobOrchestrator, JobReadiness},
    mapping_service::{AcceptedFile, ClassifiedImportResult, MappingService},
    persistence_service::{PersistenceService, SegmentStateFile, VideoProcessingState},
    preset_service::PresetService,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobResponse {
    pub job: Job,
    pub manifest_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobReadinessResponse {
    pub readiness: JobReadiness,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartJobResponse {
    pub started: bool,
    pub job: Job,
    pub blockers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentJobSummary {
    pub job_id: String,
    pub created_at: String,
    pub status: String,
    pub video_count: usize,
    pub last_modified: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListJobsResponse {
    pub jobs: Vec<RecentJobSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedSegmentState {
    pub file_name: String,
    pub payload: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadJobResponse {
    pub job: Job,
    pub preset: Option<Preset>,
    pub video_states: Vec<VideoProcessingState>,
    pub segment_states: Vec<LoadedSegmentState>,
    pub warning_message: Option<String>,
    pub last_modified: String,
}

#[tauri::command]
pub fn import_assets(file_paths: Vec<String>) -> Result<ClassifiedImportResult, String> {
    if file_paths.is_empty() {
        return Err("Không có file nào được gửi để import".to_string());
    }

    Ok(MappingService::classify_files(&file_paths))
}

#[tauri::command]
pub fn create_job(
    app_handle: AppHandle,
    existing_created_at: Option<String>,
    existing_job_id: Option<String>,
    selected_task: Option<String>,
    accepted_files: Vec<AcceptedFile>,
) -> Result<CreateJobResponse, String> {
    let job_id = existing_job_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let accepted_files = dedupe_accepted_files(accepted_files);
    let output_folder = PersistenceService::job_directory_path(&app_handle, &job_id)?
        .display()
        .to_string();

    let video_items = accepted_files
        .iter()
        .filter(|file| matches!(file.role, crate::services::mapping_service::FileRole::Video))
        .map(|file| VideoItem {
            video_id: Uuid::new_v5(&Uuid::NAMESPACE_URL, file.path.as_bytes()).to_string(),
            source_path: file.path.clone(),
            source_metadata: None,
            mapped_logo_path: None,
            mapped_audio_path: None,
            mapped_srt_path: None,
            status: "Imported".to_string(),
        })
        .collect();

    let job = Job {
        job_id: job_id.clone(),
        created_at: existing_created_at.unwrap_or_else(|| Utc::now().to_rfc3339()),
        selected_task,
        preset_id: None,
        output_folder,
        export_output_folder: None,
        status: "Draft".to_string(),
        video_items,
        imported_files: accepted_files,
    };

    let manifest_path = PersistenceService::persist_job(&app_handle, &job)?;

    Ok(CreateJobResponse {
        job,
        manifest_path: manifest_path.display().to_string(),
    })
}

#[tauri::command]
pub fn get_job_readiness(
    app_handle: AppHandle,
    job_id: String,
) -> Result<JobReadinessResponse, String> {
    Ok(JobReadinessResponse {
        readiness: JobOrchestrator::validate_readiness(&app_handle, &job_id)?,
    })
}

#[tauri::command]
pub fn list_jobs(app_handle: AppHandle) -> Result<ListJobsResponse, String> {
    let jobs = PersistenceService::list_jobs(&app_handle)?
        .into_iter()
        .map(|entry| RecentJobSummary {
            job_id: entry.job.job_id,
            created_at: entry.job.created_at,
            status: entry.job.status,
            video_count: entry.job.video_items.len(),
            last_modified: entry.last_modified,
        })
        .collect();

    Ok(ListJobsResponse { jobs })
}

#[tauri::command]
pub fn load_job(app_handle: AppHandle, job_id: String) -> Result<LoadJobResponse, String> {
    let persisted = PersistenceService::load_job_state(&app_handle, &job_id)?;
    let (state, warning_message) = JobOrchestrator::restore_from_checkpoint(&app_handle, persisted)?;
    let preset = match state.job.preset_id.as_deref() {
        Some(preset_id) => Some(PresetService::get_preset(&app_handle, preset_id)?),
        None => None,
    };

    Ok(LoadJobResponse {
        job: state.job,
        preset,
        video_states: state.video_states,
        segment_states: map_segment_states(state.segment_files),
        warning_message,
        last_modified: state.last_modified,
    })
}

#[tauri::command]
pub fn start_job(app_handle: AppHandle, job_id: String) -> Result<StartJobResponse, String> {
    let (job, readiness, started) = JobOrchestrator::start_job(&app_handle, &job_id)?;

    Ok(StartJobResponse {
        started,
        job,
        blockers: detailed_blockers(&readiness),
    })
}

fn detailed_blockers(readiness: &JobReadiness) -> Vec<String> {
    let mut blockers = readiness.blockers.clone();

    for video in &readiness.videos {
        for blocker in &video.blockers {
            blockers.push(format!("Video `{}`: {}", video.video_id, blocker));
        }
    }

    blockers
}

fn map_segment_states(segment_files: Vec<SegmentStateFile>) -> Vec<LoadedSegmentState> {
    segment_files
        .into_iter()
        .map(|item| LoadedSegmentState {
            file_name: item.file_name,
            payload: item.payload,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::job_orchestrator::VideoReadiness;

    #[test]
    fn expands_job_readiness_into_actionable_blockers() {
        let blockers = detailed_blockers(&JobReadiness {
            is_ready: false,
            blockers: vec!["Job chua san sang; van con video dang bi block.".to_string()],
            videos: vec![VideoReadiness {
                video_id: "video-1".to_string(),
                is_ready: false,
                blockers: vec!["Chua co audio".to_string(), "Chua co SRT".to_string()],
            }],
        });

        assert!(blockers.iter().any(|value| value.contains("video-1")));
        assert!(blockers.iter().any(|value| value.contains("Chua co audio")));
    }
}

fn dedupe_accepted_files(accepted_files: Vec<AcceptedFile>) -> Vec<AcceptedFile> {
    let mut unique_files = Vec::new();

    for file in accepted_files {
        if unique_files
            .iter()
            .all(|existing: &AcceptedFile| existing.path != file.path)
        {
            unique_files.push(file);
        }
    }

    unique_files
}
