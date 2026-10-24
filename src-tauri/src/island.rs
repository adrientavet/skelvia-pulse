//! La pastille : une toute petite barre, qui ne change pas au survol. Un
//! clic l'ouvre (temps du jour, pause, bouton pour ouvrir l'application),
//! un autre clic la referme (principe inspiré de l'« île » de Coucou, code
//! écrit pour SKELVIA Pulse).
//!
//! Cliquer, maintenir et glisser la déplace. Une fois lâchée, elle se colle
//! au bord de l'écran le plus proche (haut, bas, gauche ou droite), là où
//! elle a été lâchée le long de ce bord. Sa position est gardée dans les
//! réglages. Par défaut, elle est en haut, au centre de l'écran principal. Ouverte, elle se déploie vers le bas, ou vers le haut si elle
//! est trop près du bas de l'écran ; la petite barre ne bouge jamais.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::state::AppState;

pub const LABEL: &str = "island";
/// Tailles en pixels « logiques » (avant mise à l'échelle de Windows).
const COMPACT: (f64, f64) = (150.0, 40.0);
/// Assez haute pour le bloc « Nouvelle version » ; la fenêtre est
/// transparente autour du panneau.
const EXPANDED: (f64, f64) = (300.0, 290.0);
/// Les déplacements reçus juste après un placement par le programme sont
/// ignorés : ce ne sont pas des glissements de l'utilisateur.
const IGNORE_AFTER_PLACE: Duration = Duration::from_millis(400);
/// Après un déplacement, on attend que la barre soit lâchée, puis on la
/// colle au bord le plus proche, en quelques images.
const SETTLE_DELAY: Duration = Duration::from_millis(120);
const SNAP_STEPS: i32 = 6;
const SNAP_STEP: Duration = Duration::from_millis(14);

