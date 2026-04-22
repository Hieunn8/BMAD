use serde::Serialize;
use tauri::AppHandle;

use crate::services::export_service::{
    ExportReadiness, ExportService, JobExportSummaryReport, StartExportResult, VideoReport,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReadinessResponse {
    pub result: ExportReadiness,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExportOutputFolderResponse {
    pub job: crate::domain::job::Job,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartExportResponse {
    pub job: crate::domain::job::Job,
    pub started: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoReportResponse {
    pub report: VideoReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobExportSummaryReportResponse {
    pub summary: JobExportSummaryReport,
}

#[tauri::command]
pub fn get_export_readiness(
    app_handle: AppHandle,
    job_id: String,
) -> Result<ExportReadinessResponse, String> {
    Ok(ExportReadinessResponse {
        result: ExportService::get_export_readiness(&app_handle, &job_id)?,
    })
}

#[tauri::command]
pub fn set_export_output_folder(
    app_handle: AppHandle,
    job_id: String,
    output_folder: String,
) -> Result<SetExportOutputFolderResponse, String> {
    Ok(SetExportOutputFolderResponse {
        job: ExportService::set_export_output_folder(&app_handle, &job_id, &output_folder)?,
    })
}

#[tauri::command]
pub fn start_export(app_handle: AppHandle, job_id: String) -> Result<StartExportResponse, String> {
    let StartExportResult { job, started } = ExportService::start_export(&app_handle, &job_id)?;
    Ok(StartExportResponse { job, started })
}

#[tauri::command]
pub fn generate_video_report(
    app_handle: AppHandle,
    job_id: String,
    video_id: String,
    regenerate: Option<bool>,
) -> Result<VideoReportResponse, String> {
    Ok(VideoReportResponse {
        report: ExportService::generate_video_report(&app_handle, &job_id, &video_id, regenerate.unwrap_or(false))?,
    })
}

#[tauri::command]
pub fn get_report(app_handle: AppHandle, job_id: String, video_id: String) -> Result<VideoReportResponse, String> {
    Ok(VideoReportResponse {
        report: ExportService::get_report(&app_handle, &job_id, &video_id)?,
    })
}

#[tauri::command]
pub fn get_job_export_summary_report(
    app_handle: AppHandle,
    job_id: String,
) -> Result<JobExportSummaryReportResponse, String> {
    Ok(JobExportSummaryReportResponse {
        summary: ExportService::get_job_summary_report(&app_handle, &job_id)?,
    })
}
