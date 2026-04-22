use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    constants::{JOB_MANIFEST, PRESETS_ROOT_DIR, SEGMENT_STATE_DIR, VIDEO_STATE_DIR},
    domain::{job::Job, preset::Preset},
    services::persistence_service::PersistenceService,
};

#[derive(Debug, Clone)]
pub struct EditPresetResult {
    pub saved: bool,
    pub preset: Option<Preset>,
    pub warning_message: Option<String>,
}

#[derive(Debug, Default)]
pub struct PresetService;

impl PresetService {
    pub fn list_presets(app_handle: &AppHandle) -> Result<Vec<Preset>, String> {
        let presets_dir = presets_directory(app_handle)?;
        seed_default_presets(&presets_dir)?;
        list_presets_from_directory(&presets_dir)
    }

    pub fn get_preset(app_handle: &AppHandle, preset_id: &str) -> Result<Preset, String> {
        let presets = Self::list_presets(app_handle)?;
        presets
            .into_iter()
            .find(|preset| preset.preset_id == preset_id)
            .ok_or_else(|| format!("Khong tim thay preset `{preset_id}`"))
    }

    pub fn create_preset(app_handle: &AppHandle, mut preset: Preset) -> Result<Preset, String> {
        let presets_dir = presets_directory(app_handle)?;
        seed_default_presets(&presets_dir)?;
        validate_preset(&preset)?;

        preset.preset_id = Uuid::new_v4().to_string();
        write_preset(&presets_dir, &preset)?;

        Ok(preset)
    }

    pub fn edit_preset(
        app_handle: &AppHandle,
        preset_id: &str,
        mut next_preset: Preset,
        current_job_id: Option<&str>,
        confirm_override: bool,
    ) -> Result<EditPresetResult, String> {
        let presets_dir = presets_directory(app_handle)?;
        seed_default_presets(&presets_dir)?;
        let existing = read_preset(&preset_path(&presets_dir, preset_id))?;

        next_preset.preset_id = existing.preset_id.clone();
        validate_preset(&next_preset)?;
        let warning_message = preset_edit_warning(app_handle, current_job_id, preset_id)?;

        if warning_message.is_some() && !confirm_override {
            return Ok(EditPresetResult {
                saved: false,
                preset: None,
                warning_message,
            });
        }

        write_preset(&presets_dir, &next_preset)?;

        Ok(EditPresetResult {
            saved: true,
            preset: Some(next_preset),
            warning_message: None,
        })
    }

    pub fn duplicate_preset(app_handle: &AppHandle, preset_id: &str) -> Result<Preset, String> {
        let presets_dir = presets_directory(app_handle)?;
        seed_default_presets(&presets_dir)?;

        let mut preset = read_preset(&preset_path(&presets_dir, preset_id))?;
        preset.preset_id = Uuid::new_v4().to_string();
        preset.brand_name = format!("{} - Copy", preset.brand_name);
        write_preset(&presets_dir, &preset)?;

        Ok(preset)
    }

    pub fn apply_preset(app_handle: &AppHandle, job_id: &str, preset: &Preset) -> Result<Job, String> {
        let job_dir = PersistenceService::job_directory_path(app_handle, job_id)?;
        let manifest_path = job_dir.join(JOB_MANIFEST);
        let payload = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("Khong doc duoc job manifest: {error}"))?;
        let mut job: Job = serde_json::from_str(&payload)
            .map_err(|error| format!("Khong parse duoc job manifest: {error}"))?;

        job.preset_id = Some(preset.preset_id.clone());

        PersistenceService::persist_job(app_handle, &job)?;

        Ok(job)
    }

    pub fn preset_change_warning(
        app_handle: &AppHandle,
        job_id: &str,
        next_preset_id: &str,
    ) -> Result<Option<String>, String> {
        let job = load_job(app_handle, job_id)?;

        if job.preset_id.as_deref() == Some(next_preset_id) {
            return Ok(None);
        }

        let job_dir = PersistenceService::job_directory_path(app_handle, job_id)?;
        let has_video_state = directory_has_files(&job_dir.join(VIDEO_STATE_DIR))?;
        let has_review_data = directory_has_files(&job_dir.join(SEGMENT_STATE_DIR))?;

        if has_video_state || has_review_data {
            return Ok(Some(
                "Doi preset co the lam mat hieu luc mapping, detect, hoac cac chinh sua truoc do.".to_string(),
            ));
        }

        Ok(None)
    }
}