/// La partie utilisable d'un écran (sans la barre des tâches), en pixels
/// physiques.
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
        let work = m.work_area();
        Area {
            x: work.position.x,
            y: work.position.y,
            w: work.size.width as i32,
            h: work.size.height as i32,
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

/// L'écran sous le centre de la barre, s'il y en a un.
fn screen_under(pos: [i32; 2], screens: &[Area]) -> Option<Area> {
    screens
        .iter()
        .find(|s| {
            let (w, h) = (px(COMPACT.0, s.scale), px(COMPACT.1, s.scale));
            s.contains(pos[0] + w / 2, pos[1] + h / 2)
        })
        .copied()
}

/// Colle la barre au bord le plus proche de l'écran, sans la faire glisser
/// le long de ce bord (elle reste entièrement sur l'écran).
pub fn snap(pos: [i32; 2], s: Area) -> [i32; 2] {
    let (w, h) = (px(COMPACT.0, s.scale), px(COMPACT.1, s.scale));
    let x = pos[0].clamp(s.x, s.x + (s.w - w).max(0));
    let y = pos[1].clamp(s.y, s.y + (s.h - h).max(0));
    let left = x - s.x;
    let right = s.x + s.w - (x + w);
    let top = y - s.y;
    let bottom = s.y + s.h - (y + h);
    let nearest = left.min(right).min(top).min(bottom);
    if nearest == top {
        [x, s.y]
    } else if nearest == bottom {
        [x, s.y + s.h - h]
    } else if nearest == left {
        [s.x, y]
    } else {
        [s.x + s.w - w, y]
    }
}

/// Coin haut gauche de la barre : la position gardée si son centre est sur
/// un écran (collée au bord le plus proche), sinon en haut au centre de
/// l'écran principal. Renvoie aussi l'écran concerné.
pub fn compact_origin(saved: Option<[i32; 2]>, screens: &[Area], primary: Area) -> ([i32; 2], Area) {
    if let Some(pos) = saved {
        if let Some(s) = screen_under(pos, screens) {
            return (snap(pos, s), s);
        }
    }
    let w = px(COMPACT.0, primary.scale);
    ([primary.x + (primary.w - w) / 2, primary.y], primary)
}

/// Fenêtre ouverte autour de la barre : centrée sur elle (sans sortir de
/// l'écran), déployée vers le bas, ou vers le haut si la place manque.
/// Renvoie (origine, taille, vers le haut).
pub fn expanded_rect(compact: [i32; 2], screen: Area) -> ([i32; 2], [i32; 2], bool) {
    let (cw, ch) = (px(COMPACT.0, screen.scale), px(COMPACT.1, screen.scale));
    let (ew, eh) = (px(EXPANDED.0, screen.scale), px(EXPANDED.1, screen.scale));
    let x = (compact[0] - (ew - cw) / 2).clamp(screen.x, screen.x + (screen.w - ew).max(0));
    let up = compact[1] + eh > screen.y + screen.h;
    let y = if up { compact[1] + ch - eh } else { compact[1] };
    ([x, y], [ew, eh], up)
}

/// Ce que l'interface doit savoir de la fenêtre ouverte.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    /// Déployée vers le haut (barre posée en bas de l'écran).
    pub up: bool,
    /// Position de la barre depuis le bord gauche de la fenêtre, en pixels
    /// de l'interface (non nulle quand la fenêtre est ouverte).
    pub bar_left: f64,
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

/// Place la pastille, fermée ou ouverte.
pub fn place(app: &AppHandle, win: &WebviewWindow, expanded: bool) -> Placement {
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
    Placement { up, bar_left: (compact[0] - origin[0]) as f64 / screen.scale }
}

/// Appelé quand la fenêtre de la pastille bouge : si c'est l'utilisateur qui
/// la fait glisser, on suit sa position ; quand il la lâche, on la colle au
/// bord le plus proche et on enregistre sa place.
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
        std::thread::sleep(SETTLE_DELAY);
        // On attend que le bouton de la souris soit relâché (au plus une
        // minute) ; un nouveau déplacement reprend la main.
        let island = app.state::<Island>();
        let still = |island: &Island| island.inner.lock().unwrap().move_generation == generation;
        let started = Instant::now();
        while pulse_collector::mouse_button_down() && started.elapsed() < Duration::from_secs(60) {
            if !still(&island) {
                return;
            }
            std::thread::sleep(Duration::from_millis(60));
        }
        if !still(&island) {
            return;
        }
        let Some(win) = app.get_webview_window(LABEL) else { return };
        let (all, primary) = screens(&win);
        let (from, offset) = {
            let inner = island.inner.lock().unwrap();
            (inner.compact.unwrap_or_default(), inner.offset)
        };
        let (target, _) = compact_origin(Some(from), &all, primary);
        // Petit glissement jusqu'au bord.
        for step in 1..=SNAP_STEPS {
            let x = from[0] + (target[0] - from[0]) * step / SNAP_STEPS;
            let y = from[1] + (target[1] - from[1]) * step / SNAP_STEPS;
            island.inner.lock().unwrap().placed_at = Some(Instant::now());
            let _ = win.set_position(PhysicalPosition::new(x + offset[0], y + offset[1]));
            std::thread::sleep(SNAP_STEP);
        }
        island.inner.lock().unwrap().compact = Some(target);
        let state = app.state::<Arc<AppState>>();
        let mut s = state.settings();
        if s.island_position != Some(target) {
            s.island_position = Some(target);
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
        assert_eq!(pos, [(1920 - 150) / 2, 0]);
        assert_eq!(s, SCREEN);
    }

    #[test]
    fn sticks_to_the_nearest_edge_where_it_was_dropped() {
        // Près du haut : collée en haut, à la même place de gauche à droite.
        assert_eq!(snap([600, 50], SCREEN), [600, 0]);
        // Près du bas.
        assert_eq!(snap([600, 1000], SCREEN), [600, 1080 - 40]);
        // Près de la gauche, à mi-hauteur.
        assert_eq!(snap([30, 500], SCREEN), [0, 500]);
        // Près de la droite.
        assert_eq!(snap([1700, 500], SCREEN), [1920 - 150, 500]);
        // Dans un coin : reste dans le coin.
        assert_eq!(snap([1900, 1070], SCREEN), [1920 - 150, 1080 - 40]);
    }

    #[test]
    fn saved_position_is_snapped_on_its_screen() {
        let (pos, _) = compact_origin(Some([300, 900]), &[SCREEN], SCREEN);
        assert_eq!(pos, [300, 1080 - 40]);
        // Centre hors de tout écran : position par défaut.
        let (pos, _) = compact_origin(Some([1900, 500]), &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 150) / 2, 0]);
    }

    #[test]
    fn position_on_a_missing_screen_falls_back_to_default() {
        // Deuxième écran débranché depuis.
        let (pos, _) = compact_origin(Some([2500, 300]), &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 150) / 2, 0]);
        // Écran présent, avec sa propre mise à l'échelle : collée en haut.
        let (pos, s) = compact_origin(Some([2500, 300]), &[SCREEN, RIGHT], SCREEN);
        assert_eq!(pos, [2500, 0]);
        assert_eq!(s, RIGHT);
    }

    #[test]
    fn expands_down_or_up_without_moving_the_bar() {
        let (origin, size, up) = expanded_rect([500, 100], SCREEN);
        assert!(!up);
        assert_eq!(origin, [500 - (300 - 150) / 2, 100]);
        assert_eq!(size, [300, 290]);
        // Près du bas de l'écran : s'ouvre vers le haut, la barre garde sa place.
        let (origin, size, up) = expanded_rect([500, 1040], SCREEN);
        assert!(up);
        assert_eq!(origin[1] + size[1], 1040 + 40, "le bas de la fenêtre reste le bas de la barre");
        // Contre le bord gauche : la fenêtre ouverte reste sur l'écran.
        let (origin, _, _) = expanded_rect([0, 100], SCREEN);
        assert_eq!(origin[0], 0);
    }
}
