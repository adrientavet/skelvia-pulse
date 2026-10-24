//! État partagé de l'application, et fichiers sur le disque.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use pulse_core::settings::Settings;
use serde::Serialize;

use crate::store::Store;

/// Où en est l'enregistrement, pour la pastille et l'écran Aujourd'hui.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub paused: bool,
    /// Aucune action de la souris ou du clavier depuis le délai choisi, ou
    /// session verrouillée.
    pub idle: bool,
    /// Temps actif compté aujourd'hui, en secondes.
    pub today_secs: f64,
    /// Le serveur local pour l'extension navigateur fonctionne.
    pub server_ok: bool,
    /// Message à montrer si le serveur local n'a pas pu démarrer.
    pub server_message: Option<String>,
    pub first_run_done: bool,
    /// Nouvelle version trouvée, et où en est son installation.
    pub update: Option<crate::updater::UpdateStatus>,
}

pub struct AppState {
    pub store: Store,
    pub settings: Mutex<Settings>,
    pub status: Mutex<Status>,
    /// Lu par le serveur local pour refuser les données de l'extension
    /// pendant la pause.
    pub paused: Arc<AtomicBool>,
    pub data_dir: PathBuf,
}

/// Dossier des données : `%LOCALAPPDATA%\SKELVIA Pulse` sous Windows.
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("SKELVIA Pulse")
}

fn settings_path(dir: &std::path::Path) -> PathBuf {
    dir.join("reglages.json")
}

pub fn load_settings(dir: &std::path::Path) -> Settings {
    std::fs::read_to_string(settings_path(dir))
        .ok()
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default()
        .sanitized()
}

impl AppState {
    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn save_settings(&self, new: Settings) -> Settings {
        let new = new.sanitized();
        // Avant l'accord du premier lancement, rien n'est enregistré.
        self.paused.store(new.paused || !new.first_run_done, Ordering::SeqCst);
        *self.settings.lock().unwrap() = new.clone();
        let path = settings_path(&self.data_dir);
        match serde_json::to_string_pretty(&new) {
            Ok(json) => {
                // Écriture dans un fichier temporaire puis renommage : un
                // arrêt brutal du PC ne laisse pas un fichier à moitié écrit.
                let tmp = path.with_extension("json.tmp");
                if std::fs::write(&tmp, json).and_then(|_| std::fs::rename(&tmp, &path)).is_err() {
                    log::error!("Réglages non enregistrés dans {}", path.display());
                }
            }
            Err(e) => log::error!("Réglages illisibles : {e}"),
        }
        {
            let mut st = self.status.lock().unwrap();
            st.paused = new.paused;
            st.first_run_done = new.first_run_done;
        }
        new
    }

    pub fn status(&self) -> Status {
        self.status.lock().unwrap().clone()
    }
}
