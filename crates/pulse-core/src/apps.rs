//! Noms lisibles des applications et nettoyage des titres de fenêtres.

/// Navigateur correspondant à un programme, sous le nom que donne
/// l'extension ActivityWatch Web Watcher à ses données
/// (`aw-watcher-web-<navigateur>_<nom du PC>`).
pub fn browser_key(app_exe: &str) -> Option<&'static str> {
    match app_exe.to_lowercase().as_str() {
        "chrome.exe" => Some("chrome"),
        "msedge.exe" => Some("edge"),
        "firefox.exe" => Some("firefox"),
        "brave.exe" => Some("brave"),
        "opera.exe" | "opera_gx.exe" => Some("opera"),
        _ => None,
    }
}

/// Nom lisible d'un programme Windows. Les programmes inconnus gardent leur
/// nom de fichier, sans « .exe ».
pub fn friendly_name(app_exe: &str) -> String {
    let lower = app_exe.to_lowercase();
    let known = match lower.as_str() {
        "excel.exe" => "Excel",
        "winword.exe" => "Word",
        "powerpnt.exe" => "PowerPoint",
        "outlook.exe" | "olk.exe" => "Outlook",
        "onenote.exe" => "OneNote",
        "msaccess.exe" => "Access",
        "ms-teams.exe" | "teams.exe" => "Teams",
        "acrobat.exe" | "acrord32.exe" => "Adobe Acrobat",
        "chrome.exe" => "Google Chrome",
        "msedge.exe" => "Microsoft Edge",
        "firefox.exe" => "Firefox",
        "brave.exe" => "Brave",
        "opera.exe" | "opera_gx.exe" => "Opera",
        "explorer.exe" => "Explorateur de fichiers",
        "thunderbird.exe" => "Thunderbird",
        "soffice.bin" | "soffice.exe" | "scalc.exe" | "swriter.exe" => "LibreOffice",
        "acad.exe" => "AutoCAD",
        "sketchup.exe" => "SketchUp",
        "revit.exe" => "Revit",
        "zoom.exe" => "Zoom",
        "notepad.exe" => "Bloc-notes",
        "whatsapp.exe" => "WhatsApp",
        "applicationframehost.exe" => "Application Windows",
        "skelvia-pulse.exe" => "SKELVIA Pulse",
        _ => "",
    };
    if !known.is_empty() {
        return known.to_string();
    }
    let base = app_exe.rsplit(['\\', '/']).next().unwrap_or(app_exe);
    let base = base.strip_suffix(".exe").or_else(|| base.strip_suffix(".EXE")).unwrap_or(base);
    if base.is_empty() {
        "Application inconnue".to_string()
    } else {
        base.to_string()
    }
}

/// Retire du titre le nom de l'application qui le termine souvent
/// (« Devis.xlsx - Excel » → « Devis.xlsx »).
pub fn clean_title(title: &str, app_name: &str) -> String {
    let t = title.trim();
    let app = app_name.to_lowercase();
    for sep in [" - ", " — ", " – ", " | "] {
        if let Some((head, tail)) = t.rsplit_once(sep) {
            // Edge écrit « Microsoft​ Edge » avec une espace de largeur nulle.
            let tail_l = tail.replace('\u{200b}', "").to_lowercase();
            let tail_t = tail_l.trim();
            let is_app_suffix = !app.is_empty()
                && (tail_l.contains(&app)
                    || (tail_t.len() >= 3 && app.contains(tail_t))
                    || tail_l.starts_with("mozilla firefox")
                    || tail_l.starts_with("microsoft")
                    || tail_l.starts_with("adobe acrobat"));
            if is_app_suffix && !head.trim().is_empty() {
                return clean_title(head, app_name);
            }
        }
    }
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(friendly_name("EXCEL.EXE"), "Excel");
        assert_eq!(friendly_name("BatiPro.exe"), "BatiPro");
        assert_eq!(friendly_name(""), "Application inconnue");
        assert_eq!(browser_key("MSEDGE.EXE"), Some("edge"));
        assert_eq!(browser_key("excel.exe"), None);
    }

    #[test]
    fn titles() {
        assert_eq!(clean_title("Devis.xlsx - Excel", "Excel"), "Devis.xlsx");
        assert_eq!(
            clean_title("BOAMP - Avis - Google Chrome", "Google Chrome"),
            "BOAMP - Avis"
        );
        assert_eq!(
            clean_title("Page — Mozilla Firefox", "Firefox"),
            "Page"
        );
        assert_eq!(clean_title("Excel", "Excel"), "Excel");
        assert_eq!(
            clean_title("Accueil - Profil 1 - Microsoft Edge", "Microsoft Edge"),
            "Accueil - Profil 1"
        );
    }
}
