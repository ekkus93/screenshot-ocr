mod actions;
mod app;
mod cancellation;
mod capture;
mod commands;
mod desktop;
mod diagnostics;
// Stable public error codes are shared by Linux and native Windows capture.
#[cfg_attr(target_os = "windows", allow(dead_code))]
mod error;
mod image_pipeline;
mod models;
mod ocr;
mod platform;
mod settings;
mod state;

use actions::{spawn_dispatch, ActivationSource, AppAction};
use app::AppServices;
use commands::{
    cancel_capture, copy_text, get_diagnostics, get_settings, reset_settings, start_capture,
    take_pending_app_action, update_settings,
};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

/// A global hotkey may deliver repeated Pressed events while a key is held.
/// Dispatch only on the first edge; a fresh press after Released still toggles.
#[derive(Default)]
struct ShortcutPressEdge(AtomicBool);

impl ShortcutPressEdge {
    fn observe(&self, state: ShortcutState) -> bool {
        if state == ShortcutState::Pressed {
            !self.0.swap(true, Ordering::AcqRel)
        } else {
            self.0.store(false, Ordering::Release);
            false
        }
    }
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .compact()
        .init();

    let initial_action = AppAction::from_argument(std::env::args().nth(1).as_deref());
    let shortcut_edge = ShortcutPressEdge::default();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let action = AppAction::from_secondary_args(&args);
            spawn_dispatch(app.clone(), action, ActivationSource::SecondInstance);
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if shortcut_edge.observe(event.state()) {
                        spawn_dispatch(
                            app.clone(),
                            AppAction::ToggleCapture,
                            ActivationSource::GlobalShortcut,
                        );
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(move |app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|_| "configuration directory unavailable")?;
            app.manage(AppServices::new(config_dir));
            desktop::setup(app);
            if let Some(action) = initial_action {
                spawn_dispatch(app.handle().clone(), action, ActivationSource::Startup);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_capture,
            cancel_capture,
            take_pending_app_action,
            copy_text,
            get_settings,
            update_settings,
            reset_settings,
            get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Screenshot OCR");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_hotkey_press_is_one_action_until_release() {
        let edge = ShortcutPressEdge::default();
        assert!(edge.observe(ShortcutState::Pressed));
        assert!(!edge.observe(ShortcutState::Pressed));
        assert!(!edge.observe(ShortcutState::Pressed));
        assert!(!edge.observe(ShortcutState::Released));
        assert!(!edge.observe(ShortcutState::Released));
        assert!(edge.observe(ShortcutState::Pressed));
    }
}