fn presets_directory(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|error| format!("Khong lay duoc app data dir: {error}"))?;
    let presets_dir = app_data_dir.join(PRESETS_ROOT_DIR);

    fs::create_dir_all(&presets_dir)
        .map_err(|error| format!("Khong tao duoc thu muc presets: {error}"))?;

    Ok(presets_dir)
}

fn load_job(app_handle: &AppHandle, job_id: &str) -> Result<Job, String> {
    let job_dir = PersistenceService::job_directory_path(app_handle, job_id)?;
    let manifest_path = job_dir.join(JOB_MANIFEST);
    let payload = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Khong doc duoc job manifest: {error}"))?;

    serde_json::from_str(&payload).map_err(|error| format!("Khong parse duoc job manifest: {error}"))
}

fn preset_edit_warning(
    app_handle: &AppHandle,
    current_job_id: Option<&str>,
    preset_id: &str,
) -> Result<Option<String>, String> {
    let Some(job_id) = current_job_id else {
        return Ok(None);
    };

    let job = load_job(app_handle, job_id)?;
    if job.preset_id.as_deref() != Some(preset_id) {
        return Ok(None);
    }

    let job_dir = PersistenceService::job_directory_path(app_handle, job_id)?;
    if directory_has_files(&job_dir.join(SEGMENT_STATE_DIR))? {
        return Ok(Some(
            "Thay doi preset co the anh huong den ket qua detect va quick-fix da co.".to_string(),
        ));
    }

    Ok(None)
}

fn preset_path(presets_dir: &Path, preset_id: &str) -> PathBuf {
    presets_dir.join(format!("{preset_id}.json"))
}

