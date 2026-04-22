use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    constants::{JOB_MANIFEST, SEGMENT_STATE_DIR},
    domain::{job::Job, preset::Preset, video_item::VideoItem},
    services::{persistence_service::PersistenceService, preset_service::PresetService},
};

const STATUS_MATCHED: &str = "Matched";
const STATUS_MISSING: &str = "Missing";
const STATUS_NEEDS_REVIEW: &str = "NeedsReview";
const STATUS_INPUT_NEEDS_REVIEW: &str = "Input Needs Review";
const STATUS_READY: &str = "Ready";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileRole {
    Video,
    Logo,
    Audio,
    Srt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedFile {
    pub file_name: String,
    pub path: String,
    pub role: FileRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RejectedFile {
    pub file_name: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassifiedImportResult {
    pub accepted_files: Vec<AcceptedFile>,
    pub rejected_files: Vec<RejectedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MappingOption {
    pub file_name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MappingFieldState {
    pub current_path: Option<String>,
    pub options: Vec<MappingOption>,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MappingRow {
    pub video_id: String,
    pub video_name: String,
    pub task: Option<String>,
    pub preset_name: Option<String>,
    pub logo: MappingFieldState,
    pub audio: MappingFieldState,
    pub srt: MappingFieldState,
    pub status: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MappingField {
    Logo,
    Audio,
    Srt,
}

#[derive(Debug, Default)]
pub struct MappingService;

impl MappingService {
    pub fn classify_files(file_paths: &[String]) -> ClassifiedImportResult {
        let mut accepted_files = Vec::new();
        let mut rejected_files = Vec::new();

        for path in file_paths {
            let file_name = Path::new(path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(path)
                .to_string();

            match classify_path(path) {
                Some(role) => accepted_files.push(AcceptedFile {
                    file_name,
                    path: path.clone(),
                    role,
                }),
                None => rejected_files.push(RejectedFile {
                    file_name,
                    path: path.clone(),
                    reason: "File khong duoc ho tro".to_string(),
                }),
            }
        }

        ClassifiedImportResult {
            accepted_files,
            rejected_files,
        }
    }

    pub fn auto_map_job(app_handle: &AppHandle, job_id: &str) -> Result<(Job, Vec<MappingRow>), String> {
        let mut job = load_job(app_handle, job_id)?;
        let preset = load_preset(app_handle, job.preset_id.as_deref())?;

        apply_exact_matches(&mut job, preset.as_ref());
        PersistenceService::persist_job(app_handle, &job)?;

        let rows = build_mapping_rows(&job, preset.as_ref());
        Ok((job, rows))
    }

    pub fn fix_mapping(
        app_handle: &AppHandle,
        job_id: &str,
        video_id: &str,
        field: MappingField,
        file_path: String,
    ) -> Result<(Job, Vec<MappingRow>, Option<String>), String> {
        let mut job = load_job(app_handle, job_id)?;
        let preset = load_preset(app_handle, job.preset_id.as_deref())?;
        let review_warning = replacement_warning(app_handle, job_id, video_id, field, &file_path, &job)?;
        let replacement = validate_mapping_option(&job, preset.as_ref(), field, &file_path)?;

        let requirements = required_inputs(job.selected_task.as_deref());
        let video_item = job
            .video_items
            .iter_mut()
            .find(|item| item.video_id == video_id)
            .ok_or_else(|| format!("Khong tim thay video `{video_id}` trong job"))?;

        match field {
            MappingField::Logo => video_item.mapped_logo_path = Some(replacement),
            MappingField::Audio => video_item.mapped_audio_path = Some(replacement),
            MappingField::Srt => video_item.mapped_srt_path = Some(replacement),
        }

        refresh_video_status(video_item, requirements);
        PersistenceService::persist_job(app_handle, &job)?;

        let rows = build_mapping_rows(&job, preset.as_ref());
        Ok((job, rows, review_warning))
    }
}

fn classify_path(path: &str) -> Option<FileRole> {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())?;

    match extension.as_str() {
        "mp4" | "mov" | "mkv" | "avi" | "webm" => Some(FileRole::Video),
        "png" | "jpg" | "jpeg" | "svg" => Some(FileRole::Logo),
        "mp3" | "aac" | "wav" | "m4a" => Some(FileRole::Audio),
        "srt" => Some(FileRole::Srt),
        _ => None,
    }
}

fn load_job(app_handle: &AppHandle, job_id: &str) -> Result<Job, String> {
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

fn apply_exact_matches(job: &mut Job, preset: Option<&Preset>) {
    let audio_files = files_by_role(&job.imported_files, FileRole::Audio);
    let srt_files = files_by_role(&job.imported_files, FileRole::Srt);
    let requirements = required_inputs(job.selected_task.as_deref());

    for video_item in &mut job.video_items {
        if let Some(active_preset) = preset {
            if video_item.mapped_logo_path.is_none() {
                video_item.mapped_logo_path = Some(active_preset.default_logo_path.clone());
            }
        }

        if video_item.mapped_audio_path.is_none() {
            let matches = exact_matches(&video_item.source_path, &audio_files);
            if matches.len() == 1 {
                video_item.mapped_audio_path = Some(matches[0].path.clone());
            }
        }

        if video_item.mapped_srt_path.is_none() {
            let matches = exact_matches(&video_item.source_path, &srt_files);
            if matches.len() == 1 {
                video_item.mapped_srt_path = Some(matches[0].path.clone());
            }
        }

        refresh_video_status(video_item, requirements);
    }
}

fn build_mapping_rows(job: &Job, preset: Option<&Preset>) -> Vec<MappingRow> {
    let audio_files = files_by_role(&job.imported_files, FileRole::Audio);
    let srt_files = files_by_role(&job.imported_files, FileRole::Srt);
    let logo_files = files_by_role(&job.imported_files, FileRole::Logo);
    let requirements = required_inputs(job.selected_task.as_deref());

    job.video_items
        .iter()
        .map(|video_item| {
            let audio_matches = exact_matches(&video_item.source_path, &audio_files);
            let srt_matches = exact_matches(&video_item.source_path, &srt_files);

            let logo = build_logo_state(video_item, preset, &logo_files);
            let audio = build_field_state(
                video_item.mapped_audio_path.as_deref(),
                &audio_files,
                &audio_matches,
                requirements.audio_required,
                "audio",
            );
            let srt = build_field_state(
                video_item.mapped_srt_path.as_deref(),
                &srt_files,
                &srt_matches,
                requirements.srt_required,
                "SRT",
            );

            let status = if audio.status == STATUS_NEEDS_REVIEW
                || srt.status == STATUS_NEEDS_REVIEW
                || (requirements.audio_required && audio.status == STATUS_MISSING)
                || (requirements.srt_required && srt.status == STATUS_MISSING)
            {
                STATUS_INPUT_NEEDS_REVIEW.to_string()
            } else {
                STATUS_READY.to_string()
            };

            MappingRow {
                video_id: video_item.video_id.clone(),
                video_name: file_name_from_path(&video_item.source_path),
                task: job.selected_task.clone(),
                preset_name: preset.map(|value| value.brand_name.clone()),
                logo,
                audio,
                srt,
                status,
            }
        })
        .collect()
}

fn build_logo_state(video_item: &VideoItem, preset: Option<&Preset>, logo_files: &[AcceptedFile]) -> MappingFieldState {
    let mut options = Vec::new();

    if let Some(active_preset) = preset {
        options.push(MappingOption {
            file_name: format!("Preset mac dinh ({})", active_preset.brand_name),
            path: active_preset.default_logo_path.clone(),
        });
    }

    options.extend(logo_files.iter().map(|file| MappingOption {
        file_name: file.file_name.clone(),
        path: file.path.clone(),
    }));

    let current_path = video_item
        .mapped_logo_path
        .clone()
        .or_else(|| preset.map(|value| value.default_logo_path.clone()));

    let message = if let Some(active_preset) = preset {
        if current_path.as_deref() == Some(active_preset.default_logo_path.as_str()) {
            "Lay logo mac dinh tu preset".to_string()
        } else {
            "Da chon logo cho video".to_string()
        }
    } else if current_path.is_some() {
        "Da chon logo cho video".to_string()
    } else {
        "Chua co logo mac dinh".to_string()
    };

    MappingFieldState {
        current_path,
        options,
        status: if preset.is_some() {
            STATUS_MATCHED.to_string()
        } else {
            STATUS_MISSING.to_string()
        },
        message,
    }
}

fn build_field_state(
    current_path: Option<&str>,
    options: &[AcceptedFile],
    exact_matches: &[AcceptedFile],
    required: bool,
    label: &str,
) -> MappingFieldState {
    let options = options
        .iter()
        .map(|file| MappingOption {
            file_name: file.file_name.clone(),
            path: file.path.clone(),
        })
        .collect::<Vec<_>>();

    if let Some(current_value) = current_path {
        return MappingFieldState {
            current_path: Some(current_value.to_string()),
            options,
            status: STATUS_MATCHED.to_string(),
            message: if exact_matches.iter().any(|file| file.path == current_value) {
                "Da khop tu dong".to_string()
            } else {
                format!("Da chon {label} thu cong")
            },
        };
    }

    if exact_matches.len() > 1 {
        return MappingFieldState {
            current_path: None,
            options,
            status: STATUS_NEEDS_REVIEW.to_string(),
            message: format!("Co nhieu file co the phu hop - can ban chon dung {label}"),
        };
    }

    MappingFieldState {
        current_path: None,
        options,
        status: STATUS_MISSING.to_string(),
        message: if required {
            format!("Chua tim thay {label} khop")
        } else {
            format!("Khong bat buoc {label} cho task nay")
        },
    }
}

fn files_by_role(files: &[AcceptedFile], role: FileRole) -> Vec<AcceptedFile> {
    files.iter().filter(|file| file.role == role).cloned().collect()
}

fn exact_matches(video_path: &str, candidates: &[AcceptedFile]) -> Vec<AcceptedFile> {
    let Some(video_stem) = lower_file_stem(video_path) else {
        return Vec::new();
    };

    candidates
        .iter()
        .filter(|candidate| lower_file_stem(&candidate.path).as_deref() == Some(video_stem.as_str()))
        .cloned()
        .collect()
}

fn lower_file_stem(path: &str) -> Option<String> {
    Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
}

fn file_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_string()
}

#[derive(Clone, Copy)]
struct RequiredInputs {
    audio_required: bool,
    srt_required: bool,
}

fn required_inputs(selected_task: Option<&str>) -> RequiredInputs {
    match selected_task {
        Some("replace-audio") => RequiredInputs {
            audio_required: true,
            srt_required: false,
        },
        Some("replace-subtitle") => RequiredInputs {
            audio_required: false,
            srt_required: true,
        },
        Some("replace-all") => RequiredInputs {
            audio_required: true,
            srt_required: true,
        },
        _ => RequiredInputs {
            audio_required: false,
            srt_required: false,
        },
    }
}

fn refresh_video_status(video_item: &mut VideoItem, requirements: RequiredInputs) {
    let needs_audio = requirements.audio_required && video_item.mapped_audio_path.is_none();
    let needs_srt = requirements.srt_required && video_item.mapped_srt_path.is_none();

    video_item.status = if needs_audio || needs_srt {
        STATUS_INPUT_NEEDS_REVIEW.to_string()
    } else {
        STATUS_MATCHED.to_string()
    };
}

fn validate_mapping_option(
    job: &Job,
    preset: Option<&Preset>,
    field: MappingField,
    file_path: &str,
) -> Result<String, String> {
    if matches!(field, MappingField::Logo) {
        if let Some(active_preset) = preset {
            if active_preset.default_logo_path == file_path {
                return Ok(file_path.to_string());
            }
        }
    }

    let expected_role = match field {
        MappingField::Logo => FileRole::Logo,
        MappingField::Audio => FileRole::Audio,
        MappingField::Srt => FileRole::Srt,
    };

    job.imported_files
        .iter()
        .find(|file| file.role == expected_role && file.path == file_path)
        .map(|file| file.path.clone())
        .ok_or_else(|| format!("File duoc chon khong hop le cho truong mapping {:?}", field))
}

fn replacement_warning(
    app_handle: &AppHandle,
    job_id: &str,
    video_id: &str,
    field: MappingField,
    next_path: &str,
    job: &Job,
) -> Result<Option<String>, String> {
    let video_item = job
        .video_items
        .iter()
        .find(|item| item.video_id == video_id)
        .ok_or_else(|| format!("Khong tim thay video `{video_id}` trong job"))?;

    let current_path = match field {
        MappingField::Logo => video_item.mapped_logo_path.as_deref(),
        MappingField::Audio => video_item.mapped_audio_path.as_deref(),
        MappingField::Srt => video_item.mapped_srt_path.as_deref(),
    };

    if current_path == Some(next_path) {
        return Ok(None);
    }

    let job_dir = PersistenceService::job_directory_path(app_handle, job_id)?;
    if segment_state_file(&job_dir, video_id).exists() {
        return Ok(Some(
            "Ban vua thay input sau khi da co review data; can kiem tra lai ket qua review.".to_string(),
        ));
    }

    Ok(None)
}

pub fn segment_state_file(job_directory: &Path, video_id: &str) -> PathBuf {
    job_directory.join(SEGMENT_STATE_DIR).join(format!("{video_id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_extensions() {
        let result = MappingService::classify_files(&[
            "C:/video/demo.mp4".to_string(),
            "C:/logo/brand.png".to_string(),
            "C:/audio/voiceover.m4a".to_string(),
            "C:/subtitle/captions.srt".to_string(),
        ]);

        assert_eq!(result.accepted_files.len(), 4);
        assert_eq!(result.accepted_files[0].role, FileRole::Video);
        assert_eq!(result.accepted_files[1].role, FileRole::Logo);
        assert_eq!(result.accepted_files[2].role, FileRole::Audio);
        assert_eq!(result.accepted_files[3].role, FileRole::Srt);
        assert!(result.rejected_files.is_empty());
    }

    #[test]
    fn rejects_unknown_extensions() {
        let result = MappingService::classify_files(&["C:/misc/notes.xyz".to_string()]);

        assert!(result.accepted_files.is_empty());
        assert_eq!(result.rejected_files.len(), 1);
        assert_eq!(result.rejected_files[0].reason, "File khong duoc ho tro");
    }

    #[test]
    fn exact_base_filename_match_is_case_insensitive() {
        let matches = exact_matches(
            "C:/video/VIDEO_01.mp4",
            &[AcceptedFile {
                file_name: "video_01.MP3".to_string(),
                path: "C:/audio/video_01.MP3".to_string(),
                role: FileRole::Audio,
            }],
        );

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "C:/audio/video_01.MP3");
    }

    #[test]
    fn build_field_state_marks_multiple_matches_as_needing_review() {
        let options = vec![
            AcceptedFile {
                file_name: "clip_01.mp3".to_string(),
                path: "C:/audio/clip_01.mp3".to_string(),
                role: FileRole::Audio,
            },
            AcceptedFile {
                file_name: "CLIP_01.wav".to_string(),
                path: "C:/audio/CLIP_01.wav".to_string(),
                role: FileRole::Audio,
            },
        ];

        let field = build_field_state(None, &options, &options, true, "audio");

        assert_eq!(field.status, STATUS_NEEDS_REVIEW);
    }

    #[test]
    fn apply_exact_matches_preserves_manual_logo_override() {
        let mut job = Job {
            job_id: "job-123".to_string(),
            created_at: "2026-04-21T00:00:00Z".to_string(),
            selected_task: Some("replace-logo".to_string()),
            preset_id: Some("preset-1".to_string()),
            output_folder: "D:/jobs/job-123".to_string(),
            export_output_folder: None,
            status: "Draft".to_string(),
            video_items: vec![VideoItem {
                video_id: "video-1".to_string(),
                source_path: "D:/video/clip.mp4".to_string(),
                source_metadata: None,
                mapped_logo_path: Some("D:/logos/custom.png".to_string()),
                mapped_audio_path: None,
                mapped_srt_path: None,
                status: "Imported".to_string(),
            }],
            imported_files: Vec::new(),
        };

        let preset = Preset {
            preset_id: "preset-1".to_string(),
            brand_name: "Brand".to_string(),
            default_logo_path: "D:/logos/default.png".to_string(),
            audio_replacement_policy: "policy".to_string(),
            subtitle_style_preset: "style".to_string(),
            layout_rules: "rules".to_string(),
            export_preset: "export".to_string(),
            notes: "notes".to_string(),
        };

        apply_exact_matches(&mut job, Some(&preset));

        assert_eq!(
            job.video_items[0].mapped_logo_path.as_deref(),
            Some("D:/logos/custom.png")
        );
    }
}
