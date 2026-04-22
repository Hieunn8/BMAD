use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::constants::{FFMPEG_RESOURCE_RELATIVE_PATH, FFPROBE_RESOURCE_RELATIVE_PATH};

#[tauri::command]
pub fn get_ffmpeg_path(app_handle: AppHandle) -> Result<String, String> {
    resolve_ffmpeg_path(&app_handle)
        .map(|path| path.display().to_string())
        .ok_or_else(|| "Khong tim thay FFmpeg binary da bundle".to_string())
}

pub fn resolve_ffmpeg_path(app_handle: &AppHandle) -> Option<PathBuf> {
    resolve_binary_path(app_handle, FFMPEG_RESOURCE_RELATIVE_PATH, "ffmpeg.exe")
}

pub fn resolve_ffprobe_path(app_handle: &AppHandle) -> Option<PathBuf> {
    resolve_binary_path(app_handle, FFPROBE_RESOURCE_RELATIVE_PATH, "ffprobe.exe")
}

fn resolve_binary_path(
    app_handle: &AppHandle,
    resource_relative_path: &str,
    binary_name: &str,
) -> Option<PathBuf> {
    let resource_dir = app_handle.path().resource_dir().ok();

    resource_dir
        .and_then(|dir| find_first_existing_path(dir, resource_relative_path, binary_name))
        .or_else(|| {
            find_first_existing_path(
                std::env::current_dir().ok()?,
                resource_relative_path,
                binary_name,
            )
        })
}

fn find_first_existing_path(root: PathBuf, resource_relative_path: &str, binary_name: &str) -> Option<PathBuf> {
    let candidates = [
        root.join("ffmpeg").join(binary_name),
        root.join("resources").join("ffmpeg").join(binary_name),
        root.join("src-tauri").join("resources").join("ffmpeg").join(binary_name),
        root.join(resource_relative_path),
    ];

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn finds_ffmpeg_in_expected_subdirectory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let ffmpeg_dir = temp_dir.path().join("resources").join("ffmpeg");
        std::fs::create_dir_all(&ffmpeg_dir).expect("create ffmpeg dir");
        let ffmpeg_path = ffmpeg_dir.join("ffmpeg.exe");
        let ffprobe_path = ffmpeg_dir.join("ffprobe.exe");
        std::fs::write(&ffmpeg_path, b"binary").expect("write ffmpeg");
        std::fs::write(&ffprobe_path, b"binary").expect("write ffprobe");

        let resolved = find_first_existing_path(
            temp_dir.path().to_path_buf(),
            FFMPEG_RESOURCE_RELATIVE_PATH,
            "ffmpeg.exe",
        );

        assert_eq!(resolved.as_deref(), Some(Path::new(&ffmpeg_path)));
    }
}
