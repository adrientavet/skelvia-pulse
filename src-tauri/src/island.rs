//! La pastille : petite quand on ne s'en sert pas, elle s'agrandit au survol
//! (principe inspiré de l'« île » de Coucou, code écrit pour SKELVIA Pulse).
//!
//! L'utilisateur peut la faire glisser n'importe où ; sa position est gardée
//! dans les réglages. Par défaut, elle est en haut, au centre de l'écran
//! principal. Agrandie, elle s'ouvre vers le bas, ou vers le haut si elle est
//! trop près du bas de l'écran ; la petite pilule ne bouge jamais.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::state::AppState;

pub const LABEL: &str = "island";
/// Tailles en pixels « logiques » (avant mise à l'échelle de Windows).
const COMPACT: (f64, f64) = (268.0, 52.0);
/// Assez haute pour le bloc « Nouvelle version » ; la fenêtre est
/// transparente autour du panneau.
const EXPANDED: (f64, f64) = (332.0, 272.0);
const TOP_MARGIN: f64 = 6.0;
/// Les déplacements reçus juste après un placement par le programme sont
/// ignorés : ce ne sont pas des glissements de l'utilisateur.
const IGNORE_AFTER_PLACE: Duration = Duration::from_millis(400);
/// La position est enregistrée quand la pastille ne bouge plus.
const SAVE_DELAY: Duration = Duration::from_millis(700);

/// Un écran, en pixels physiques.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f64,
}

impl Area {
    fn from_monitor(m: &Monitor) -> Self {
        Area {
            x: m.position().x,
            y: m.position().y,
            w: m.size().width as i32,
            h: m.size().height as i32,
            scale: m.scale_factor(),
        }
    }
    fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

fn px(logical: f64, scale: f64) -> i32 {
    (logical * scale).round() as i32
}

/// Coin haut gauche de la petite pilule : la position gardée si son centre
/// est sur un écran (ramenée entièrement sur cet écran), sinon en haut au
/// centre de l'écran principal. Renvoie aussi l'écran concerné.
pub fn compact_origin(saved: Option<[i32; 2]>, screens: &[Area], primary: Area) -> ([i32; 2], Area) {
    if let Some([x, y]) = saved {
        let found = screens.iter().find(|s| {
            let (w, h) = (px(COMPACT.0, s.scale), px(COMPACT.1, s.scale));
            s.contains(x + w / 2, y + h / 2)
        });
        if let Some(s) = found {
            let (w, h) = (px(COMPACT.0, s.scale), px(COMPACT.1, s.scale));
            let cx = x.clamp(s.x, s.x + (s.w - w).max(0));
            let cy = y.clamp(s.y, s.y + (s.h - h).max(0));
            return ([cx, cy], *s);
        }
    }
    let w = px(COMPACT.0, primary.scale);
    let x = primary.x + (primary.w - w) / 2;
    let y = primary.y + px(TOP_MARGIN, primary.scale);
    ([x, y], primary)
}

/// Fenêtre agrandie autour de la pilule : centrée sur elle, ouverte vers le
/// bas, ou vers le haut si la place manque. Renvoie (origine, taille, vers le haut).
pub fn expanded_rect(compact: [i32; 2], screen: Area) -> ([i32; 2], [i32; 2], bool) {
    let (cw, ch) = (px(COMPACT.0, screen.scale), px(COMPACT.1, screen.scale));
    let (ew, eh) = (px(EXPANDED.0, screen.scale), px(EXPANDED.1, screen.scale));
    let x = compact[0] - (ew - cw) / 2;
    let up = compact[1] + eh > screen.y + screen.h;
    let y = if up { compact[1] + ch - eh } else { compact[1] };
    ([x, y], [ew, eh], up)
}

#[derive(Default)]
struct Inner {
    /// Position de la pilule pendant cette session.
    compact: Option<[i32; 2]>,
    /// Écart entre la fenêtre et la pilule (non nul quand elle est agrandie).
    offset: [i32; 2],
    placed_at: Option<Instant>,
    move_generation: u64,
}

#[derive(Default)]
pub struct Island {
    inner: Mutex<Inner>,
}

fn screens(win: &WebviewWindow) -> (Vec<Area>, Area) {
    let all: Vec<Area> = win.available_monitors().unwrap_or_default().iter().map(Area::from_monitor).collect();
    let primary = win
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| Area::from_monitor(&m))
        .or_else(|| all.first().copied())
        .unwrap_or(Area { x: 0, y: 0, w: 1280, h: 720, scale: 1.0 });
    (all, primary)
}

