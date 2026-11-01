//! Règles de regroupement par type de tâche, et exclusions.
//!
//! Tout se fait sur le PC, avec des règles lisibles et modifiables :
//! nom d'application, domaine, ou mot dans le titre. Aucune IA externe.

use crate::settings::{Criterion, Exclusion, Rule};
use crate::text::{contains_word, fold};

/// Catégorie du temps qu'aucune règle ne range.
pub const UNCATEGORIZED: &str = "Autre";

/// Ce que l'on sait d'un moment passé devant l'écran.
#[derive(Debug, Clone, Default)]
pub struct Context<'a> {
    /// Nom du programme, par exemple `EXCEL.EXE`.
    pub app_exe: &'a str,
    /// Nom lisible, par exemple `Excel`.
    pub app_name: &'a str,
    pub domain: Option<&'a str>,
    pub title: &'a str,
}

fn matches(criterion: Criterion, value: &str, ctx: &Context) -> bool {
    let v = fold(value);
    match criterion {
        Criterion::App => {
            contains_word(&fold(ctx.app_exe), &v) || contains_word(&fold(ctx.app_name), &v)
        }
        Criterion::Site => ctx.domain.is_some_and(|d| fold(d).contains(v.trim())),
        Criterion::Titre => contains_word(&fold(ctx.title), &v),
    }
}

/// Catégorie du moment : la première règle qui correspond, sinon « Autre ».
pub fn categorize<'r>(rules: &'r [Rule], ctx: &Context) -> &'r str {
    rules
        .iter()
        .find(|r| matches(r.criterion, &r.value, ctx))
        .map_or(UNCATEGORIZED, |r| r.category.as_str())
}

/// Vrai si ce moment correspond à une exclusion de l'utilisateur.
pub fn is_excluded(exclusions: &[Exclusion], ctx: &Context) -> bool {
    exclusions.iter().any(|e| matches(e.criterion, &e.value, ctx))
}

fn rule(category: &str, criterion: Criterion, value: &str) -> Rule {
    Rule { category: category.to_string(), criterion, value: value.to_string() }
}

