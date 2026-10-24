//! Icône près de l'horloge de Windows : toujours visible, avec l'état
//! (enregistre ou en pause) et un menu court.

use std::sync::atomic::{AtomicU8, Ordering};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::state::Status;

const TRAY_ID: &str = "pulse";
const ICON_ACTIVE: &[u8] = include_bytes!("../icons/tray-active.png");
const ICON_PAUSED: &[u8] = include_bytes!("../icons/tray-paused.png");

/// Dernier état affiché (0 : inconnu, 1 : enregistre, 2 : en pause, 3 : inactif).
static SHOWN: AtomicU8 = AtomicU8::new(0);

pub struct TrayItems {
    pause: MenuItem<Wry>,
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Ouvrir SKELVIA Pulse", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Mettre en pause", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter (l'enregistrement s'arrête)", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &pause, &sep, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_ACTIVE)?)
        .tooltip("SKELVIA Pulse")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => crate::show_main(app),
            "pause" => crate::commands::toggle_pause(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                crate::show_main(tray.app_handle());
            }
        })
        .build(app)?;
    app.manage(TrayItems { pause });
    Ok(())
}

/// Met l'icône et le menu à jour quand l'état change.
pub fn refresh(app: &AppHandle, status: &Status) {
    let code = if status.paused { 2 } else if status.idle { 3 } else { 1 };
    if SHOWN.swap(code, Ordering::SeqCst) == code {
        return;
    }
    let (icon, tip) = match code {
        2 => (ICON_PAUSED, "SKELVIA Pulse : en pause, rien n'est enregistré"),
        3 => (ICON_PAUSED, "SKELVIA Pulse : en veille, aucune activité en ce moment"),
        _ => (ICON_ACTIVE, "SKELVIA Pulse : enregistre"),
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(img) = Image::from_bytes(icon) {
            let _ = tray.set_icon(Some(img));
        }
        let _ = tray.set_tooltip(Some(tip));
    }
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.pause.set_text(if status.paused { "Reprendre l'enregistrement" } else { "Mettre en pause" });
    }
}
