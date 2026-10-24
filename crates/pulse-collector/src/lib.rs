//! Ce que SKELVIA Pulse lit sur le PC, et rien de plus :
//!
//! - le programme et le titre de la fenêtre **au premier plan** ;
//! - le temps écoulé depuis la dernière action de la souris ou du clavier
//!   (un nombre de secondes : ni les touches, ni les clics, ni la position
//!   de la souris ne sont lus) ;
//! - si la session Windows est verrouillée ;
//! - seulement pendant que l'on fait glisser la pastille : si un bouton de la
//!   souris est encore enfoncé, pour savoir quand on la lâche (rien n'est
//!   enregistré).
//!
//! Pas de capture d'écran, pas d'écoute du clavier, pas de lecture du
//! contenu des fenêtres.

/// La fenêtre au premier plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foreground {
    /// Nom du programme, par exemple `EXCEL.EXE`.
    pub app: String,
    /// Titre de la fenêtre, tel qu'affiché dans sa barre de titre.
    pub title: String,
}

#[cfg(windows)]
mod windows_impl;
#[cfg(windows)]
pub use windows_impl::{foreground, is_session_locked, mouse_button_down, seconds_since_last_input};

// Hors Windows (développement sous Linux ou macOS) : rien n'est lu. Les
// fonctions existent pour que le reste de l'application se compile et se
// teste, mais elles ne renvoient aucune donnée.
#[cfg(not(windows))]
pub fn foreground() -> Option<Foreground> {
    None
}

#[cfg(not(windows))]
pub fn seconds_since_last_input() -> Option<f64> {
    None
}

#[cfg(not(windows))]
pub fn is_session_locked() -> bool {
    false
}

#[cfg(not(windows))]
pub fn mouse_button_down() -> bool {
    false
}
