use crate::actions::{spawn_dispatch, ActivationSource, AppAction};
use crate::app::AppServices;
use crate::platform::PlatformServices;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Manager,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn setup(app: &mut App) {
    let services = app.state::<AppServices>().inner().clone();
    match build_tray(app) {
        Ok(()) => services.runtime_diagnostics.set_tray_status("available"),
        Err(_) => services
            .runtime_diagnostics
            .set_tray_status("unavailable; main window remains usable"),
    }

    services
        .runtime_diagnostics
        .set_shortcut_status(register_direct_shortcut(app, services.platform.as_ref()));
}

fn tray_action_from_id(id: &str) -> Option<AppAction> {
    match id {
        "capture" => Some(AppAction::StartCapture),
        "cancel" => Some(AppAction::CancelCapture),
        "show" => Some(AppAction::ShowMainWindow),
        "quit" => Some(AppAction::Quit),
        _ => None,
    }
}

fn build_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let capture = MenuItem::with_id(app, "capture", "Capture text", true, None::<&str>)?;
    let cancel = MenuItem::with_id(app, "cancel", "Cancel capture", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Screenshot OCR", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&capture, &cancel, &show, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Screenshot OCR")
        .on_menu_event(|app, event| {
            if let Some(action) = tray_action_from_id(event.id.as_ref()) {
                spawn_dispatch(app.clone(), action, ActivationSource::Tray);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn register_direct_shortcut(app: &App, platform: &dyn PlatformServices) -> &'static str {
    let Some(shortcut) = platform.direct_shortcut() else {
        return platform.shortcut_unavailable_status();
    };
    match app.global_shortcut().register(shortcut) {
        Ok(()) => "registered",
        Err(_) => "registration failed; use the documented platform shortcut",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_actions_route_only_the_four_approved_commands() {
        assert_eq!(tray_action_from_id("capture"), Some(AppAction::StartCapture));
        assert_eq!(tray_action_from_id("cancel"), Some(AppAction::CancelCapture));
        assert_eq!(tray_action_from_id("show"), Some(AppAction::ShowMainWindow));
        assert_eq!(tray_action_from_id("quit"), Some(AppAction::Quit));
        for unrecognized in ["", "toggle", "Capture", "capture --quit", "../quit"] {
            assert_eq!(tray_action_from_id(unrecognized), None);
        }
    }
}
