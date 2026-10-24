//! Boucle d'enregistrement, une fois par seconde.
//!
//! - Fenêtre au premier plan : programme et titre (sauf exclusions, et sans
//!   titre si l'utilisateur l'a choisi). Rien n'est enregistré pendant la
//!   pause ni pendant l'inactivité.
//! - Activité : « actif » jusqu'à la dernière action de la souris ou du
//!   clavier, « inactif » à partir de cette dernière action dès que le délai
//!   est dépassé (le délai d'attente lui-même n'est donc pas compté), ou dès
//!   que la session est verrouillée.

use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use pulse_core::apps::friendly_name;
use pulse_core::rules::{is_excluded, Context};
use pulse_core::stats::summarize;
use serde_json::{json, Map, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

const POLL_SECS: f64 = 1.0;
/// Un événement identique qui arrive moins de 2 s après le précédent le
/// prolonge (même principe qu'ActivityWatch : intervalle + 1 s).
const WINDOW_PULSETIME: f64 = POLL_SECS + 1.0;
/// Les événements d'activité sont prolongés en renvoyant le même début :
/// ils fusionnent toujours, une petite marge suffit.
const AFK_PULSETIME: f64 = 1.0;

#[derive(Clone, Copy)]
enum Activity {
    Unknown,
    Active { since: DateTime<Utc> },
    Idle { since: DateTime<Utc> },
}

fn data(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

/// Début de la journée en cours, à l'heure du PC.
pub fn start_of_today() -> DateTime<Utc> {
    let today = Local::now().date_naive();
    today
        .and_hms_opt(0, 0, 0)
        .and_then(|n| Local.from_local_datetime(&n).earliest())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("collecte".into())
        .spawn(move || run(app))
        .expect("impossible de démarrer la collecte");
}

fn run(app: AppHandle) {
    let state: Arc<AppState> = app.state::<Arc<AppState>>().inner().clone();
    state.store.ensure_buckets();

    let mut activity = Activity::Unknown;
    let mut tick: u64 = 0;
    let mut locked = false;

    loop {
        std::thread::sleep(StdDuration::from_secs_f64(POLL_SECS));
        tick += 1;
        let settings = state.settings();
        let now = Utc::now();

        // Rien n'est enregistré avant l'accord donné au premier lancement.
        if settings.paused || !settings.first_run_done {
            // La plage active en cours s'arrête à la dernière action déjà
            // enregistrée ; au retour, une nouvelle plage commencera.
            activity = Activity::Unknown;
            update_status(&app, &state, tick, true, false);
            continue;
        }

        if tick % 5 == 1 {
            locked = pulse_collector::is_session_locked();
        }
        let since_input = pulse_collector::seconds_since_last_input();
        let timeout = settings.idle_timeout_secs as f64;
        // Hors Windows, aucune mesure : on reste en « inconnu », rien n'est enregistré.
        let Some(since_input) = since_input else {
            update_status(&app, &state, tick, false, true);
            continue;
        };
        let last_input = now - Duration::milliseconds((since_input * 1000.0) as i64);
        let idle = locked || since_input >= timeout;
        let lock_now = locked && since_input < timeout;

        let (next, writes) = step(activity, now, last_input, idle, lock_now, tick.is_multiple_of(5));
        activity = next;
        for w in writes {
            let status = if w.afk { "afk" } else { "not-afk" };
            state.store.heartbeat(&state.store.afk_bucket, w.start, w.secs, data(json!({ "status": status })), AFK_PULSETIME);
        }
        if let Activity::Active { .. } = activity {
            record_window(&state, &settings, now);
        }

        if tick.is_multiple_of(60) {
            // Les onglets de navigation privée et les sites exclus envoyés
            // par l'extension sont effacés au fil de l'eau.
            state.store.purge(&settings, now - Duration::minutes(10));
        }
        update_status(&app, &state, tick, false, idle);
    }
}

/// Une écriture dans le seau d'activité.
#[derive(Debug, PartialEq)]
struct AfkWrite {
    start: DateTime<Utc>,
    secs: f64,
    afk: bool,
}

/// Passe d'un état d'activité au suivant et dit quoi écrire. Fonction pure,
/// testée plus bas.
///
/// - Actif : la plage active va de son début à la **dernière action**.
/// - Devenir inactif : l'inactivité commence à la dernière action (le délai
///   d'attente n'est donc pas compté comme du travail), ou maintenant si
///   c'est le verrouillage de la session qui la déclenche.
/// - Redevenir actif : l'inactivité s'arrête à la nouvelle action.
fn step(
    prev: Activity,
    now: DateTime<Utc>,
    last_input: DateTime<Utc>,
    idle: bool,
    lock_now: bool,
    refresh_idle: bool,
) -> (Activity, Vec<AfkWrite>) {
    let mut w = Vec::new();
    let next = match prev {
        Activity::Unknown if idle => Activity::Idle { since: now },
        Activity::Unknown => Activity::Active { since: last_input.min(now) },
        Activity::Idle { since } if !idle => {
            let end = last_input.max(since);
            w.push(AfkWrite { start: since, secs: secs(end - since), afk: true });
            Activity::Active { since: end }
        }
        Activity::Active { since } if idle => {
            let start = if lock_now { now } else { last_input.max(since) };
            w.push(AfkWrite { start: since, secs: secs(start - since), afk: false });
            w.push(AfkWrite { start, secs: secs(now - start), afk: true });
            return (Activity::Idle { since: start }, w);
        }
        other => other,
    };
    match next {
        Activity::Active { since } => {
            w.push(AfkWrite { start: since, secs: secs(last_input.max(since) - since), afk: false })
        }
        Activity::Idle { since } if refresh_idle => w.push(AfkWrite { start: since, secs: secs(now - since), afk: true }),
        _ => {}
    }
    (next, w)
}

fn secs(d: Duration) -> f64 {
    d.num_milliseconds().max(0) as f64 / 1000.0
}

fn record_window(state: &AppState, settings: &pulse_core::settings::Settings, now: DateTime<Utc>) {
    let Some(fg) = pulse_collector::foreground() else { return };
    let name = friendly_name(&fg.app);
    let ctx = Context { app_exe: &fg.app, app_name: &name, domain: None, title: &fg.title };
    if is_excluded(&settings.exclusions, &ctx) {
        return;
    }
    let title = if settings.record_titles { fg.title } else { String::new() };
    state.store.heartbeat(
        &state.store.window_bucket,
        now,
        0.0,
        data(json!({"app": fg.app, "title": title})),
        WINDOW_PULSETIME,
    );
}

fn update_status(app: &AppHandle, state: &AppState, tick: u64, paused: bool, idle: bool) {
    let mut status = {
        let mut st = state.status.lock().unwrap();
        st.paused = paused;
        st.idle = idle && !paused;
        st.clone()
    };
    // Le total du jour est recalculé toutes les 20 secondes.
    if tick % 20 == 1 {
        let from = start_of_today();
        let to = Utc::now();
        let settings = state.settings();
        let summary = summarize(&state.store.inputs(from, to), &settings, from, to);
        state.status.lock().unwrap().today_secs = summary.active_secs;
        status.today_secs = summary.active_secs;
    }
    let _ = app.emit("pulse://status", &status);
    crate::tray::refresh(app, &status);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap() + Duration::seconds(s)
    }

    /// Simule une suite de secondes : `inputs` donne l'instant de la
    /// dernière action à chaque seconde.
    fn simulate(inputs: &[i64], timeout: i64) -> Vec<AfkWrite> {
        let mut a = Activity::Unknown;
        let mut all = Vec::new();
        for (i, last) in inputs.iter().enumerate() {
            let now = t(i as i64);
            let idle = i as i64 - last >= timeout;
            let (n, w) = step(a, now, t(*last), idle, false, true);
            a = n;
            all.extend(w);
        }
        all
    }

    #[test]
    fn idle_wait_is_not_counted_as_work() {
        // Actif de 0 à 100 s, puis plus rien jusqu'à 400 s, délai 180 s.
        let mut inputs: Vec<i64> = (0..=100).collect();
        inputs.extend(std::iter::repeat_n(100, 300));
        let writes = simulate(&inputs, 180);
        let last_active = writes.iter().filter(|w| !w.afk).next_back().unwrap();
        assert_eq!(last_active.start, t(0));
        assert_eq!(last_active.secs, 100.0, "la plage active s'arrête à la dernière action");
        let afk = writes.iter().filter(|w| w.afk).next_back().unwrap();
        assert_eq!(afk.start, t(100), "l'inactivité commence à la dernière action");
    }

    #[test]
    fn coming_back_closes_idle_and_opens_active() {
        let mut inputs: Vec<i64> = (0..=10).collect();
        inputs.extend(std::iter::repeat_n(10, 200)); // inactif à partir de 190 s
        inputs.extend(211..=220); // retour à 211 s
        let writes = simulate(&inputs, 180);
        let closing = writes.iter().filter(|w| w.afk).next_back().unwrap();
        assert_eq!(closing.start, t(10));
        assert_eq!(closing.secs, 201.0, "l'inactivité s'arrête à la nouvelle action (211 s)");
        let active = writes.iter().filter(|w| !w.afk).next_back().unwrap();
        assert_eq!(active.start, t(211));
        assert_eq!(active.secs, 9.0);
    }

    #[test]
    fn lock_starts_idle_immediately() {
        let a = Activity::Active { since: t(0) };
        let (n, w) = step(a, t(60), t(55), true, true, false);
        assert!(matches!(n, Activity::Idle { since } if since == t(60)));
        assert_eq!(w[0], AfkWrite { start: t(0), secs: 60.0, afk: false });
    }
}
