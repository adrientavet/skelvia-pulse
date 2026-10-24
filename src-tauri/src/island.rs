//! La pastille : une toute petite barre, qui ne change pas au survol. Un
//! clic l'ouvre (temps du jour, pause, bouton pour ouvrir l'application),
//! un autre clic la referme (principe inspiré de l'« île » de Coucou, code
//! écrit pour SKELVIA Pulse).
//!
//! Cliquer, maintenir et glisser la déplace. Une fois lâchée, elle se colle
//! au bord de l'écran le plus proche, là où elle a été lâchée le long de ce
//! bord : horizontale en haut et en bas, verticale à gauche et à droite.
//! Ouverte, son panneau se déploie vers l'intérieur de l'écran ; la petite
//! barre ne bouge pas. Sa place est gardée dans les réglages. Par défaut,
//! elle est en haut, au centre de l'écran principal.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::state::AppState;

pub const LABEL: &str = "island";
/// Tailles en pixels « logiques » (avant mise à l'échelle de Windows). La
/// fenêtre laisse 4 px transparents autour de la barre, pour son ombre.
const BAR_HORIZONTAL: (f64, f64) = (100.0, 40.0);
const BAR_VERTICAL: (f64, f64) = (54.0, 76.0);
/// Fenêtre ouverte : la barre et le panneau, en dessous ou au-dessus
/// (barre horizontale), ou à côté (barre verticale).
const OPEN_HORIZONTAL: (f64, f64) = (300.0, 290.0);
const OPEN_VERTICAL: (f64, f64) = (356.0, 290.0);
/// Les déplacements reçus juste après un placement par le programme sont
/// ignorés : ce ne sont pas des glissements de l'utilisateur.
const IGNORE_AFTER_PLACE: Duration = Duration::from_millis(400);
/// Après un déplacement, on attend que la barre soit lâchée, puis on la
/// colle au bord le plus proche, en quelques images.
const SETTLE_DELAY: Duration = Duration::from_millis(120);
const SNAP_STEPS: i32 = 6;
const SNAP_STEP: Duration = Duration::from_millis(14);

/// Le bord de l'écran contre lequel la barre est collée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    #[default]
    Haut,
    Bas,
    Gauche,
    Droite,
}

impl Edge {
    fn vertical(self) -> bool {
        matches!(self, Edge::Gauche | Edge::Droite)
    }
}

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
    fn contains(&self, p: [i32; 2]) -> bool {
        p[0] >= self.x && p[0] < self.x + self.w && p[1] >= self.y && p[1] < self.y + self.h
    }
}

fn px(logical: (f64, f64), scale: f64) -> [i32; 2] {
    [(logical.0 * scale).round() as i32, (logical.1 * scale).round() as i32]
}

fn bar_size(edge: Edge, scale: f64) -> [i32; 2] {
    px(if edge.vertical() { BAR_VERTICAL } else { BAR_HORIZONTAL }, scale)
}

/// Le bord de l'écran le plus proche d'un point.
pub fn nearest_edge(c: [i32; 2], s: Area) -> Edge {
    let candidates = [
        (c[1] - s.y, Edge::Haut),
        (s.y + s.h - c[1], Edge::Bas),
        (c[0] - s.x, Edge::Gauche),
        (s.x + s.w - c[0], Edge::Droite),
    ];
    candidates.iter().min_by_key(|(d, _)| *d).map(|(_, e)| *e).unwrap_or_default()
}

/// Colle la barre dont le centre est `c` au bord le plus proche, au même
/// endroit le long de ce bord (sans dépasser de l'écran). Renvoie le coin
/// haut gauche de sa fenêtre et le bord choisi.
pub fn snap(c: [i32; 2], s: Area) -> ([i32; 2], Edge) {
    let edge = nearest_edge(c, s);
    let [w, h] = bar_size(edge, s.scale);
    let x = (c[0] - w / 2).clamp(s.x, s.x + (s.w - w).max(0));
    let y = (c[1] - h / 2).clamp(s.y, s.y + (s.h - h).max(0));
    let origin = match edge {
        Edge::Haut => [x, s.y],
        Edge::Bas => [x, s.y + s.h - h],
        Edge::Gauche => [s.x, y],
        Edge::Droite => [s.x + s.w - w, y],
    };
    (origin, edge)
}