fn list_presets_from_directory(presets_dir: &Path) -> Result<Vec<Preset>, String> {
    let mut presets = fs::read_dir(presets_dir)
        .map_err(|error| format!("Khong doc duoc thu muc presets: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .map(|path| read_preset(&path))
        .collect::<Result<Vec<_>, _>>()?;

    presets.sort_by(|left, right| left.brand_name.cmp(&right.brand_name));
    Ok(presets)
}

fn write_preset(presets_dir: &Path, preset: &Preset) -> Result<(), String> {
    let path = preset_path(presets_dir, &preset.preset_id);
    let payload = serde_json::to_string_pretty(preset)
        .map_err(|error| format!("Khong serialize duoc preset: {error}"))?;
    fs::write(path, payload).map_err(|error| format!("Khong ghi duoc preset: {error}"))
}

fn seed_default_presets(presets_dir: &Path) -> Result<(), String> {
    let defaults = default_presets();

    for preset in defaults {
        let path = preset_path(presets_dir, &preset.preset_id);

        if path.exists() {
            continue;
        }

        write_preset(presets_dir, &preset)?;
    }

    Ok(())
}

fn default_presets() -> Vec<Preset> {
    vec![
        Preset {
            preset_id: "gastown-daily".to_string(),
            brand_name: "Gastown Daily".to_string(),
            default_logo_path: "brand/gastown-daily/logo-primary.png".to_string(),
            audio_replacement_policy: "ReplaceAll".to_string(),
            subtitle_style_preset: "Lower-third sans / cyan emphasis".to_string(),
            layout_rules: "Top-right logo safe zone, subtitle baseline 8%".to_string(),
            export_preset: "MP4 H264 CRF20".to_string(),
            notes: "Preset cho channel tin tuc hang ngay, uu tien tempo nhanh va subtitle sang ro."
                .to_string(),
        },
        Preset {
            preset_id: "boss-shortform".to_string(),
            brand_name: "BOSS Shortform".to_string(),
            default_logo_path: "brand/boss-shortform/logo-stamp.svg".to_string(),
            audio_replacement_policy: "NoReplacement".to_string(),
            subtitle_style_preset: "Bold captions / high contrast".to_string(),
            layout_rules: "Center-safe subtitles, logo stamp top-left".to_string(),
            export_preset: "MP4 H264 CRF23".to_string(),
            notes: "Preset cho video ngan, giu nhac nen va tang do doc cua subtitle.".to_string(),
        },
        Preset {
            preset_id: "studio-premium".to_string(),
            brand_name: "Studio Premium".to_string(),
            default_logo_path: "brand/studio-premium/logo-gold.png".to_string(),
            audio_replacement_policy: "NoReplacement".to_string(),
            subtitle_style_preset: "Minimal serif / premium lower third".to_string(),
            layout_rules: "Wide safe margins, premium end card export".to_string(),
            export_preset: "MP4 H264 CRF18".to_string(),
            notes: "Preset cho brand cao cap, uu tien giu framing rong va xuat master 4K.".to_string(),
        },
    ]
}

fn read_preset(path: &Path) -> Result<Preset, String> {
    let payload =
        fs::read_to_string(path).map_err(|error| format!("Khong doc duoc preset `{}`: {error}", path.display()))?;
    serde_json::from_str(&payload)
        .map_err(|error| format!("Khong parse duoc preset `{}`: {error}", path.display()))
}

fn directory_has_files(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    Ok(fs::read_dir(path)
        .map_err(|error| format!("Khong doc duoc thu muc `{}`: {error}", path.display()))?
        .filter_map(Result::ok)
        .any(|entry| entry.path().is_file()))
}

fn validate_preset(preset: &Preset) -> Result<(), String> {
    if preset.brand_name.trim().is_empty() {
        return Err("Brand Name khong duoc de trong.".to_string());
    }

    if preset.default_logo_path.trim().is_empty() {
        return Err("Default Logo khong duoc de trong.".to_string());
    }

    let extension = Path::new(&preset.default_logo_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    if !matches!(extension.as_deref(), Some("png" | "jpg" | "jpeg" | "svg")) {
        return Err("Default Logo chi ho tro file .png, .jpg, .jpeg, hoac .svg.".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_default_presets_once() {
        let temp_dir = tempfile::tempdir().expect("temp dir");

        seed_default_presets(temp_dir.path()).expect("seed defaults");
        seed_default_presets(temp_dir.path()).expect("seed defaults again");

        let entries = fs::read_dir(temp_dir.path()).expect("read presets");
        assert_eq!(entries.count(), 3);
    }

    #[test]
    fn duplicate_keeps_original_unchanged() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let original = Preset {
            preset_id: "preset-1".to_string(),
            brand_name: "Brand".to_string(),
            default_logo_path: "logo.png".to_string(),
            audio_replacement_policy: "ReplaceAll".to_string(),
            subtitle_style_preset: "style".to_string(),
            layout_rules: "rules".to_string(),
            export_preset: "MP4 H264 CRF20".to_string(),
            notes: "notes".to_string(),
        };

        write_preset(temp_dir.path(), &original).expect("write original");
        let mut duplicate = read_preset(&preset_path(temp_dir.path(), "preset-1")).expect("read original");
        duplicate.preset_id = "preset-2".to_string();
        duplicate.brand_name = format!("{} - Copy", duplicate.brand_name);
        write_preset(temp_dir.path(), &duplicate).expect("write duplicate");

        let presets = list_presets_from_directory(temp_dir.path()).expect("list presets");
        assert_eq!(presets.len(), 2);
        assert!(presets.iter().any(|preset| preset.brand_name == "Brand"));
        assert!(presets.iter().any(|preset| preset.brand_name == "Brand - Copy"));
    }

    #[test]
    fn rejects_invalid_logo_extension() {
        let preset = Preset {
            preset_id: "preset-1".to_string(),
            brand_name: "Brand".to_string(),
            default_logo_path: "logo.gif".to_string(),
            audio_replacement_policy: "ReplaceAll".to_string(),
            subtitle_style_preset: "style".to_string(),
            layout_rules: "rules".to_string(),
            export_preset: "MP4 H264 CRF20".to_string(),
            notes: "notes".to_string(),
        };

        let error = validate_preset(&preset).expect_err("should reject invalid logo");
        assert!(error.contains(".png"));
    }
}
