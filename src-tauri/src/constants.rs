pub const APP_DATA_ROOT_DIR: &str = "jobs";
pub const JOB_MANIFEST: &str = "job.json";
pub const PRESETS_ROOT_DIR: &str = "presets";
pub const VIDEO_STATE_DIR: &str = "videos";
pub const SEGMENT_STATE_DIR: &str = "segments";
pub const WORKING_DIR: &str = "working";
pub const REPORTS_DIR: &str = "reports";
pub const LOGS_DIR: &str = "logs";
pub const CACHE_DIR: &str = "cache";
pub const PREVIEW_CACHE_DIR: &str = "cache/previews";
pub const FFMPEG_RESOURCE_RELATIVE_PATH: &str = "resources/ffmpeg/ffmpeg.exe";
pub const FFPROBE_RESOURCE_RELATIVE_PATH: &str = "resources/ffmpeg/ffprobe.exe";

pub fn job_output_layout(job_id: &str) -> Vec<String> {
    vec![
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{JOB_MANIFEST}"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{VIDEO_STATE_DIR}/"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{SEGMENT_STATE_DIR}/"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{WORKING_DIR}/"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{REPORTS_DIR}/"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{LOGS_DIR}/"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{CACHE_DIR}/"),
        format!("{APP_DATA_ROOT_DIR}/{job_id}/{PREVIEW_CACHE_DIR}/"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_job_output_layout() {
        let layout = job_output_layout("job-123");

        assert!(layout.contains(&"jobs/job-123/job.json".to_string()));
        assert!(layout.contains(&"jobs/job-123/videos/".to_string()));
        assert!(layout.contains(&"jobs/job-123/segments/".to_string()));
        assert!(layout.contains(&"jobs/job-123/working/".to_string()));
        assert!(layout.contains(&"jobs/job-123/reports/".to_string()));
        assert!(layout.contains(&"jobs/job-123/logs/".to_string()));
        assert!(layout.contains(&"jobs/job-123/cache/".to_string()));
        assert!(layout.contains(&"jobs/job-123/cache/previews/".to_string()));
    }
}
