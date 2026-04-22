use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub video_id: String,
    pub source_path: String,
    pub source_metadata: Option<String>,
    pub mapped_logo_path: Option<String>,
    pub mapped_audio_path: Option<String>,
    pub mapped_srt_path: Option<String>,
    pub status: String,
}
