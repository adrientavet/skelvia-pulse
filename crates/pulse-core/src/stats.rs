//! Calcul du temps passé, à partir des événements enregistrés.
//!
//! Trois sources, au format ActivityWatch :
//! - la fenêtre au premier plan (`app`, `title`) ;
//! - l'activité clavier et souris (`status` : `afk` ou `not-afk`) ;
//! - l'onglet actif du navigateur, si l'extension est installée (`url`,
//!   `title`, `incognito`).
//!
//! Un moment n'est compté que si une fenêtre était au premier plan **et**
//! que la souris ou le clavier avaient servi depuis moins que le délai
//! d'inactivité. Les fenêtres ouvertes en arrière-plan ne comptent jamais.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::apps::{browser_key, clean_title, friendly_name};
use crate::rules::{categorize, is_excluded, Context};
use crate::settings::Settings;
use crate::text::domain_of;

/// Un événement tel qu'il est stocké : un début, une durée, des données.
#[derive(Debug, Clone)]
pub struct RawEvent {
    pub start: DateTime<Utc>,
    pub duration: f64,
    pub data: Map<String, Value>,
}

impl RawEvent {
    pub fn end(&self) -> DateTime<Utc> {
        self.start + Duration::milliseconds((self.duration.max(0.0) * 1000.0) as i64)
    }
    fn str(&self, key: &str) -> &str {
        self.data.get(key).and_then(Value::as_str).unwrap_or("")
    }
    fn bool(&self, key: &str) -> bool {
        self.data.get(key).and_then(Value::as_bool).unwrap_or(false)
    }
}

/// Les événements d'une période, lus depuis la base locale.
#[derive(Debug, Clone, Default)]
pub struct Inputs {
    pub window: Vec<RawEvent>,
    pub afk: Vec<RawEvent>,
    /// Par navigateur (`chrome`, `edge`, `firefox`…).
    pub web: HashMap<String, Vec<RawEvent>>,
}

type Span = (DateTime<Utc>, DateTime<Utc>);

/// Un morceau de temps actif, rangé.
#[derive(Debug, Clone, Serialize)]
pub struct Segment {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub app_name: String,
    /// Titre nettoyé (vide si l'utilisateur n'enregistre pas les titres).
    pub title: String,
    pub domain: Option<String>,
    pub category: String,
}

impl Segment {
    pub fn secs(&self) -> f64 {
        (self.end - self.start).num_milliseconds() as f64 / 1000.0
    }
}

fn clip(span: Span, from: DateTime<Utc>, to: DateTime<Utc>) -> Option<Span> {
    let s = span.0.max(from);
    let e = span.1.min(to);
    (e > s).then_some((s, e))
}

/// Réunion triée de plages qui peuvent se chevaucher.
fn union(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_by_key(|s| s.0);
    let mut out: Vec<Span> = Vec::with_capacity(spans.len());
    for s in spans {
        match out.last_mut() {
            Some(last) if s.0 <= last.1 => last.1 = last.1.max(s.1),
            _ => out.push(s),
        }
    }
    out
}

/// Intersection d'une plage avec une liste triée de plages disjointes.
fn intersect(span: Span, sorted: &[Span]) -> Vec<Span> {
    // Première plage qui peut chevaucher : celle dont la fin dépasse le début.
    let first = sorted.partition_point(|s| s.1 <= span.0);
    sorted[first..]
        .iter()
        .take_while(|s| s.0 < span.1)
        .filter_map(|s| clip(*s, span.0, span.1))
        .collect()
}

/// Plages où l'utilisateur était actif (clavier ou souris utilisés).
fn active_spans(afk: &[RawEvent]) -> Vec<Span> {
    union(
        afk.iter()
            .filter(|e| e.str("status") == "not-afk")
            .map(|e| (e.start, e.end()))
            .filter(|s| s.1 > s.0)
            .collect(),
    )
}

