use serde::{Deserialize, Serialize};

use crate::domain::video_item::VideoItem;
use crate::services::mapping_service::AcceptedFile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub job_id: String,
    pub created_at: String,
    pub selected_task: Option<String>,
    pub preset_id: Option<String>,
    pub output_folder: String,
    #[serde(default)]
    pub export_output_folder: Option<String>,
    pub status: String,
    pub video_items: Vec<VideoItem>,
    pub imported_files: Vec<AcceptedFile>,
}
