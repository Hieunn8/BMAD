use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub preset_id: String,
    pub brand_name: String,
    pub default_logo_path: String,
    pub audio_replacement_policy: String,
    pub subtitle_style_preset: String,
    pub layout_rules: String,
    pub export_preset: String,
    pub notes: String,
}
