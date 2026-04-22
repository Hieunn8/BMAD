mod commands;
mod constants;
mod domain;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::app::get_ffmpeg_path,
            commands::job_commands::import_assets,
            commands::job_commands::create_job,
            commands::job_commands::get_job_readiness,
            commands::job_commands::list_jobs,
            commands::job_commands::load_job,
            commands::job_commands::start_job,
            commands::export_commands::get_export_readiness,
            commands::export_commands::set_export_output_folder,
            commands::export_commands::start_export,
            commands::export_commands::generate_video_report,
            commands::export_commands::get_report,
            commands::export_commands::get_job_export_summary_report,
            commands::preset_commands::create_preset,
            commands::preset_commands::edit_preset,
            commands::preset_commands::duplicate_preset,
            commands::mapping_commands::auto_map_job,
            commands::mapping_commands::fix_mapping,
            commands::preset_commands::list_presets,
            commands::preset_commands::select_preset,
            commands::review_commands::get_review_context,
            commands::review_commands::get_video_preview,
            commands::review_commands::apply_logo_fix,
            commands::review_commands::apply_subtitle_fix,
            commands::review_commands::mark_segment_accepted,
            commands::review_commands::reset_logo_fix,
            commands::review_commands::reset_subtitle_fix,
            commands::review_commands::check_video_review_gating,
            commands::review_commands::mark_video_ready,
            commands::review_commands::get_frame_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
