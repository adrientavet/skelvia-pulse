//! Les actions que l'interface peut demander.

use std::path::PathBuf;
use std::sync::Arc;

use chrono::{Duration, Local, Utc};
use pulse_core::format::date_long;
use pulse_core::report::{render, Fonts, ReportOptions};
use pulse_core::settings::Settings;
use pulse_core::stats::{summarize, Summary};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

use crate::collector::start_of_today;
use crate::state::{AppState, Status};

type St<'a> = State<'a, Arc<AppState>>;

const FONT_DISPLAY: &[u8] = include_bytes!("../../src/fonts/bricolage-latin.woff2");
const FONT_TEXT: &[u8] = include_bytes!("../../src/fonts/dmsans-latin.woff2");

/// Adresses des pages de l'extension dans les magasins officiels (vérifiées
/// dans la documentation d'ActivityWatch). Edge installe les extensions du
/// Chrome Web Store.
const CHROME_STORE: &str = "https://chromewebstore.google.com/detail/activitywatch-web-watcher/nglaklhklhcoonedhgnpgddginnjdadi";
const FIREFOX_STORE: &str = "https://addons.mozilla.org/fr/firefox/addon/aw-watcher-web/";

fn bounds(range: &str) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
    let today = start_of_today();
    let from = match range {
        "week" => today - Duration::days(6),
        "month" => today - Duration::days(29),
        _ => today,
    };
    (from, Utc::now())
}

fn compute(state: &AppState, range: &str) -> Summary {
    let (from, to) = bounds(range);
    let settings = state.settings();
    summarize(&state.store.inputs(from, to), &settings, from, to)
}

#[tauri::command]
pub fn get_status(state: St) -> Status {
    state.status()
}

pub fn set_paused_inner(app: &AppHandle, paused: bool) {
    let state = app.state::<Arc<AppState>>();
    let mut s = state.settings();
    s.paused = paused;
    state.save_settings(s);
    let status = state.status();
    let _ = app.emit("pulse://status", &status);
    crate::tray::refresh(app, &status);
}

pub fn toggle_pause(app: &AppHandle) {
    let paused = app.state::<Arc<AppState>>().settings().paused;
    set_paused_inner(app, !paused);
}

#[tauri::command]
pub fn set_paused(app: AppHandle, paused: bool) {
    set_paused_inner(&app, paused);
}

#[tauri::command]
pub fn get_settings(state: St) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn default_rules() -> Vec<pulse_core::settings::Rule> {
    pulse_core::rules::default_rules()
}

/// Enregistre les réglages. Une nouvelle exclusion efface aussi ce qui
/// avait déjà été enregistré.
#[tauri::command]
pub fn save_settings(state: St, settings: Settings) -> Settings {
    let old = state.settings();
    let mut new = settings;
    // Ces deux valeurs ne se changent pas depuis l'écran des réglages.
    new.paused = old.paused;
    new.first_run_done = old.first_run_done;
    new.island_position = old.island_position;
    let saved = state.save_settings(new);
    if saved.exclusions != old.exclusions {
        let store = state.store.clone();
        let s = saved.clone();
        std::thread::spawn(move || store.purge(&s, Utc::now() - Duration::days(3650)));
    }
    saved
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryView {
    pub summary: Summary,
    pub period_label: String,
}

fn period_label(range: &str) -> String {
    let today = Local::now().date_naive();
    match range {
        "week" => format!("7 derniers jours, depuis le {}", date_long(today - Duration::days(6))),
        "month" => format!("30 derniers jours, depuis le {}", date_long(today - Duration::days(29))),
        _ => format!("Aujourd'hui, {}", date_long(today)),
    }
}

#[tauri::command]
pub async fn get_summary(app: AppHandle, range: String) -> Result<SummaryView, String> {
    let state = app.state::<Arc<AppState>>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || SummaryView {
        summary: compute(&state, &range),
        period_label: period_label(&range),
    })
    .await
    .map_err(|e| e.to_string())
}

fn downloads_dir() -> PathBuf {
    dirs::download_dir().or_else(dirs::document_dir).unwrap_or_else(std::env::temp_dir)
}