/// Découpe la période en morceaux de temps actif, rangés par type de tâche.
/// Les moments exclus et la navigation privée sont retirés.
pub fn segments(
    inputs: &Inputs,
    settings: &Settings,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<Segment> {
    let active = active_spans(&inputs.afk);

    // Onglets par navigateur, triés, sans chevauchement (le plus récent gagne).
    let mut web: HashMap<&str, Vec<(Span, &RawEvent)>> = HashMap::new();
    for (browser, events) in &inputs.web {
        let mut v: Vec<(Span, &RawEvent)> =
            events.iter().map(|e| ((e.start, e.end()), e)).filter(|(s, _)| s.1 > s.0).collect();
        v.sort_by_key(|(s, _)| s.0);
        for i in 1..v.len() {
            if v[i].0 .0 < v[i - 1].0 .1 {
                v[i - 1].0 .1 = v[i].0 .0;
            }
        }
        v.retain(|(s, _)| s.1 > s.0);
        web.insert(browser.as_str(), v);
    }

    let mut out = Vec::new();
    let mut push = |start, end, app_exe: &str, app_name: &str, raw_title: &str, domain: Option<String>| {
        let ctx = Context { app_exe, app_name, domain: domain.as_deref(), title: raw_title };
        if is_excluded(&settings.exclusions, &ctx) {
            return;
        }
        let category = categorize(&settings.rules, &ctx).to_string();
        let title = if settings.record_titles { clean_title(raw_title, app_name) } else { String::new() };
        out.push(Segment { start, end, app_name: app_name.to_string(), title, domain, category });
    };

    let mut windows: Vec<&RawEvent> = inputs.window.iter().collect();
    windows.sort_by_key(|e| e.start);
    for w in windows {
        let Some(span) = clip((w.start, w.end()), from, to) else { continue };
        let app_exe = w.str("app");
        let app_name = friendly_name(app_exe);
        let win_title = w.str("title");
        for piece in intersect(span, &active) {
            let tabs = browser_key(app_exe).and_then(|b| web.get(b));
            let Some(tabs) = tabs else {
                push(piece.0, piece.1, app_exe, &app_name, win_title, None);
                continue;
            };
            // Navigateur : on prend le site de l'onglet actif quand on le connaît.
            let mut cursor = piece.0;
            let first = tabs.partition_point(|(s, _)| s.1 <= piece.0);
            for (tspan, tab) in tabs[first..].iter().take_while(|(s, _)| s.0 < piece.1) {
                let Some(t) = clip(*tspan, piece.0, piece.1) else { continue };
                if t.0 > cursor {
                    push(cursor, t.0, app_exe, &app_name, win_title, None);
                }
                if !tab.bool("incognito") {
                    let title = tab.str("title");
                    let title = if title.is_empty() { win_title } else { title };
                    push(t.0, t.1, app_exe, &app_name, title, domain_of(tab.str("url")));
                }
                cursor = t.1;
            }
            if cursor < piece.1 {
                push(cursor, piece.1, app_exe, &app_name, win_title, None);
            }
        }
    }
    out
}

/// Une ligne de classement : un nom et un temps.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Entry {
    pub label: String,
    pub secs: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryEntry {
    pub label: String,
    pub secs: f64,
    /// Applications les plus utilisées dans cette catégorie.
    pub apps: Vec<Entry>,
    /// Documents ou pages les plus longs dans cette catégorie.
    pub items: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayEntry {
    pub date: NaiveDate,
    pub secs: f64,
}

/// Le résumé d'une période, prêt à afficher.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub active_secs: f64,
    pub by_category: Vec<CategoryEntry>,
    pub by_app: Vec<Entry>,
    pub by_site: Vec<Entry>,
    pub top_items: Vec<Entry>,
    /// Temps actif par jour (heure locale du PC).
    pub days: Vec<DayEntry>,
    /// Des données de l'extension navigateur existent sur la période.
    pub has_web_data: bool,
}

fn ranked(map: HashMap<String, f64>, limit: usize) -> Vec<Entry> {
    let mut v: Vec<Entry> = map
        .into_iter()
        .filter(|(_, s)| *s >= 1.0)
        .map(|(label, secs)| Entry { label, secs })
        .collect();
    v.sort_by(|a, b| b.secs.total_cmp(&a.secs).then_with(|| a.label.cmp(&b.label)));
    v.truncate(limit);
    v
}

/// Libellé d'un document ou d'une page : le titre, ou le site à défaut.
fn item_label(s: &Segment) -> Option<String> {
    if !s.title.is_empty() {
        Some(s.title.clone())
    } else {
        s.domain.clone()
    }
}

pub fn summarize(
    inputs: &Inputs,
    settings: &Settings,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Summary {
    let segs = segments(inputs, settings, from, to);

    let mut active = 0.0;
    let mut cats: HashMap<String, (f64, HashMap<String, f64>, HashMap<String, f64>)> = HashMap::new();
    let mut apps: HashMap<String, f64> = HashMap::new();
    let mut sites: HashMap<String, f64> = HashMap::new();
    let mut items: HashMap<String, f64> = HashMap::new();
    let mut days: BTreeMap<NaiveDate, f64> = BTreeMap::new();

    for s in &segs {
        let secs = s.secs();
        active += secs;
        let c = cats.entry(s.category.clone()).or_default();
        c.0 += secs;
        *c.1.entry(s.app_name.clone()).or_default() += secs;
        if let Some(label) = item_label(s) {
            *c.2.entry(label.clone()).or_default() += secs;
            *items.entry(label).or_default() += secs;
        }
        *apps.entry(s.app_name.clone()).or_default() += secs;
        if let Some(d) = &s.domain {
            *sites.entry(d.clone()).or_default() += secs;
        }
        add_to_days(&mut days, s.start, s.end);
    }
    // Les jours sans activité apparaissent aussi, à zéro.
    if to > from {
        let first = from.with_timezone(&Local).date_naive();
        let last = (to - Duration::milliseconds(1)).with_timezone(&Local).date_naive();
        let mut d = first;
        while d <= last {
            days.entry(d).or_insert(0.0);
            d = match d.succ_opt() {
                Some(n) => n,
                None => break,
            };
        }
    }

    let mut by_category: Vec<CategoryEntry> = cats
        .into_iter()
        .map(|(label, (secs, a, i))| CategoryEntry { label, secs, apps: ranked(a, 3), items: ranked(i, 5) })
        .filter(|c| c.secs >= 1.0)
        .collect();
    by_category.sort_by(|a, b| b.secs.total_cmp(&a.secs).then_with(|| a.label.cmp(&b.label)));

    Summary {
        from,
        to,
        active_secs: active,
        by_category,
        by_app: ranked(apps, 15),
        by_site: ranked(sites, 15),
        top_items: ranked(items, 15),
        days: days.into_iter().map(|(date, secs)| DayEntry { date, secs }).collect(),
        has_web_data: inputs.web.values().any(|v| !v.is_empty()),
    }
}

/// Répartit un morceau de temps sur les jours (locaux) qu'il traverse.
fn add_to_days(days: &mut BTreeMap<NaiveDate, f64>, start: DateTime<Utc>, end: DateTime<Utc>) {
    let mut cur = start;
    while cur < end {
        let local = cur.with_timezone(&Local);
        let day = local.date_naive();
        let next_midnight = day
            .succ_opt()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .and_then(|n| Local.from_local_datetime(&n).earliest())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or(end);
        let stop = end.min(next_midnight.max(cur + Duration::seconds(1)));
        *days.entry(day).or_default() += (stop - cur).num_milliseconds() as f64 / 1000.0;
        cur = stop;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn t(min: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap() + Duration::minutes(min)
    }

    fn ev(start_min: i64, dur_min: f64, data: Value) -> RawEvent {
        RawEvent { start: t(start_min), duration: dur_min * 60.0, data: data.as_object().unwrap().clone() }
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.5
    }

    #[test]
    fn only_active_foreground_time_counts() {
        let inputs = Inputs {
            window: vec![
                ev(0, 60.0, json!({"app": "EXCEL.EXE", "title": "Devis Martin.xlsx - Excel"})),
                ev(60, 30.0, json!({"app": "OUTLOOK.EXE", "title": "Boîte de réception - Outlook"})),
            ],
            afk: vec![
                ev(0, 40.0, json!({"status": "not-afk"})),
                ev(40, 20.0, json!({"status": "afk"})),
                ev(60, 30.0, json!({"status": "not-afk"})),
            ],
            web: HashMap::new(),
        };
        let s = summarize(&inputs, &Settings::default(), t(0), t(120));
        assert!(approx(s.active_secs, 70.0 * 60.0));
        assert_eq!(s.by_category[0].label, "Devis et chiffrage");
        assert!(approx(s.by_category[0].secs, 40.0 * 60.0));
        assert_eq!(s.by_category[1].label, "Mails");
        assert_eq!(s.by_app[0].label, "Excel");
    }

    #[test]
    fn browser_time_uses_tabs_and_skips_incognito() {
        let mut web = HashMap::new();
        web.insert(
            "chrome".to_string(),
            vec![
                ev(0, 10.0, json!({"url": "https://www.pointp.fr/x", "title": "Point.P", "incognito": false})),
                ev(10, 5.0, json!({"url": "https://example.org", "title": "Secret", "incognito": true})),
            ],
        );
        let inputs = Inputs {
            window: vec![ev(0, 20.0, json!({"app": "chrome.exe", "title": "Onglet - Google Chrome"}))],
            afk: vec![ev(0, 20.0, json!({"status": "not-afk"}))],
            web,
        };
        let segs = segments(&inputs, &Settings::default(), t(0), t(30));
        // 10 min Point.P, 5 min privé retirées, 5 min sans onglet connu.
        let total: f64 = segs.iter().map(Segment::secs).sum();
        assert!(approx(total, 15.0 * 60.0));
        assert_eq!(segs[0].domain.as_deref(), Some("pointp.fr"));
        assert_eq!(segs[0].category, "Recherche de fournisseurs");
        assert!(segs.iter().all(|s| s.title != "Secret"));
    }

    #[test]
    fn exclusions_remove_time() {
        let mut settings = Settings::default();
        settings.exclusions.push(crate::settings::Exclusion {
            criterion: crate::settings::Criterion::App,
            value: "keepass".into(),
        });
        let inputs = Inputs {
            window: vec![ev(0, 10.0, json!({"app": "KeePass.exe", "title": "Base"}))],
            afk: vec![ev(0, 10.0, json!({"status": "not-afk"}))],
            web: HashMap::new(),
        };
        assert!(summarize(&inputs, &settings, t(0), t(10)).active_secs < 1.0);
    }

    #[test]
    fn titles_can_be_left_out() {
        let mut settings = Settings::default();
        settings.record_titles = false;
        let inputs = Inputs {
            window: vec![ev(0, 10.0, json!({"app": "WINWORD.EXE", "title": "Facture Dupont.docx - Word"}))],
            afk: vec![ev(0, 10.0, json!({"status": "not-afk"}))],
            web: HashMap::new(),
        };
        let s = summarize(&inputs, &settings, t(0), t(10));
        assert!(s.top_items.is_empty());
        assert_eq!(s.by_category[0].label, "Factures et relances");
    }

    #[test]
    fn clipping_to_period() {
        let inputs = Inputs {
            window: vec![ev(-30, 60.0, json!({"app": "EXCEL.EXE", "title": "x"}))],
            afk: vec![ev(-30, 60.0, json!({"status": "not-afk"}))],
            web: HashMap::new(),
        };
        let s = summarize(&inputs, &Settings::default(), t(0), t(60));
        assert!(approx(s.active_secs, 30.0 * 60.0));
        let s = summarize(&inputs, &Settings::default(), t(0), t(0) + Duration::days(3));
        assert!(s.days.len() >= 3, "les jours sans activité sont listés");
    }

    #[test]
    fn overlapping_afk_events_are_merged() {
        let spans = active_spans(&[
            ev(0, 10.0, json!({"status": "not-afk"})),
            ev(5, 10.0, json!({"status": "not-afk"})),
            ev(30, 5.0, json!({"status": "not-afk"})),
        ]);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0], (t(0), t(15)));
    }
}
