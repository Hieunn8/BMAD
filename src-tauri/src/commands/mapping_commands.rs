use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::{
    domain::job::Job,
    services::mapping_service::{MappingField, MappingRow, MappingService},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingUpdatedEvent {
    pub job: Job,
    pub rows: Vec<MappingRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFileReplacedAfterReviewEvent {
    pub video_id: String,
    pub field: MappingField,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingResponse {
    pub job: Job,
    pub rows: Vec<MappingRow>,
}

#[tauri::command]
pub fn auto_map_job(app_handle: AppHandle, job_id: String) -> Result<MappingResponse, String> {
    let (job, rows) = MappingService::auto_map_job(&app_handle, &job_id)?;
    emit_mapping_updated(&app_handle, job.clone(), rows.clone())?;

    Ok(MappingResponse { job, rows })
}

#[tauri::command]
pub fn fix_mapping(
    app_handle: AppHandle,
    job_id: String,
    video_id: String,
    field: MappingField,
    file_path: String,
) -> Result<MappingResponse, String> {
    let (job, rows, review_warning) =
        MappingService::fix_mapping(&app_handle, &job_id, &video_id, field, file_path)?;

    if let Some(message) = review_warning {
        app_handle
            .emit(
                "inputFileReplacedAfterReview",
                InputFileReplacedAfterReviewEvent {
                    video_id,
                    field,
                    message,
                },
            )
            .map_err(|error| format!("Khong emit duoc inputFileReplacedAfterReview: {error}"))?;
    }

    emit_mapping_updated(&app_handle, job.clone(), rows.clone())?;

    Ok(MappingResponse { job, rows })
}

fn emit_mapping_updated(app_handle: &AppHandle, job: Job, rows: Vec<MappingRow>) -> Result<(), String> {
    app_handle
        .emit("mappingUpdated", MappingUpdatedEvent { job, rows })
        .map_err(|error| format!("Khong emit duoc mappingUpdated: {error}"))
}
