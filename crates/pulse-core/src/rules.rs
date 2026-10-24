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

/// Règles proposées au premier lancement, pensées pour une entreprise du
/// bâtiment. Ce sont des points de départ : l'utilisateur les modifie dans
/// les réglages. L'ordre compte : les mots du titre passent avant les
/// sites, qui passent avant les applications (un devis ouvert dans Excel
/// compte comme du chiffrage, pas comme « Excel »).
pub fn default_rules() -> Vec<Rule> {
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
        assert_eq!(categorize(&r, &c), "Devis et chiffrage");
        let c = ctx("EXCEL.EXE", "Excel", None, "Suivi chantier.xlsx - Excel");
        assert_eq!(categorize(&r, &c), "Autres documents et tableurs");
    }

    #[test]
    fn site_rules() {
        let r = default_rules();
        let c = ctx("chrome.exe", "Google Chrome", Some("pointp.fr"), "Plaque de plâtre BA13");
        assert_eq!(categorize(&r, &c), "Recherche de fournisseurs");
        let c = ctx("msedge.exe", "Microsoft Edge", Some("example.org"), "Météo");
        assert_eq!(categorize(&r, &c), "Autres recherches sur internet");
    }

    #[test]
    fn cctp_pdf() {
        let r = default_rules();
        let c = ctx("Acrobat.exe", "Adobe Acrobat", None, "CCTP lot 03 menuiseries.pdf - Adobe Acrobat");
        assert_eq!(categorize(&r, &c), "Plans et CCTP");
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
