mod batch;
mod commands;
mod discovery;
mod error;
mod image_io;
mod preview;
mod processing;

use batch::BatchControl;

/// Opens the window and serves the front end's commands until it closes.
///
/// # Panics
///
/// If the window or its webview cannot be created — there is nowhere left to
/// report the failure to.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(BatchControl::default())
        .invoke_handler(tauri::generate_handler![
            commands::import_paths,
            commands::supported_extensions,
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