/// Place la pastille, petite ou agrandie. Renvoie vrai si elle s'ouvre vers le haut.
pub fn place(app: &AppHandle, win: &WebviewWindow, expanded: bool) -> bool {
    let island = app.state::<Island>();
    let saved = app.state::<Arc<AppState>>().settings().island_position;
    let (all, primary) = screens(win);
    let mut inner = island.inner.lock().unwrap();
    let (compact, screen) = compact_origin(inner.compact.or(saved), &all, primary);
    inner.compact = Some(compact);
    let (origin, size, up) = if expanded {
        expanded_rect(compact, screen)
    } else {
        (compact, [px(COMPACT.0, screen.scale), px(COMPACT.1, screen.scale)], false)
    };
    inner.offset = [origin[0] - compact[0], origin[1] - compact[1]];
    inner.placed_at = Some(Instant::now());
    drop(inner);
    let _ = win.set_size(PhysicalSize::new(size[0] as u32, size[1] as u32));
    let _ = win.set_position(PhysicalPosition::new(origin[0], origin[1]));
    up
}

/// Appelé quand la fenêtre de la pastille bouge : si c'est l'utilisateur qui
/// la fait glisser, on garde la nouvelle position, puis on l'enregistre
/// quand elle ne bouge plus.
pub fn on_moved(app: &AppHandle, pos: PhysicalPosition<i32>) {
    let island = app.state::<Island>();
    let generation = {
        let mut inner = island.inner.lock().unwrap();
        if inner.placed_at.is_some_and(|t| t.elapsed() < IGNORE_AFTER_PLACE) {
            return;
        }
        inner.compact = Some([pos.x - inner.offset[0], pos.y - inner.offset[1]]);
        inner.move_generation += 1;
        inner.move_generation
    };
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(SAVE_DELAY);
        let island = app.state::<Island>();
        let compact = {
            let inner = island.inner.lock().unwrap();
            if inner.move_generation != generation {
                return; // elle bouge encore
            }
            inner.compact
        };
        let state = app.state::<Arc<AppState>>();
        let mut s = state.settings();
        if s.island_position != compact {
            s.island_position = compact;
            state.save_settings(s);
        }
    });
}

/// Remet la pastille en haut, au centre de l'écran principal.
pub fn reset(app: &AppHandle) {
    let state = app.state::<Arc<AppState>>();
    let mut s = state.settings();
    s.island_position = None;
    state.save_settings(s);
    app.state::<Island>().inner.lock().unwrap().compact = None;
    if let Some(win) = app.get_webview_window(LABEL) {
        place(app, &win, false);
        // Avant l'accord du premier lancement, la pastille reste cachée.
        if state.settings().first_run_done {
            let _ = win.show();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Area = Area { x: 0, y: 0, w: 1920, h: 1080, scale: 1.0 };
    const RIGHT: Area = Area { x: 1920, y: 0, w: 2560, h: 1440, scale: 1.5 };

    #[test]
    fn default_is_top_center_of_primary() {
        let (pos, s) = compact_origin(None, &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 268) / 2, 6]);
        assert_eq!(s, SCREEN);
    }

    #[test]
    fn saved_position_is_kept_and_clamped() {
        let (pos, _) = compact_origin(Some([100, 900]), &[SCREEN], SCREEN);
        assert_eq!(pos, [100, 900]);
        // Dépasse à droite (centre encore sur l'écran) : ramenée entièrement sur l'écran.
        let (pos, _) = compact_origin(Some([1700, 500]), &[SCREEN], SCREEN);
        assert_eq!(pos, [1920 - 268, 500]);
        // Centre hors de tout écran : position par défaut.
        let (pos, _) = compact_origin(Some([1800, 500]), &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 268) / 2, 6]);
    }

    #[test]
    fn position_on_a_missing_screen_falls_back_to_default() {
        // Deuxième écran débranché depuis.
        let (pos, _) = compact_origin(Some([2500, 300]), &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 268) / 2, 6]);
        // Écran présent, avec sa propre mise à l'échelle.
        let (pos, s) = compact_origin(Some([2500, 300]), &[SCREEN, RIGHT], SCREEN);
        assert_eq!(pos, [2500, 300]);
        assert_eq!(s, RIGHT);
    }

    #[test]
    fn expands_down_or_up_without_moving_the_pill() {
        let (origin, size, up) = expanded_rect([500, 100], SCREEN);
        assert!(!up);
        assert_eq!(origin, [500 - (332 - 268) / 2, 100]);
        assert_eq!(size, [332, 272]);
        // Près du bas de l'écran : s'ouvre vers le haut, la pilule garde sa place.
        let (origin, size, up) = expanded_rect([500, 1000], SCREEN);
        assert!(up);
        assert_eq!(origin[1] + size[1], 1000 + 52, "le bas de la fenêtre reste le bas de la pilule");
    }
}
