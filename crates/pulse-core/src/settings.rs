//! Réglages de l'utilisateur, enregistrés dans un fichier JSON sur son PC.
//!
//! Aucun réglage n'est obligatoire : les valeurs par défaut suffisent.

use serde::{Deserialize, Serialize};

use crate::rules;

/// Délai sans souris ni clavier au bout duquel le temps n'est plus compté
/// (même valeur par défaut qu'ActivityWatch : 180 secondes).
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 180;

/// Ce sur quoi porte une règle ou une exclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Criterion {
    /// Nom de l'application (par exemple « excel » ou « outlook »).
    App,
    /// Domaine du site web (par exemple « boamp.fr »).
    Site,
    /// Mot présent dans le titre de la fenêtre ou de la page.
    Titre,
}

/// Une règle de regroupement : « si le titre contient devis, c'est du
/// chiffrage ». Les règles sont lues dans l'ordre, la première qui
/// correspond l'emporte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub category: String,
    pub criterion: Criterion,
    pub value: String,
}

/// Une exclusion : ce qui correspond n'est jamais enregistré, et ce qui
/// avait déjà été enregistré est effacé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exclusion {
    pub criterion: Criterion,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Délai d'inactivité, en secondes.
    pub idle_timeout_secs: u64,
    /// Enregistrer le titre des fenêtres. Sans titre, le regroupement ne
    /// peut se faire que sur le nom des applications et des sites.
    pub record_titles: bool,
    pub exclusions: Vec<Exclusion>,
    pub rules: Vec<Rule>,
    /// L'écran d'information du premier lancement a été lu et accepté.
    pub first_run_done: bool,
    /// Enregistrement en pause (gardé au redémarrage du PC).
    pub paused: bool,
    /// Position de la pastille choisie par l'utilisateur (son centre, en
    /// pixels de l'écran ; elle est collée au bord le plus proche). `None` :
    /// en haut, au centre.
    pub island_position: Option<[i32; 2]>,
    /// Vérifier de temps en temps s'il existe une nouvelle version. C'est
    /// la seule connexion à internet de l'application : elle télécharge un
    /// petit fichier public sur GitHub et n'envoie aucune mesure.
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
            record_titles: true,
            exclusions: Vec::new(),
            rules: rules::default_rules(),
            first_run_done: false,
            paused: false,
            island_position: None,
            check_updates: true,
        }
    }
}

impl Settings {
    /// Remet les valeurs dans des bornes raisonnables (un fichier modifié à
    /// la main ne doit pas casser l'application).
    pub fn sanitized(mut self) -> Self {
        self.idle_timeout_secs = self.idle_timeout_secs.clamp(30, 3600);
        self.exclusions.retain(|e| !e.value.trim().is_empty());
        self.rules
            .retain(|r| !r.value.trim().is_empty() && !r.category.trim().is_empty());
        for e in &mut self.exclusions {
            e.value = e.value.trim().to_string();
        }
        for r in &mut self.rules {
            r.value = r.value.trim().to_string();
            r.category = r.category.trim().to_string();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_defaults() {
        let s: Settings = serde_json::from_str(r#"{"paused": true}"#).unwrap();
        assert!(s.paused);
        assert_eq!(s.idle_timeout_secs, DEFAULT_IDLE_TIMEOUT_SECS);
        assert!(!s.rules.is_empty());
        // Réglages écrits par une version sans mises à jour automatiques.
        assert!(s.check_updates);
    }

    #[test]
    fn sanitize_bounds() {
        let mut s = Settings::default();
        s.idle_timeout_secs = 1;
        s.exclusions.push(Exclusion { criterion: Criterion::Site, value: "  ".into() });
        let s = s.sanitized();
        assert_eq!(s.idle_timeout_secs, 30);
        assert!(s.exclusions.is_empty());
    }
}