/// Écrit le rapport dans le dossier Téléchargements, l'ouvre dans le
/// navigateur, et renvoie son chemin.
#[tauri::command]
pub async fn save_report(
    app: AppHandle,
    range: String,
    include_items: bool,
    include_sites: bool,
    include_audit: bool,
) -> Result<String, String> {
    let state = app.state::<Arc<AppState>>().inner().clone();
    let path = tauri::async_runtime::spawn_blocking(move || -> Result<PathBuf, String> {
        let summary = compute(&state, &range);
        let opts = ReportOptions { include_items, include_sites, include_audit, generated_at: Utc::now() };
        let html = render(&summary, &opts, &Fonts { display_woff2: FONT_DISPLAY, text_woff2: FONT_TEXT });
        let day = Local::now().format("%Y-%m-%d").to_string();
        let dir = downloads_dir();
        let mut path = dir.join(format!("Rapport SKELVIA Pulse {day}.html"));
        let mut n = 2;
        while path.exists() {
            path = dir.join(format!("Rapport SKELVIA Pulse {day} ({n}).html"));
            n += 1;
        }
        std::fs::write(&path, html).map_err(|e| format!("Le rapport n'a pas pu être écrit : {e}"))?;
        Ok(path)
    })
    .await
    .map_err(|e| e.to_string())??;
    let shown = path.to_string_lossy().to_string();
    let _ = app.opener().open_path(shown.clone(), None::<&str>);
    Ok(shown)
}

/// `scope` : `hour` (la dernière heure), `today` (aujourd'hui) ou `all`.
#[tauri::command]
pub async fn erase(app: AppHandle, scope: String) -> Result<(), String> {
    let state = app.state::<Arc<AppState>>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || match scope.as_str() {
        "hour" => state.store.erase_since(Utc::now() - Duration::hours(1)),
        "today" => state.store.erase_since(start_of_today()),
        "all" => state.store.erase_all(),
        _ => {}
    })
    .await
    .map_err(|e| e.to_string())?;
    let st = app.state::<Arc<AppState>>();
    st.status.lock().unwrap().today_secs = 0.0;
    let _ = app.emit("pulse://status", &st.status());
    Ok(())
}

#[tauri::command]
pub fn complete_first_run(app: AppHandle, autostart: bool) {
    let state = app.state::<Arc<AppState>>();
    let mut s = state.settings();
    s.first_run_done = true;
    s.paused = false;
    state.save_settings(s);
    let _ = set_autostart_inner(&app, autostart);
    if let Some(island) = app.get_webview_window(crate::island::LABEL) {
        crate::island::place(&app, &island, false);
        let _ = island.show();
    }
    let _ = app.emit("pulse://status", &state.status());
}

fn set_autostart_inner(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let res = if enabled { launcher.enable() } else { launcher.disable() };
    res.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    set_autostart_inner(&app, enabled)
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn open_extension_page(app: AppHandle, browser: String) -> Result<(), String> {
    let url = if browser == "firefox" { FIREFOX_STORE } else { CHROME_STORE };
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_data_folder(app: AppHandle) -> Result<(), String> {
    let dir = app.state::<Arc<AppState>>().data_dir.clone();
    app.opener().open_path(dir.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_audit_page(app: AppHandle) -> Result<(), String> {
    app.opener().open_url(pulse_core::report::AUDIT_URL, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    crate::show_main(&app);
}

/// Ouvre ou referme la pastille.
#[tauri::command]
pub fn island_expand(app: AppHandle, expanded: bool) -> Option<crate::island::Layout> {
    app.get_webview_window(crate::island::LABEL).map(|win| crate::island::place(&app, &win, expanded))
}

/// Forme de la pastille (bord, horizontale ou verticale), demandée par
/// l'interface à son chargement.
#[tauri::command]
pub fn island_layout(app: AppHandle) -> Option<crate::island::Layout> {
    island_expand(app, false)
}

/// Remet la pastille en haut, au centre de l'écran principal.
#[tauri::command]
pub fn reset_island(app: AppHandle) {
    crate::island::reset(&app);
}

/// Version installée, affichée dans les réglages.
#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Vérification demandée depuis les réglages. Renvoie un message à afficher.
#[tauri::command]
pub async fn check_updates_now(app: AppHandle) -> Result<String, String> {
    match crate::updater::check(&app).await {
        Ok(Some(v)) => Ok(format!("La version {v} est disponible.")),
        Ok(None) => Ok(format!("Vous avez la dernière version ({}).", app.package_info().version)),
        Err(e) => Err(format!("Vérification impossible pour l'instant (pas d'internet ?). Détail : {e}")),
    }
}

/// Télécharge et installe la nouvelle version (après un clic de l'utilisateur).
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    crate::updater::install(&app).await
}
