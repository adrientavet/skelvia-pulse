//! Mises à jour : SKELVIA Pulse regarde de temps en temps s'il existe une
//! nouvelle version et le signale dans la pastille. Rien ne s'installe sans
//! un clic de l'utilisateur.
//!
//! C'est la seule connexion à internet de l'application. Elle télécharge un
//! petit fichier public (`latest.json`) sur GitHub, puis, après le clic,
//! l'installeur. Aucune mesure n'est envoyée, et l'adresse appelée ne
//! contient ni la version ni un identifiant. L'installeur téléchargé doit
//! porter la signature de la clé de SKELVIA, sinon il est refusé. La
//! vérification peut être coupée dans les réglages.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::state::AppState;

/// Première vérification peu après le démarrage, puis toutes les 6 heures.
const FIRST_CHECK: Duration = Duration::from_secs(30);
const EVERY: Duration = Duration::from_secs(6 * 3600);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Step {
    /// Une nouvelle version attend le clic de l'utilisateur.
    Disponible,
    Telechargement,
    /// Le téléchargement est fini : l'application va se fermer pour
    /// laisser l'installeur travailler, puis se relancer.
    Installation,
    Erreur,
}

/// Ce que l'interface affiche d'une mise à jour.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub version: String,
    /// Nouveautés de la version, en quelques lignes.
    pub notes: Option<String>,
    pub step: Step,
    /// Avancement du téléchargement, de 0 à 1, s'il est connu.
    pub progress: Option<f64>,
    pub error: Option<String>,
}

/// La mise à jour trouvée, gardée jusqu'au clic.
#[derive(Default)]
pub struct Pending(Mutex<Option<Update>>);

fn publish(app: &AppHandle, update: Option<UpdateStatus>) {
    let state = app.state::<Arc<AppState>>();
    let status = {
        let mut st = state.status.lock().unwrap();
        st.update = update;
        st.clone()
    };
    let _ = app.emit("pulse://status", &status);
}

fn modify(app: &AppHandle, f: impl FnOnce(&mut UpdateStatus)) {
    let state = app.state::<Arc<AppState>>();
    let current = state.status.lock().unwrap().update.clone();
    if let Some(mut u) = current {
        f(&mut u);
        publish(app, Some(u));
    }
}

/// Demande à GitHub s'il existe une version plus récente. Renvoie la
/// version trouvée, ou `None` si l'application est à jour.
pub async fn check(app: &AppHandle) -> Result<Option<String>, String> {
    let found = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let Some(update) = found else {
        return Ok(None);
    };
    let version = update.version.clone();
    // Un téléchargement en cours n'est pas interrompu par une vérification.
    let busy = app
        .state::<Arc<AppState>>()
        .status()
        .update
        .is_some_and(|u| matches!(u.step, Step::Telechargement | Step::Installation));
    if !busy {
        let notes = update.body.clone().filter(|n| !n.trim().is_empty());
        *app.state::<Pending>().0.lock().unwrap() = Some(update);
        publish(
            app,
            Some(UpdateStatus { version: version.clone(), notes, step: Step::Disponible, progress: None, error: None }),
        );
    }
    Ok(Some(version))
}

/// Vérifie au démarrage puis régulièrement, si l'utilisateur ne l'a pas coupé.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        loop {
            // Rien n'est demandé à GitHub avant l'accord du premier lancement.
            let s = app.state::<Arc<AppState>>().settings();
            if s.check_updates && s.first_run_done {
                if let Err(e) = tauri::async_runtime::block_on(check(&app)) {
                    // Pas d'internet, GitHub injoignable… on réessaiera plus tard.
                    log::warn!("Vérification des mises à jour impossible : {e}");
                }
            }
            std::thread::sleep(EVERY);
        }
    });
}

/// Télécharge et installe la mise à jour trouvée. Sous Windows,
/// l'application se ferme pendant l'installation, puis l'installeur la
/// relance. Les mesures et les réglages restent dans leur dossier.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let Some(update) = app.state::<Pending>().0.lock().unwrap().clone() else {
        return Err("Aucune mise à jour à installer.".into());
    };
    modify(app, |u| {
        u.step = Step::Telechargement;
        u.progress = Some(0.0);
        u.error = None;
    });

    let mut received = 0usize;
    let mut last_shown = -1i64;
    let progress_app = app.clone();
    let done_app = app.clone();
    let result = update
        .download_and_install(
            move |chunk, total| {
                received += chunk;
                if let Some(total) = total.filter(|t| *t > 0) {
                    let p = (received as f64 / total as f64).min(1.0);
                    // On ne prévient l'interface qu'à chaque pour cent.
                    let pct = (p * 100.0) as i64;
                    if pct != last_shown {
                        last_shown = pct;
                        modify(&progress_app, |u| u.progress = Some(p));
                    }
                }
            },
            move || {
                // Les dernières secondes de mesure sont écrites avant la fermeture.
                let _ = done_app.state::<Arc<AppState>>().store.ds.force_commit();
                modify(&done_app, |u| {
                    u.step = Step::Installation;
                    u.progress = Some(1.0);
                });
            },
        )
        .await;

    match result {
        // Sous Windows, on n'arrive pas ici : l'installeur a pris la main.
        Ok(()) => {
            app.restart();
        }
        Err(e) => {
            let message = format!("La mise à jour n'a pas pu être installée ({e}). Vous pouvez réessayer plus tard.");
            modify(app, |u| {
                u.step = Step::Erreur;
                u.progress = None;
                u.error = Some(message.clone());
            });
            Err(message)
        }
    }
}