/// Règles proposées au premier lancement, valables pour toute entreprise,
/// de l'indépendant à la PME (0.2.4). Ce sont des points de départ :
/// l'utilisateur les modifie dans les réglages. L'ordre compte : les mots
/// du titre passent avant les sites, qui passent avant les applications
/// (un devis ouvert dans Excel compte comme un devis, pas comme « Excel »).
/// Un site se reconnaît à un morceau de son adresse : jamais « x.com »,
/// que l'on trouverait aussi dans « dropbox.com ».
pub fn default_rules() -> Vec<Rule> {
    use Criterion::*;
    const DEVIS: &str = "Devis et factures";
    const COMPTA: &str = "Comptabilité et administratif";
    const VENTES: &str = "Clients et prospection";
    const AO: &str = "Appels d'offres";
    const ACHATS: &str = "Achats et fournisseurs";
    const MAILS: &str = "Mails";
    const MESSAGES: &str = "Messagerie";
    const REUNIONS: &str = "Réunions et agenda";
    const PROJETS: &str = "Projets et tâches";
    const IA: &str = "Outils d'IA";
    const RESEAUX: &str = "Réseaux sociaux";
    const CREATION: &str = "Création et visuels";
    const PLANS: &str = "Plans et conception";
    const CODE: &str = "Développement";
    const DOCS: &str = "Documents et tableurs";
    const WEB: &str = "Autres recherches sur internet";

    vec![
        // Mots dans le titre.
        rule(DEVIS, Titre, "devis"),
        rule(DEVIS, Titre, "facture"),
        rule(DEVIS, Titre, "impayé"),
        rule(DEVIS, Titre, "bon de commande"),
        rule(COMPTA, Titre, "comptabilité"),
        rule(COMPTA, Titre, "note de frais"),
        rule(COMPTA, Titre, "fiche de paie"),
        rule(VENTES, Titre, "proposition commerciale"),
        rule(VENTES, Titre, "prospect"),
        rule(AO, Titre, "appel d'offres"),
        rule(AO, Titre, "DCE"),
        rule(AO, Titre, "mémoire technique"),
        rule(PLANS, Titre, "CCTP"),
        rule(PLANS, Titre, ".dwg"),
        rule(ACHATS, Titre, "fournisseur"),
        rule(ACHATS, Titre, "tarif"),
        rule(ACHATS, Titre, "catalogue"),
        // Sites.
        rule(AO, Site, "boamp.fr"),
        rule(AO, Site, "marches-publics.gouv.fr"),
        rule(AO, Site, "achatpublic.com"),
        rule(MAILS, Site, "mail.google.com"),
        rule(MAILS, Site, "outlook.live.com"),
        rule(MAILS, Site, "outlook.office.com"),
        rule(MAILS, Site, "outlook.office365.com"),
        rule(MAILS, Site, "mail.yahoo.com"),
        rule(MAILS, Site, "mail.proton.me"),
        rule(MESSAGES, Site, "web.whatsapp.com"),
        rule(MESSAGES, Site, "slack.com"),
        rule(MESSAGES, Site, "messenger.com"),
        rule(MESSAGES, Site, "discord.com"),
        rule(REUNIONS, Site, "teams.microsoft.com"),
        rule(REUNIONS, Site, "teams.live.com"),
        rule(REUNIONS, Site, "meet.google.com"),
        rule(REUNIONS, Site, "zoom.us"),
        rule(REUNIONS, Site, "calendar.google.com"),
        rule(REUNIONS, Site, "calendly.com"),
        rule(COMPTA, Site, "impots.gouv.fr"),
        rule(COMPTA, Site, "urssaf.fr"),
        rule(COMPTA, Site, "net-entreprises.fr"),
        rule(COMPTA, Site, "pennylane.com"),
        rule(COMPTA, Site, "indy.fr"),
        rule(COMPTA, Site, "tiime.fr"),
        rule(VENTES, Site, "hubspot.com"),
        rule(VENTES, Site, "pipedrive.com"),
        rule(VENTES, Site, "salesforce.com"),
        rule(VENTES, Site, "axonaut.com"),
        rule(VENTES, Site, "sellsy.com"),
        rule(PROJETS, Site, "notion.so"),
        rule(PROJETS, Site, "trello.com"),
        rule(PROJETS, Site, "asana.com"),
        rule(PROJETS, Site, "monday.com"),
        rule(PROJETS, Site, "clickup.com"),
        rule(PROJETS, Site, "airtable.com"),
        rule(PROJETS, Site, "todoist.com"),
        rule(PROJETS, Site, "atlassian.net"),
        rule(IA, Site, "chatgpt.com"),
        rule(IA, Site, "chat.openai.com"),
        rule(IA, Site, "claude.ai"),
        rule(IA, Site, "gemini.google.com"),
        rule(IA, Site, "perplexity.ai"),
        rule(IA, Site, "chat.mistral.ai"),
        rule(IA, Site, "copilot.microsoft.com"),
        rule(RESEAUX, Site, "linkedin.com"),
        rule(RESEAUX, Site, "facebook.com"),
        rule(RESEAUX, Site, "instagram.com"),
        rule(RESEAUX, Site, "tiktok.com"),
        rule(RESEAUX, Site, "twitter.com"),
        rule(CREATION, Site, "canva.com"),
        rule(CREATION, Site, "figma.com"),
        rule(CODE, Site, "github.com"),
        rule(CODE, Site, "gitlab.com"),
        rule(DOCS, Site, "docs.google.com"),
        rule(DOCS, Site, "drive.google.com"),
        rule(DOCS, Site, "sharepoint.com"),
        rule(DOCS, Site, "onedrive.live.com"),
        rule(ACHATS, Site, "pointp.fr"),
        rule(ACHATS, Site, "cedeo.fr"),
        rule(ACHATS, Site, "rexel.fr"),
        rule(ACHATS, Site, "gedimat.fr"),
        // Applications.
        rule(MAILS, App, "outlook"),
        rule(MAILS, App, "olk"),
        rule(MAILS, App, "thunderbird"),
        rule(MESSAGES, App, "whatsapp"),
        rule(MESSAGES, App, "slack"),
        rule(MESSAGES, App, "telegram"),
        rule(MESSAGES, App, "signal"),
        rule(MESSAGES, App, "discord"),
        rule(REUNIONS, App, "teams"),
        rule(REUNIONS, App, "zoom"),
        rule(REUNIONS, App, "webex"),
        rule(COMPTA, App, "sage"),
        rule(COMPTA, App, "ebp"),
        rule(PROJETS, App, "notion"),
        rule(PROJETS, App, "todoist"),
        rule(IA, App, "chatgpt"),
        rule(IA, App, "claude"),
        rule(CREATION, App, "canva"),
        rule(CREATION, App, "figma"),
        rule(CREATION, App, "photoshop"),
        rule(CREATION, App, "illustrator"),
        rule(CREATION, App, "indesign"),
        rule(CREATION, App, "lightroom"),
        rule(CREATION, App, "premiere"),
        rule(CREATION, App, "afterfx"),
        rule(CREATION, App, "capcut"),
        rule(CREATION, App, "resolve"),
        rule(PLANS, App, "acad"),
        rule(PLANS, App, "autocad"),
        rule(PLANS, App, "sketchup"),
        rule(PLANS, App, "revit"),
        rule(PLANS, App, "archicad"),
        rule(CODE, App, "code"),
        rule(CODE, App, "cursor"),
        rule(CODE, App, "devenv"),
        rule(DOCS, App, "excel"),
        rule(DOCS, App, "winword"),
        rule(DOCS, App, "powerpnt"),
        rule(DOCS, App, "onenote"),
        rule(DOCS, App, "acrobat"),
        rule(DOCS, App, "acrord32"),
        rule(DOCS, App, "soffice"),
        rule(WEB, App, "chrome"),
        rule(WEB, App, "msedge"),
        rule(WEB, App, "firefox"),
        rule(WEB, App, "brave"),
        rule(WEB, App, "opera"),
    ]
}

