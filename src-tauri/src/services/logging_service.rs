use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::{
    constants::LOGS_DIR,
    domain::{job::Job, video_item::VideoItem},
};

#[derive(Debug, Default)]
pub struct LoggingService;

impl LoggingService {
    pub fn append_video_log(job: &Job, video: &VideoItem, line: &str) -> Result<(), String> {
        let log_dir = PathBuf::from(&job.output_folder).join(LOGS_DIR);
        fs::create_dir_all(&log_dir).map_err(|error| format!("Khong tao duoc log dir: {error}"))?;

        let log_path = log_dir.join(format!("{}.log", video.video_id));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("Khong mo duoc log file `{}`: {error}", log_path.display()))?;

        writeln!(file, "{line}").map_err(|error| format!("Khong ghi duoc log file: {error}"))
    }
}
