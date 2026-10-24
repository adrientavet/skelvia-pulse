//! Serveur local pour l'extension navigateur « ActivityWatch Web Watcher ».
//!
//! C'est le serveur d'ActivityWatch (aw-server-rust, MPL-2.0), utilisé tel
//! quel. Il n'écoute que sur `127.0.0.1` (le PC lui-même, jamais le réseau),
//! sur le port 5600 que l'extension utilise. Pendant la pause, les données
//! de l'extension sont refusées.

use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use aw_datastore::Datastore;
use aw_server::config::AWConfig;
use aw_server::endpoints::{build_rocket, AssetResolver, ServerState};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::http::Status;
use rocket::{Data, Request};

pub const PORT: u16 = 5600;
const PAUSED_PATH: &str = "/skelvia-pulse/en-pause";

/// Refuse les envois de l'extension pendant la pause.
struct PauseGate {
    paused: Arc<AtomicBool>,
}

fn is_web_heartbeat(req: &Request<'_>) -> bool {
    let segs: Vec<&str> = req.uri().path().segments().collect();
    matches!(segs.as_slice(), ["api", "0", "buckets", id, "heartbeat"] if id.starts_with("aw-watcher-web"))
}

#[rocket::async_trait]
impl Fairing for PauseGate {
    fn info(&self) -> Info {
        Info { name: "Pause SKELVIA Pulse", kind: Kind::Request }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        if self.paused.load(Ordering::SeqCst) && is_web_heartbeat(req) {
            req.set_uri(Origin::parse(PAUSED_PATH).expect("chemin valide"));
        }
    }
}

#[rocket::post("/en-pause")]
fn paused_route() -> Status {
    Status::ServiceUnavailable
}

/// Vrai si le port est libre (ActivityWatch, s'il est déjà installé et
/// lancé, l'occupe).
pub fn port_is_free() -> bool {
    TcpListener::bind(("127.0.0.1", PORT)).is_ok()
}

/// Le serveur ActivityWatch, avec le refus des données pendant la pause.
fn build(ds: Datastore, device_id: String, paused: Arc<AtomicBool>) -> rocket::Rocket<rocket::Build> {
    let mut config = AWConfig::default();
    config.address = "127.0.0.1".to_string();
    config.port = PORT;
    let state = ServerState::new(ds, AssetResolver::new(None), device_id);
    build_rocket(state, config)
        .attach(PauseGate { paused })
        .mount("/skelvia-pulse", rocket::routes![paused_route])
}

/// Démarre le serveur en arrière-plan. Renvoie une erreur lisible si le
/// port est déjà pris.
pub fn start(ds: Datastore, device_id: String, paused: Arc<AtomicBool>) -> Result<(), String> {
    if !port_is_free() {
        return Err("Le port 5600 est déjà utilisé, sans doute par ActivityWatch. \
            L'extension navigateur enverra ses données à ce programme et non à SKELVIA Pulse. \
            Le temps par application reste mesuré normalement."
            .to_string());
    }
    let rocket = build(ds, device_id, paused);
    tauri::async_runtime::spawn(async move {
        if let Err(e) = rocket.launch().await {
            log::error!("Serveur local arrêté : {e}");
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Header};
    use rocket::local::blocking::Client;

    #[test]
    fn extension_heartbeats_are_refused_during_pause() {
        let paused = Arc::new(AtomicBool::new(false));
        let ds = Datastore::new_in_memory(false);
        let client = Client::untracked(build(ds.clone(), "test".into(), paused.clone())).unwrap();
        let host = || Header::new("Host", "127.0.0.1:5600");
        let bucket = "/api/0/buckets/aw-watcher-web-chrome_PC";
        let res = client
            .post(bucket)
            .header(host())
            .header(ContentType::JSON)
            .body(r#"{"client":"aw-client-web","type":"web.tab.current","hostname":"PC"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let beat = |ts: &str| {
            client
                .post(format!("{bucket}/heartbeat?pulsetime=30"))
                .header(host())
                .header(ContentType::JSON)
                .body(format!(r#"{{"timestamp":"{ts}","duration":0,"data":{{"url":"https://boamp.fr","title":"BOAMP"}}}}"#))
                .dispatch()
                .status()
        };
        assert_eq!(beat("2026-10-20T08:00:00Z"), Status::Ok);
        paused.store(true, Ordering::SeqCst);
        assert_eq!(beat("2026-10-20T09:00:00Z"), Status::ServiceUnavailable);
        let events = ds.get_events("aw-watcher-web-chrome_PC", None, None, None).unwrap();
        assert_eq!(events.len(), 1, "rien n'est enregistré pendant la pause");
        paused.store(false, Ordering::SeqCst);
        assert_eq!(beat("2026-10-20T10:00:00Z"), Status::Ok);
    }
}
