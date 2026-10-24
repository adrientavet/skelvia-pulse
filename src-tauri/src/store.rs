//! Accès à la base locale (moteur ActivityWatch, fichier SQLite sur le PC).
//!
//! Les données gardent le format d'ActivityWatch : fenêtre au premier plan
//! (`currentwindow`), activité (`afkstatus`) et, si l'extension est
//! installée, onglet actif (`web.tab.current`).

use std::collections::HashMap;

use aw_datastore::{Datastore, DatastoreError};
use aw_models::{Bucket, Event};
use chrono::{DateTime, Duration, Utc};
use pulse_core::apps::{browser_key, friendly_name};
use pulse_core::rules::{is_excluded, Context};
use pulse_core::settings::Settings;
use pulse_core::stats::{Inputs, RawEvent};
use pulse_core::text::domain_of;
use serde_json::{Map, Value};

pub const CLIENT: &str = "skelvia-pulse";
const WEB_TYPE: &str = "web.tab.current";
const WEB_PREFIX: &str = "aw-watcher-web-";

#[derive(Clone)]
pub struct Store {
    pub ds: Datastore,
    pub window_bucket: String,
    pub afk_bucket: String,
    hostname: String,
}

fn to_raw(e: &Event) -> RawEvent {
    RawEvent {
        start: e.timestamp,
        duration: e.duration.num_milliseconds() as f64 / 1000.0,
        data: e.data.clone(),
    }
}

impl Store {
    pub fn new(ds: Datastore, hostname: &str) -> Self {
        Store {
            ds,
            window_bucket: format!("aw-watcher-window_{hostname}"),
            afk_bucket: format!("aw-watcher-afk_{hostname}"),
            hostname: hostname.to_string(),
        }
    }

    /// Crée les deux « seaux » de données s'ils n'existent pas encore.
    pub fn ensure_buckets(&self) {
        for (id, kind) in [(&self.window_bucket, "currentwindow"), (&self.afk_bucket, "afkstatus")] {
            let bucket = Bucket {
                bid: None,
                id: id.clone(),
                _type: kind.to_string(),
                client: CLIENT.to_string(),
                hostname: self.hostname.clone(),
                created: Some(Utc::now()),
                data: Map::new(),
                metadata: Default::default(),
                events: None,
                last_updated: None,
            };
            match self.ds.create_bucket(&bucket) {
                Ok(()) | Err(DatastoreError::BucketAlreadyExists(_)) => {}
                Err(e) => log::error!("Création du seau {id} impossible : {e:?}"),
            }
        }
    }

    /// Ajoute (ou prolonge) un événement. Un événement identique au
    /// précédent et proche dans le temps le prolonge au lieu d'en créer un.
    pub fn heartbeat(&self, bucket: &str, start: DateTime<Utc>, secs: f64, data: Map<String, Value>, pulsetime: f64) {
        let ev = Event::new(start, Duration::milliseconds((secs.max(0.0) * 1000.0) as i64), data);
        if let Err(e) = self.ds.heartbeat(bucket, ev.clone(), pulsetime) {
            if matches!(e, DatastoreError::NoSuchBucket(_)) {
                // Seau effacé entre-temps (effacement des données) : on le recrée.
                self.ensure_buckets();
                let _ = self.ds.heartbeat(bucket, ev, pulsetime);
            } else {
                log::warn!("Enregistrement impossible dans {bucket} : {e:?}");
            }
        }
    }