/// Les règles proposées jusqu'à la version 0.2.3, pensées pour le bâtiment.
/// Elles ne servent qu'à reconnaître un PC qui ne les a jamais modifiées,
/// pour lui donner les nouvelles (voir `Settings::upgrade_default_rules`).
pub fn default_rules_before_0_2_4() -> Vec<Rule> {
    use Criterion::*;
    const DEVIS: &str = "Devis et chiffrage";
    const FACTURES: &str = "Factures et relances";
    const AO: &str = "Appels d'offres";
    const PLANS: &str = "Plans et CCTP";
    const FOURNISSEURS: &str = "Recherche de fournisseurs";
    const MAILS: &str = "Mails";
    const ADMIN: &str = "Administratif";
    const REUNIONS: &str = "Réunions et appels";
    const DOCS: &str = "Autres documents et tableurs";
    const WEB: &str = "Autres recherches sur internet";

    vec![
        // Mots dans le titre.
        rule(DEVIS, Titre, "devis"),
        rule(DEVIS, Titre, "chiffrage"),
        rule(DEVIS, Titre, "métré"),
        rule(FACTURES, Titre, "facture"),
        rule(FACTURES, Titre, "relance"),
        rule(FACTURES, Titre, "impayé"),
        rule(FACTURES, Titre, "situation de travaux"),
        rule(AO, Titre, "appel d'offres"),
        rule(AO, Titre, "DCE"),
        rule(AO, Titre, "DPGF"),
        rule(AO, Titre, "CCAP"),
        rule(AO, Titre, "mémoire technique"),
        rule(PLANS, Titre, "CCTP"),
        rule(PLANS, Titre, ".dwg"),
        rule(PLANS, Titre, "plans"),
        rule(FOURNISSEURS, Titre, "fournisseur"),
        rule(FOURNISSEURS, Titre, "tarif"),
        rule(FOURNISSEURS, Titre, "catalogue"),
        // Sites.
        rule(AO, Site, "boamp.fr"),
        rule(AO, Site, "marches-publics.gouv.fr"),
        rule(AO, Site, "achatpublic.com"),
        rule(MAILS, Site, "mail.google.com"),
        rule(MAILS, Site, "outlook.live.com"),
        rule(MAILS, Site, "outlook.office.com"),
        rule(MAILS, Site, "outlook.office365.com"),
        rule(FOURNISSEURS, Site, "pointp.fr"),
        rule(FOURNISSEURS, Site, "cedeo.fr"),
        rule(FOURNISSEURS, Site, "leroymerlin.fr"),
        rule(FOURNISSEURS, Site, "manomano.fr"),
        rule(FOURNISSEURS, Site, "rexel.fr"),
        rule(FOURNISSEURS, Site, "gedimat.fr"),
        rule(ADMIN, Site, "impots.gouv.fr"),
        rule(ADMIN, Site, "urssaf.fr"),
        rule(ADMIN, Site, "net-entreprises.fr"),
        rule(REUNIONS, Site, "teams.microsoft.com"),
        rule(REUNIONS, Site, "meet.google.com"),
        // Applications.
        rule(MAILS, App, "outlook"),
        rule(MAILS, App, "olk"),
        rule(MAILS, App, "thunderbird"),
        rule(PLANS, App, "acad"),
        rule(PLANS, App, "autocad"),
        rule(PLANS, App, "sketchup"),
        rule(PLANS, App, "revit"),
        rule(REUNIONS, App, "teams"),
        rule(REUNIONS, App, "zoom"),
        rule(DOCS, App, "excel"),
        rule(DOCS, App, "winword"),
        rule(DOCS, App, "acrobat"),
        rule(DOCS, App, "acrord32"),
        rule(DOCS, App, "soffice"),
        rule(WEB, App, "chrome"),
        rule(WEB, App, "msedge"),
        rule(WEB, App, "firefox"),
        rule(WEB, App, "brave"),
        rule(WEB, App, "opera"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(exe: &'a str, name: &'a str, domain: Option<&'a str>, title: &'a str) -> Context<'a> {
        Context { app_exe: exe, app_name: name, domain, title }
    }

    #[test]
    fn title_beats_app() {
        let r = default_rules();
        let c = ctx("EXCEL.EXE", "Excel", None, "Devis Dupont - toiture.xlsx - Excel");
        assert_eq!(categorize(&r, &c), "Devis et factures");
        let c = ctx("EXCEL.EXE", "Excel", None, "Suivi chantier.xlsx - Excel");
        assert_eq!(categorize(&r, &c), "Documents et tableurs");
    }

    #[test]
    fn site_rules() {
        let r = default_rules();
        let c = ctx("chrome.exe", "Google Chrome", Some("pointp.fr"), "Plaque de plâtre BA13");
        assert_eq!(categorize(&r, &c), "Achats et fournisseurs");
        let c = ctx("msedge.exe", "Microsoft Edge", Some("example.org"), "Météo");
        assert_eq!(categorize(&r, &c), "Autres recherches sur internet");
    }

    #[test]
    fn cctp_pdf() {
        let r = default_rules();
        let c = ctx("Acrobat.exe", "Adobe Acrobat", None, "CCTP lot 03 menuiseries.pdf - Adobe Acrobat");
        assert_eq!(categorize(&r, &c), "Plans et conception");
    }

    #[test]
    fn any_business() {
        let r = default_rules();
        let c = ctx("chrome.exe", "Google Chrome", Some("chatgpt.com"), "Résumé de réunion");
        assert_eq!(categorize(&r, &c), "Outils d'IA");
        let c = ctx("chrome.exe", "Google Chrome", Some("www.linkedin.com"), "Fil d'actualité");
        assert_eq!(categorize(&r, &c), "Réseaux sociaux");
        let c = ctx("Canva.exe", "Canva", None, "Affiche salon");
        assert_eq!(categorize(&r, &c), "Création et visuels");
        let c = ctx("chrome.exe", "Google Chrome", Some("www.dropbox.com"), "Fichiers");
        assert_eq!(categorize(&r, &c), "Autres recherches sur internet");
        let c = ctx("WhatsApp.exe", "WhatsApp", None, "WhatsApp");
        assert_eq!(categorize(&r, &c), "Messagerie");
        let c = ctx("Notion.exe", "Notion", None, "Planning de la semaine");
        assert_eq!(categorize(&r, &c), "Projets et tâches");
    }

    #[test]
    fn previous_defaults_are_recognised() {
        assert_ne!(default_rules(), default_rules_before_0_2_4());
        assert!(default_rules_before_0_2_4().iter().any(|r| r.category == "Plans et CCTP"));
    }

    #[test]
    fn unknown_is_other() {
        let r = default_rules();
        let c = ctx("notepad.exe", "Bloc-notes", None, "liste.txt");
        assert_eq!(categorize(&r, &c), UNCATEGORIZED);
    }

    #[test]
    fn exclusions() {
        let ex = vec![
            Exclusion { criterion: Criterion::Site, value: "credit-agricole.fr".into() },
            Exclusion { criterion: Criterion::App, value: "keepass".into() },
            Exclusion { criterion: Criterion::Titre, value: "Médical".into() },
        ];
        assert!(is_excluded(&ex, &ctx("chrome.exe", "Chrome", Some("www.credit-agricole.fr"), "")));
        assert!(is_excluded(&ex, &ctx("KeePass.exe", "KeePass", None, "Base")));
        assert!(is_excluded(&ex, &ctx("WINWORD.EXE", "Word", None, "Dossier medical.docx")));
        assert!(!is_excluded(&ex, &ctx("EXCEL.EXE", "Excel", None, "Devis.xlsx")));
    }
}
