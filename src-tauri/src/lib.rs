//! SKELVIA Pulse : voir où part son temps sur son PC, sans rien envoyer.
//!
//! Tout reste sur le PC : base locale (moteur ActivityWatch), réglages dans
//! un fichier JSON, rapport écrit dans le dossier Téléchargements. Aucun
//! compte, aucun serveur distant, aucune télémétrie.

mod collector;
mod commands;
mod island;
mod server;
mod state;
mod store;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, WindowEvent};

use crate::state::{AppState, Status};
use crate::store::Store;

/// Argument ajouté au lancement automatique avec Windows : on démarre alors
/// discrètement (pastille et icône seulement, sans ouvrir la fenêtre).
const AUTOSTART_ARG: &str = "--au-demarrage";

pub fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Identifiant aléatoire de cette installation, demandé par le serveur
/// ActivityWatch. Il reste sur le PC et n'est envoyé nulle part.
fn device_id(dir: &std::path::Path) -> String {
    let path = dir.join("identifiant-installation");
    if let Ok(id) = std::fs::read_to_string(&path) {
        if !id.trim().is_empty() {
            return id.trim().to_string();
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let _ = std::fs::write(&path, &id);
    id
}

pub fn run() {
    let dir = state::data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let settings = state::load_settings(&dir);

    let db_path = dir.join("donnees.sqlite");
    let ds = aw_datastore::Datastore::new(db_path.to_string_lossy().to_string(), false);
    let hostname = gethostname::gethostname().to_string_lossy().to_string();
    let store = Store::new(ds.clone(), &hostname);

    let paused = Arc::new(AtomicBool::new(settings.paused || !settings.first_run_done));
    let server = server::start(ds, device_id(&dir), paused.clone());
    let status = Status {
        paused: settings.paused,
        server_ok: server.is_ok(),
        server_message: server.err(),
        first_run_done: settings.first_run_done,
        ..Default::default()
    };
    let first_run_done = settings.first_run_done;
    let state = Arc::new(AppState {
        store,
        settings: Mutex::new(settings),
        status: Mutex::new(status),
        paused,
        data_dir: dir,
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .manage(island::Island::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::set_paused,
            commands::get_settings,
            commands::default_rules,
            commands::save_settings,
            commands::get_summary,
            commands::save_report,
            commands::erase,
            commands::complete_first_run,
            commands::set_autostart,
            commands::get_autostart,
            commands::open_extension_page,
            commands::open_data_folder,
            commands::open_audit_page,
            commands::show_main_window,
            commands::island_expand,
            commands::reset_island,
        ])
        .setup(move |app| {
            tray::create(app.handle())?;
            if let Some(island) = app.get_webview_window(island::LABEL) {
                island::place(app.handle(), &island, false);
                if first_run_done {
                    let _ = island.show();
                } else {
                    let _ = island.hide();
                }
            }
            let at_login = std::env::args().any(|a| a == AUTOSTART_ARG);
            if !first_run_done || !at_login {
                show_main(app.handle());
            }
            collector::spawn(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // La pastille a été déplacée (glissée par l'utilisateur).
            if window.label() == island::LABEL {
                if let WindowEvent::Moved(pos) = event {
                    island::on_moved(window.app_handle(), *pos);
                }
            }
            // Fermer la fenêtre principale la cache seulement : l'enregistrement
            // continue, la pastille et l'icône restent visibles.
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("SKELVIA Pulse n'a pas pu démarrer");
}
