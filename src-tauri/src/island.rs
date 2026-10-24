//! La pastille en haut de l'écran : petite quand on ne s'en sert pas, elle
//! s'agrandit au survol (principe inspiré de l'« île » de Coucou, code
//! écrit pour SKELVIA Pulse).

use tauri::{LogicalSize, PhysicalPosition, WebviewWindow};

pub const LABEL: &str = "island";
const COMPACT: (f64, f64) = (256.0, 52.0);
const EXPANDED: (f64, f64) = (332.0, 200.0);
const TOP_MARGIN: f64 = 6.0;

/// Place la pastille en haut, au centre de l'écran principal.
pub fn place(win: &WebviewWindow, expanded: bool) {
    let (w, h) = if expanded { EXPANDED } else { COMPACT };
    let monitor = win.current_monitor().ok().flatten().or_else(|| win.primary_monitor().ok().flatten());
    let Some(m) = monitor else {
        let _ = win.set_size(LogicalSize::new(w, h));
        return;
    };
    let scale = m.scale_factor();
    let pos = m.position();
    let size = m.size();
    let x = pos.x as f64 + (size.width as f64 - w * scale) / 2.0;
    let y = pos.y as f64 + TOP_MARGIN * scale;
    let _ = win.set_size(LogicalSize::new(w, h));
    let _ = win.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}
