mod commands;
mod image_io;
mod processing;

use commands::BatchControl;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(BatchControl::default())
        .invoke_handler(tauri::generate_handler![
            commands::import_paths,
            commands::build_preview,
            commands::image_metadata,
            commands::develop_batch,
            commands::cancel_batch,
            commands::default_output_dir,
            commands::startup_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
