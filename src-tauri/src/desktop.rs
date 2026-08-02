use crate::actions::{spawn_dispatch, ActivationSource, AppAction};
use crate::app::AppServices;
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
        .set_shortcut_status(register_direct_shortcut(app));
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
            let action = match event.id.as_ref() {
                "capture" => Some(AppAction::StartCapture),
                "cancel" => Some(AppAction::CancelCapture),
                "show" => Some(AppAction::ShowMainWindow),
                "quit" => Some(AppAction::Quit),
                _ => None,
            };
            if let Some(action) = action {
                spawn_dispatch(app.clone(), action, ActivationSource::Tray);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn register_direct_shortcut(app: &App) -> &'static str {
    let Some(shortcut) = direct_shortcut() else {
        return "deferred; use the GNOME custom shortcut";
    };
    match app.global_shortcut().register(shortcut) {
        Ok(()) => "registered",
        Err(_) => "registration failed; use the documented platform shortcut",
    }
}

#[cfg(target_os = "linux")]
fn direct_shortcut() -> Option<&'static str> {
    direct_shortcut_for_session(std::env::var("XDG_SESSION_TYPE").ok().as_deref())
}

#[cfg(target_os = "linux")]
fn direct_shortcut_for_session(session: Option<&str>) -> Option<&'static str> {
    match session {
        Some(session) if session.eq_ignore_ascii_case("x11") => Some("Super+Shift+O"),
        _ => None,
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn direct_shortcut() -> Option<&'static str> {
    Some("CmdOrCtrl+Shift+O")
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn direct_shortcut() -> Option<&'static str> {
    None
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::direct_shortcut_for_session;

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_direct_shortcut_is_x11_only() {
        assert_eq!(
            direct_shortcut_for_session(Some("x11")),
            Some("Super+Shift+O")
        );
        assert_eq!(direct_shortcut_for_session(Some("wayland")), None);
        assert_eq!(direct_shortcut_for_session(None), None);
    }
}