    fn web_buckets(&self) -> Vec<String> {
        match self.ds.get_buckets() {
            Ok(b) => b
                .into_values()
                .filter(|b| b._type == WEB_TYPE || b.id.starts_with(WEB_PREFIX))
                .map(|b| b.id)
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Nom du navigateur d'après le nom du seau : `aw-watcher-web-edge_PC` → `edge`.
    fn browser_of(bucket_id: &str) -> String {
        bucket_id
            .strip_prefix(WEB_PREFIX)
            .and_then(|r| r.split('_').next())
            .unwrap_or("inconnu")
            .to_string()
    }

    fn events(&self, bucket: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<Event> {
        self.ds.get_events(bucket, Some(from), Some(to), None).unwrap_or_default()
    }

    /// Tout ce qu'il faut pour calculer le résumé d'une période.
    pub fn inputs(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Inputs {
        let mut web: HashMap<String, Vec<RawEvent>> = HashMap::new();
        for b in self.web_buckets() {
            web.entry(Self::browser_of(&b))
                .or_default()
                .extend(self.events(&b, from, to).iter().map(to_raw));
        }
        Inputs {
            window: self.events(&self.window_bucket, from, to).iter().map(to_raw).collect(),
            afk: self.events(&self.afk_bucket, from, to).iter().map(to_raw).collect(),
            web,
        }
    }

    /// Efface ce qui correspond aux exclusions, et les onglets de navigation
    /// privée, entre `from` et maintenant. Les fenêtres de navigateur qui
    /// recouvrent surtout un onglet effacé sont effacées aussi (leur titre
    /// est souvent celui de la page).
    pub fn purge(&self, settings: &Settings, from: DateTime<Utc>) {
        let to = Utc::now() + Duration::minutes(1);
        let mut removed_tabs: HashMap<String, Vec<(DateTime<Utc>, DateTime<Utc>)>> = HashMap::new();

        for b in self.web_buckets() {
            let browser = Self::browser_of(&b);
            let mut ids = Vec::new();
            for e in self.events(&b, from, to) {
                let data = &e.data;
                let url = data.get("url").and_then(Value::as_str).unwrap_or("");
                let title = data.get("title").and_then(Value::as_str).unwrap_or("");
                let incognito = data.get("incognito").and_then(Value::as_bool).unwrap_or(false);
                let domain = domain_of(url);
                let ctx = Context { app_exe: "", app_name: "", domain: domain.as_deref(), title };
                if incognito || is_excluded(&settings.exclusions, &ctx) {
                    if let Some(id) = e.id {
                        ids.push(id);
                    }
                    removed_tabs.entry(browser.clone()).or_default().push((e.timestamp, e.timestamp + e.duration));
                }
            }
            if !ids.is_empty() {
                let _ = self.ds.delete_events_by_id(&b, ids);
            }
        }

        let mut ids = Vec::new();
        for e in self.events(&self.window_bucket, from, to) {
            let app = e.data.get("app").and_then(Value::as_str).unwrap_or("");
            let title = e.data.get("title").and_then(Value::as_str).unwrap_or("");
            let name = friendly_name(app);
            let ctx = Context { app_exe: app, app_name: &name, domain: None, title };
            let mut drop = is_excluded(&settings.exclusions, &ctx);
            if !drop {
                if let Some(spans) = browser_key(app).and_then(|k| removed_tabs.get(k)) {
                    let (s, end) = (e.timestamp, e.timestamp + e.duration);
                    let total = (end - s).num_milliseconds().max(1) as f64;
                    let covered: i64 = spans
                        .iter()
                        .map(|(a, b)| (end.min(*b) - s.max(*a)).num_milliseconds().max(0))
                        .sum();
                    drop = covered as f64 / total >= 0.5;
                }
            }
            if drop {
                if let Some(id) = e.id {
                    ids.push(id);
                }
            }
        }
        if !ids.is_empty() {
            let _ = self.ds.delete_events_by_id(&self.window_bucket, ids);
        }
    }

    /// Efface les événements qui commencent après `from`, dans tous les seaux.
    pub fn erase_since(&self, from: DateTime<Utc>) {
        let to = Utc::now() + Duration::minutes(1);
        let mut buckets = self.web_buckets();
        buckets.push(self.window_bucket.clone());
        buckets.push(self.afk_bucket.clone());
        for b in buckets {
            let ids: Vec<i64> = self
                .ds
                .get_events_unclipped(&b, Some(from), Some(to), None)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|e| e.id)
                .collect();
            if !ids.is_empty() {
                let _ = self.ds.delete_events_by_id(&b, ids);
            }
        }
        let _ = self.ds.force_commit();
    }

    /// Efface tout : chaque seau et ses événements.
    pub fn erase_all(&self) {
        if let Ok(buckets) = self.ds.get_buckets() {
            for id in buckets.into_keys() {
                if let Err(e) = self.ds.delete_bucket(&id) {
                    log::error!("Effacement du seau {id} impossible : {e:?}");
                }
            }
        }
        let _ = self.ds.force_commit();
        self.ensure_buckets();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pulse_core::settings::{Criterion, Exclusion};
    use pulse_core::stats::summarize;
    use serde_json::json;

    fn t(min: i64) -> DateTime<Utc> {
        Utc::now() - Duration::hours(2) + Duration::minutes(min)
    }

    fn data(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    fn store() -> Store {
        let s = Store::new(Datastore::new_in_memory(false), "PC-TEST");
        s.ensure_buckets();
        s
    }

    fn web_bucket(s: &Store) -> String {
        let id = "aw-watcher-web-chrome_PC-TEST".to_string();
        let b = Bucket {
            bid: None,
            id: id.clone(),
            _type: WEB_TYPE.into(),
            client: "aw-client-web".into(),
            hostname: "PC-TEST".into(),
            created: Some(Utc::now()),
            data: Map::new(),
            metadata: Default::default(),
            events: None,
            last_updated: None,
        };
        s.ds.create_bucket(&b).unwrap();
        id
    }

    #[test]
    fn heartbeats_with_same_start_extend_one_event() {
        let s = store();
        for i in 0..5 {
            s.heartbeat(&s.afk_bucket, t(0), (i * 60) as f64, data(json!({"status": "not-afk"})), 1.0);
        }
        let ev = s.ds.get_events(&s.afk_bucket, None, None, None).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].duration.num_seconds(), 240);
    }

    #[test]
    fn window_heartbeats_merge_while_unchanged() {
        let s = store();
        for i in 0..10 {
            s.heartbeat(&s.window_bucket, t(0) + Duration::seconds(i), 0.0, data(json!({"app": "EXCEL.EXE", "title": "Devis.xlsx - Excel"})), 2.0);
        }
        s.heartbeat(&s.window_bucket, t(0) + Duration::seconds(10), 0.0, data(json!({"app": "OUTLOOK.EXE", "title": "Boîte"})), 2.0);
        let ev = s.ds.get_events(&s.window_bucket, None, None, None).unwrap();
        assert_eq!(ev.len(), 2);
    }

    #[test]
    fn inputs_feed_the_summary() {
        let s = store();
        s.heartbeat(&s.afk_bucket, t(0), 600.0, data(json!({"status": "not-afk"})), 1.0);
        s.heartbeat(&s.window_bucket, t(0), 600.0, data(json!({"app": "EXCEL.EXE", "title": "Devis.xlsx - Excel"})), 2.0);
        let inputs = s.inputs(t(-10), t(30));
        let sum = summarize(&inputs, &Settings::default(), t(-10), t(30));
        assert!((sum.active_secs - 600.0).abs() < 1.0);
        assert_eq!(sum.by_category[0].label, "Devis et chiffrage");
    }

    #[test]
    fn purge_removes_incognito_excluded_and_matching_browser_windows() {
        let s = store();
        let web = web_bucket(&s);
        s.heartbeat(&web, t(0), 300.0, data(json!({"url": "https://www.ma-banque.fr/compte", "title": "Mon compte", "incognito": false, "audible": false, "tabCount": 1})), 1.0);
        s.heartbeat(&web, t(5), 300.0, data(json!({"url": "https://example.org", "title": "Privé", "incognito": true, "audible": false, "tabCount": 1})), 1.0);
        s.heartbeat(&web, t(10), 300.0, data(json!({"url": "https://www.pointp.fr", "title": "Point.P", "incognito": false, "audible": false, "tabCount": 1})), 1.0);
        s.heartbeat(&s.window_bucket, t(0), 300.0, data(json!({"app": "chrome.exe", "title": "Mon compte - Google Chrome"})), 2.0);
        s.heartbeat(&s.window_bucket, t(10), 300.0, data(json!({"app": "chrome.exe", "title": "Point.P - Google Chrome"})), 2.0);
        s.heartbeat(&s.window_bucket, t(20), 300.0, data(json!({"app": "KeePass.exe", "title": "Coffre"})), 2.0);

        let mut settings = Settings::default();
        settings.exclusions.push(Exclusion { criterion: Criterion::Site, value: "ma-banque.fr".into() });
        settings.exclusions.push(Exclusion { criterion: Criterion::App, value: "keepass".into() });
        s.purge(&settings, t(-60));

        let web_left = s.ds.get_events(&web, None, None, None).unwrap();
        assert_eq!(web_left.len(), 1);
        assert_eq!(web_left[0].data["title"], "Point.P");
        let win_left = s.ds.get_events(&s.window_bucket, None, None, None).unwrap();
        assert_eq!(win_left.len(), 1);
        assert_eq!(win_left[0].data["title"], "Point.P - Google Chrome");
    }

    #[test]
    fn erase_since_and_erase_all() {
        let s = store();
        let old = Utc.with_ymd_and_hms(2026, 1, 5, 9, 0, 0).unwrap();
        s.heartbeat(&s.window_bucket, old, 60.0, data(json!({"app": "EXCEL.EXE", "title": "Ancien"})), 2.0);
        s.heartbeat(&s.window_bucket, Utc::now() - Duration::minutes(5), 60.0, data(json!({"app": "EXCEL.EXE", "title": "Récent"})), 2.0);
        s.erase_since(Utc::now() - Duration::hours(1));
        let left = s.ds.get_events(&s.window_bucket, None, None, None).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].data["title"], "Ancien");

        let web = web_bucket(&s);
        s.heartbeat(&web, old, 60.0, data(json!({"url": "https://a.fr", "title": "A"})), 1.0);
        s.erase_all();
        assert!(s.ds.get_events(&s.window_bucket, None, None, None).unwrap().is_empty());
        assert!(s.ds.get_bucket(&web).is_err(), "le seau de l'extension est supprimé");
        assert!(s.ds.get_bucket(&s.afk_bucket).is_ok(), "nos seaux sont recréés");
    }
}
