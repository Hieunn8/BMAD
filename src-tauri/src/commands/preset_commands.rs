use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    domain::{job::Job, preset::Preset},
    services::preset_service::{EditPresetResult, PresetService},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPresetsResponse {
    pub presets: Vec<Preset>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectPresetResponse {
    pub applied: bool,
    pub job: Option<Job>,
    pub preset: Preset,
    pub warning_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetInput {
    pub brand_name: String,
    pub default_logo_path: String,
    pub audio_replacement_policy: String,
    pub subtitle_style_preset: String,
    pub layout_rules: String,
    pub export_preset: String,
    pub notes: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePresetResponse {
    pub preset: Preset,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPresetResponse {
    pub saved: bool,
    pub preset: Option<Preset>,
    pub warning_message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicatePresetResponse {
    pub preset: Preset,
}

#[tauri::command]
pub fn list_presets(app_handle: AppHandle) -> Result<ListPresetsResponse, String> {
    Ok(ListPresetsResponse {
        presets: PresetService::list_presets(&app_handle)?,
    })
}

#[tauri::command]
pub fn create_preset(app_handle: AppHandle, preset: PresetInput) -> Result<CreatePresetResponse, String> {
    Ok(CreatePresetResponse {
        preset: PresetService::create_preset(&app_handle, to_preset(String::new(), preset))?,
    })
}

#[tauri::command]
pub fn edit_preset(
    app_handle: AppHandle,
    preset_id: String,
    preset: PresetInput,
    current_job_id: Option<String>,
    confirm_override: bool,
) -> Result<EditPresetResponse, String> {
    let EditPresetResult {
        saved,
        preset,
        warning_message,
    } = PresetService::edit_preset(
        &app_handle,
        &preset_id,
        to_preset(preset_id.clone(), preset),
        current_job_id.as_deref(),
        confirm_override,
    )?;

    Ok(EditPresetResponse {
        saved,
        preset,
        warning_message,
    })
}

#[tauri::command]
pub fn duplicate_preset(
    app_handle: AppHandle,
    preset_id: String,
) -> Result<DuplicatePresetResponse, String> {
    Ok(DuplicatePresetResponse {
        preset: PresetService::duplicate_preset(&app_handle, &preset_id)?,
    })
}

#[tauri::command]
pub fn select_preset(
    app_handle: AppHandle,
    job_id: String,
    preset_id: String,
    confirm_override: bool,
) -> Result<SelectPresetResponse, String> {
    let preset = PresetService::get_preset(&app_handle, &preset_id)?;
    let warning_message = PresetService::preset_change_warning(&app_handle, &job_id, &preset_id)?;

    if warning_message.is_some() && !confirm_override {
        return Ok(SelectPresetResponse {
            applied: false,
            job: None,
            preset,
            warning_message,
        });
    }

    let job = PresetService::apply_preset(&app_handle, &job_id, &preset)?;

    Ok(SelectPresetResponse {
        applied: true,
        job: Some(job),
        preset,
        warning_message: None,
    })
}

fn to_preset(preset_id: String, input: PresetInput) -> Preset {
    Preset {
        preset_id,
        brand_name: input.brand_name,
        default_logo_path: input.default_logo_path,
        audio_replacement_policy: input.audio_replacement_policy,
        subtitle_style_preset: input.subtitle_style_preset,
        layout_rules: input.layout_rules,
        export_preset: input.export_preset,
        notes: input.notes,
    }
}