/// Place de la barre : la position gardée (son centre) si elle est sur un
/// écran, collée au bord le plus proche ; sinon en haut au centre de
/// l'écran principal. Renvoie le coin haut gauche, le bord et l'écran.
pub fn compact_origin(saved: Option<[i32; 2]>, screens: &[Area], primary: Area) -> ([i32; 2], Edge, Area) {
    if let Some(c) = saved {
        if let Some(s) = screens.iter().find(|s| s.contains(c)) {
            let (origin, edge) = snap(c, *s);
            return (origin, edge, *s);
        }
    }
    let (origin, edge) = snap([primary.x + primary.w / 2, primary.y], primary);
    (origin, edge, primary)
}

/// Fenêtre ouverte autour de la barre, sans sortir de l'écran : panneau en
/// dessous (barre en haut), au-dessus (barre en bas), à droite (barre à
/// gauche) ou à gauche (barre à droite). Renvoie (origine, taille).
pub fn expanded_rect(bar: [i32; 2], edge: Edge, s: Area) -> ([i32; 2], [i32; 2]) {
    let [bw, bh] = bar_size(edge, s.scale);
    if edge.vertical() {
        let [ew, eh] = px(OPEN_VERTICAL, s.scale);
        let x = if edge == Edge::Gauche { bar[0] } else { bar[0] + bw - ew };
        let y = (bar[1] + bh / 2 - eh / 2).clamp(s.y, s.y + (s.h - eh).max(0));
        ([x, y], [ew, eh])
    } else {
        let [ew, eh] = px(OPEN_HORIZONTAL, s.scale);
        let x = (bar[0] - (ew - bw) / 2).clamp(s.x, s.x + (s.w - ew).max(0));
        let y = if edge == Edge::Bas { bar[1] + bh - eh } else { bar[1] };
        ([x, y], [ew, eh])
    }
}

/// Ce que l'interface doit savoir pour dessiner la barre et le panneau.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub edge: Edge,
    pub expanded: bool,
    /// Position de la fenêtre de la barre dans la fenêtre ouverte, en pixels
    /// de l'interface (zéro quand elle est fermée).
    pub bar_left: f64,
    pub bar_top: f64,
}

#[derive(Default)]
struct Inner {
    /// Centre de la barre pendant cette session.
    center: Option<[i32; 2]>,
    /// Écart entre la fenêtre et la barre (non nul quand elle est ouverte).
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

/// Place la pastille, fermée ou ouverte, et prévient l'interface.
pub fn place(app: &AppHandle, win: &WebviewWindow, expanded: bool) -> Layout {
    let island = app.state::<Island>();
    let saved = app.state::<Arc<AppState>>().settings().island_position;
    let (all, primary) = screens(win);
    let mut inner = island.inner.lock().unwrap();
    let (bar, edge, screen) = compact_origin(inner.center.or(saved), &all, primary);
    let size = bar_size(edge, screen.scale);
    inner.center = Some([bar[0] + size[0] / 2, bar[1] + size[1] / 2]);
    let (origin, size) = if expanded { expanded_rect(bar, edge, screen) } else { (bar, size) };
    inner.offset = [origin[0] - bar[0], origin[1] - bar[1]];
    inner.placed_at = Some(Instant::now());
    drop(inner);
    let _ = win.set_size(PhysicalSize::new(size[0] as u32, size[1] as u32));
    let _ = win.set_position(PhysicalPosition::new(origin[0], origin[1]));
    let layout = Layout {
        edge,
        expanded,
        bar_left: (bar[0] - origin[0]) as f64 / screen.scale,
        bar_top: (bar[1] - origin[1]) as f64 / screen.scale,
    };
    let _ = app.emit_to(LABEL, "pulse://island", layout);
    layout
}

/// Appelé quand la fenêtre de la pastille bouge : si c'est l'utilisateur qui
/// la fait glisser, on suit sa position ; quand il la lâche, on la colle au
/// bord le plus proche et on enregistre sa place.
pub fn on_moved(app: &AppHandle, pos: PhysicalPosition<i32>) {
    let Some(win) = app.get_webview_window(LABEL) else { return };
    let island = app.state::<Island>();
    let generation = {
        let mut inner = island.inner.lock().unwrap();
        if inner.placed_at.is_some_and(|t| t.elapsed() < IGNORE_AFTER_PLACE) {
            return;
        }
        // La barre est fermée pendant un glissement : la fenêtre, c'est elle.
        let size = win.outer_size().unwrap_or_default();
        inner.center = Some([pos.x + size.width as i32 / 2, pos.y + size.height as i32 / 2]);
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
        let (all, primary) = screens(&win);
        let Some(center) = island.inner.lock().unwrap().center else { return };
        let (target, edge, screen) = compact_origin(Some(center), &all, primary);
        // Taille du bord choisi (horizontale ou verticale), puis petit
        // glissement jusqu'au bord.
        let size = bar_size(edge, screen.scale);
        let from = [center[0] - size[0] / 2, center[1] - size[1] / 2];
        island.inner.lock().unwrap().placed_at = Some(Instant::now());
        let _ = win.set_size(PhysicalSize::new(size[0] as u32, size[1] as u32));
        let _ = app.emit_to(LABEL, "pulse://island", Layout { edge, expanded: false, bar_left: 0.0, bar_top: 0.0 });
        for step in 1..=SNAP_STEPS {
            let x = from[0] + (target[0] - from[0]) * step / SNAP_STEPS;
            let y = from[1] + (target[1] - from[1]) * step / SNAP_STEPS;
            island.inner.lock().unwrap().placed_at = Some(Instant::now());
            let _ = win.set_position(PhysicalPosition::new(x, y));
            std::thread::sleep(SNAP_STEP);
        }
        let snapped = [target[0] + size[0] / 2, target[1] + size[1] / 2];
        island.inner.lock().unwrap().center = Some(snapped);
        let state = app.state::<Arc<AppState>>();
        let mut s = state.settings();
        if s.island_position != Some(snapped) {
            s.island_position = Some(snapped);
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
    app.state::<Island>().inner.lock().unwrap().center = None;
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
        let (pos, edge, s) = compact_origin(None, &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 100) / 2, 0]);
        assert_eq!(edge, Edge::Haut);
        assert_eq!(s, SCREEN);
    }

