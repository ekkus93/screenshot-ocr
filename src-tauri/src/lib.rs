mod app;
mod capture;
mod commands;
mod diagnostics;
mod error;
mod image_pipeline;
mod models;
mod ocr;
mod settings;
mod state;

use app::AppServices;
use commands::{cancel_capture, copy_text, get_diagnostics, get_settings, reset_settings, start_capture, update_settings};
use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .compact()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir().map_err(|_| "configuration directory unavailable")?;
            app.manage(AppServices::new(config_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_capture,
            cancel_capture,
            copy_text,
            get_settings,
            update_settings,
            reset_settings,
            get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Screenshot OCR");
}
