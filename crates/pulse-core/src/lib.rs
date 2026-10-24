//! Cœur de SKELVIA Pulse : tout ce qui ne dépend ni de Windows ni de Tauri.
//!
//! - [`settings`] : les réglages de l'utilisateur (délai d'inactivité,
//!   exclusions, règles de regroupement).
//! - [`rules`] : les règles simples qui rangent le temps par type de tâche.
//! - [`stats`] : le calcul du temps actif à partir des événements enregistrés.
//! - [`report`] : le rapport HTML d'une page, lisible hors ligne.
//!
//! Rien ici n'envoie quoi que ce soit sur le réseau.

pub mod apps;
pub mod format;
pub mod mascot;
pub mod report;
pub mod rules;
pub mod settings;
pub mod stats;
pub mod text;