    #[test]
    fn sticks_to_the_nearest_edge_where_it_was_dropped() {
        // Près du haut : horizontale, collée en haut, même place de gauche à droite.
        assert_eq!(snap([600, 50], SCREEN), ([600 - 50, 0], Edge::Haut));
        // Près du bas.
        assert_eq!(snap([600, 1060], SCREEN), ([600 - 50, 1080 - 40], Edge::Bas));
        // Près de la gauche : verticale, même hauteur.
        assert_eq!(snap([30, 500], SCREEN), ([0, 500 - 38], Edge::Gauche));
        // Près de la droite.
        assert_eq!(snap([1900, 500], SCREEN), ([1920 - 54, 500 - 38], Edge::Droite));
        // Dans un coin : reste dans le coin, sans dépasser.
        assert_eq!(snap([1915, 1075], SCREEN), ([1920 - 100, 1080 - 40], Edge::Bas));
        assert_eq!(snap([5, 1070], SCREEN), ([0, 1080 - 76], Edge::Gauche));
    }

    #[test]
    fn position_on_a_missing_screen_falls_back_to_default() {
        // Deuxième écran débranché depuis.
        let (pos, _, _) = compact_origin(Some([2500, 300]), &[SCREEN], SCREEN);
        assert_eq!(pos, [(1920 - 100) / 2, 0]);
        // Écran présent, avec sa propre mise à l'échelle : collée en haut.
        let (pos, edge, s) = compact_origin(Some([2500, 300]), &[SCREEN, RIGHT], SCREEN);
        assert_eq!(pos, [2500 - 75, 0]);
        assert_eq!(edge, Edge::Haut);
        assert_eq!(s, RIGHT);
    }

    #[test]
    fn opens_towards_the_inside_of_the_screen() {
        // En haut : panneau en dessous, centré sur la barre.
        let (origin, size) = expanded_rect([500, 0], Edge::Haut, SCREEN);
        assert_eq!((origin, size), ([500 - (300 - 100) / 2, 0], [300, 290]));
        // En bas : au-dessus, le bas de la fenêtre reste le bas de la barre.
        let (origin, size) = expanded_rect([500, 1040], Edge::Bas, SCREEN);
        assert_eq!(origin[1] + size[1], 1080);
        // À gauche : à droite de la barre, centré sur elle en hauteur.
        let (origin, size) = expanded_rect([0, 462], Edge::Gauche, SCREEN);
        assert_eq!((origin, size), ([0, 462 + 38 - 145], [356, 290]));
        // À droite : à gauche de la barre, son bord droit reste celui de l'écran.
        let (origin, size) = expanded_rect([1920 - 54, 462], Edge::Droite, SCREEN);
        assert_eq!(origin[0] + size[0], 1920);
        // Contre un coin : la fenêtre ouverte reste sur l'écran.
        let (origin, _) = expanded_rect([0, 0], Edge::Haut, SCREEN);
        assert_eq!(origin, [0, 0]);
        let (origin, size) = expanded_rect([0, 1080 - 76], Edge::Gauche, SCREEN);
        assert_eq!(origin[1] + size[1], 1080);
    }
}
