//! Le rapport : un seul fichier HTML, lisible hors ligne, aux couleurs
//! SKELVIA. Polices incluses dans le fichier, aucune ressource externe.
//! Le seul lien est celui, facultatif, vers la réservation d'un audit.

use std::fmt::Write;

use chrono::{DateTime, Local, Utc};

use crate::format::{date_long, date_short, duration};
use crate::mascot;
use crate::stats::Summary;

pub const AUDIT_URL: &str = "https://skelvia.fr/#reserver";

/// Polices WOFF2 à inclure dans le fichier (sous licence SIL OFL 1.1).
#[derive(Debug, Clone, Copy, Default)]
pub struct Fonts<'a> {
    pub display_woff2: &'a [u8],
    pub text_woff2: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct ReportOptions {
    /// Ajouter le nom des documents et des pages (peut contenir des noms de
    /// clients : désactivé par défaut).
    pub include_items: bool,
    /// Ajouter la liste des sites web consultés.
    pub include_sites: bool,
    /// Ajouter la section finale, discrète, vers l'audit SKELVIA.
    pub include_audit: bool,
    pub generated_at: DateTime<Utc>,
}

impl Default for ReportOptions {
    fn default() -> Self {
        ReportOptions { include_items: false, include_sites: true, include_audit: true, generated_at: Utc::now() }
    }
}

/// Échappe le texte pour l'insérer dans du HTML.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Encodage base64 standard (pour inclure les polices dans le fichier).
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

fn pct(part: f64, total: f64) -> f64 {
    if total <= 0.0 {
        0.0
    } else {
        (part / total * 100.0).clamp(0.0, 100.0)
    }
}

fn pct_label(p: f64) -> String {
    if p < 1.0 {
        "moins de 1 %".to_string()
    } else {
        format!("{} %", p.round() as i64)
    }
}

const CSS: &str = r#"
:root{--bg:#0b0b0c;--card:#151517;--card2:#1f1f22;--text:#fff;--muted:rgba(255,255,255,.74);--faint:rgba(255,255,255,.58);--border:rgba(255,255,255,.1);--accent:#2a63f5;--accent-text:#5b8cff;--primary:#2f6bff;--radius:1.25rem;--ease:cubic-bezier(.2,.8,.2,1)}
*{box-sizing:border-box}
html{background:var(--bg)}
body{margin:0;background:var(--bg);color:var(--text);font-family:"DM Sans",system-ui,-apple-system,"Segoe UI",sans-serif;line-height:1.55;-webkit-font-smoothing:antialiased}
main{max-width:860px;margin:0 auto;padding:48px 20px 64px}
h1,h2,h3{font-family:"Bricolage Grotesque","DM Sans",system-ui,sans-serif;font-weight:800;letter-spacing:-.035em;line-height:1.1;margin:0}
h1{font-size:clamp(30px,5vw,44px)}
h2{font-size:24px;margin-bottom:16px}
p{margin:0}
.eyebrow{display:flex;align-items:center;gap:14px;margin-bottom:28px}
.eyebrow b{font-family:"Bricolage Grotesque",sans-serif;font-weight:800;letter-spacing:.02em;font-size:15px}
.eyebrow span{color:var(--faint);font-size:14px}
.sub{color:var(--muted);margin-top:12px;font-size:16px}
section{margin-top:40px}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:32px}
.card{background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:20px}
.card .k{color:var(--faint);font-size:14px}
.card .v{font-family:"Bricolage Grotesque",sans-serif;font-weight:800;font-size:30px;letter-spacing:-.03em;margin-top:6px}
.row{background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:16px 20px;margin-bottom:10px}
.row-head{display:flex;justify-content:space-between;gap:16px;align-items:baseline}
.row-head .l{font-weight:700}
.row-head .t{font-family:"Bricolage Grotesque",sans-serif;font-weight:700;white-space:nowrap}
.bar{height:8px;border-radius:99px;background:rgba(255,255,255,.06);margin-top:10px;overflow:hidden}
.bar i{display:block;height:100%;border-radius:0 4px 4px 0;background:var(--accent)}
.meta{color:var(--faint);font-size:14px;margin-top:8px}
.items{margin:10px 0 0;padding:0;list-style:none;font-size:14px;color:var(--muted)}
.items li{display:flex;justify-content:space-between;gap:12px;padding:3px 0;border-top:1px solid var(--border)}
.items li span:first-child{overflow-wrap:anywhere}
table{width:100%;border-collapse:collapse;background:var(--card);border:1px solid var(--border);border-radius:var(--radius);overflow:hidden}
td{padding:11px 20px;border-top:1px solid var(--border)}
tr:first-child td{border-top:0}
td:last-child{text-align:right;white-space:nowrap;font-family:"Bricolage Grotesque",sans-serif;font-weight:700}
td.p{color:var(--faint);font-family:inherit;font-weight:400;width:1%;white-space:nowrap}
.days{display:flex;align-items:flex-end;gap:6px;height:140px;background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:16px 16px 0}
.day{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:flex-end;height:100%;min-width:0}
.day i{display:block;width:100%;max-width:36px;background:var(--accent);border-radius:4px 4px 0 0;min-height:2px}
.day span{font-size:11px;color:var(--faint);padding:6px 0 10px;white-space:nowrap}
.note{background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:20px;color:var(--muted);font-size:15px}
.note ul{margin:8px 0 0;padding-left:20px}
.audit{margin-top:48px;border-top:1px solid var(--border);padding-top:24px;color:var(--faint);font-size:14px}
.audit a{color:var(--accent-text);text-decoration:underline;text-underline-offset:3px}
footer{margin-top:32px;color:var(--faint);font-size:13px}
.empty{color:var(--muted)}
@media print{
:root{--bg:#fff;--card:#fff;--text:#0b0b0c;--muted:#333;--faint:#555;--border:#ddd}
main{padding:0}
.row,.card,table,.days,.note{break-inside:avoid}
.bar{background:#eee}
.audit a{color:#0b0b0c}
}
"#;

fn font_faces(fonts: &Fonts) -> String {
    let mut s = String::new();
    if !fonts.display_woff2.is_empty() {
        let _ = write!(
            s,
            "@font-face{{font-family:\"Bricolage Grotesque\";font-weight:400 800;font-display:swap;src:url(data:font/woff2;base64,{}) format(\"woff2\")}}",
            base64(fonts.display_woff2)
        );
    }
    if !fonts.text_woff2.is_empty() {
        let _ = write!(
            s,
            "@font-face{{font-family:\"DM Sans\";font-weight:400 700;font-display:swap;src:url(data:font/woff2;base64,{}) format(\"woff2\")}}",
            base64(fonts.text_woff2)
        );
    }
    s
}

/// Construit le rapport HTML complet.
pub fn render(summary: &Summary, opts: &ReportOptions, fonts: &Fonts) -> String {
    let from_local = summary.from.with_timezone(&Local).date_naive();
    // `to` est exclu : le dernier jour couvert est la veille de `to` à minuit.
    let to_local = (summary.to - chrono::Duration::seconds(1)).with_timezone(&Local).date_naive();
    let period = if from_local == to_local {
        format!("Le {}", date_long(from_local))
    } else {
        format!("Du {} au {}", date_long(from_local), date_long(to_local))
    };
    let period_days = ((to_local - from_local).num_days() + 1).max(1);
    let total = summary.active_secs;
    let active_days = summary.days.iter().filter(|d| d.secs >= 60.0).count();

    let mut h = String::new();
    h.push_str("<!doctype html><html lang=\"fr\"><head><meta charset=\"utf-8\">");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    h.push_str("<meta name=\"color-scheme\" content=\"dark light\">");
    let _ = write!(h, "<title>Rapport SKELVIA Pulse · {}</title>", esc(&period));
    let _ = write!(h, "<style>{}{}</style></head><body><main>", font_faces(fonts), CSS);

    // En-tête.
    let _ = write!(
        h,
        "<div class=\"eyebrow\">{}<div><b>SKELVIA Pulse</b><br><span>Rapport créé sur ce PC le {}</span></div></div>",
        mascot::svg(mascot::BLUE, false, 56, "Mascotte SKELVIA"),
        esc(&date_long(opts.generated_at.with_timezone(&Local).date_naive()))
    );
    h.push_str("<h1>Où part votre temps sur l'ordinateur</h1>");
    let _ = write!(h, "<p class=\"sub\">{}. Seul le temps où la souris ou le clavier ont servi est compté.</p>", esc(&period));

    if total < 60.0 {
        h.push_str("<section><p class=\"empty\">Pas encore assez de temps enregistré sur cette période pour faire un rapport utile. Laissez SKELVIA Pulse tourner quelques jours, puis recommencez.</p></section>");
    } else {
        // Chiffres clés.
        h.push_str("<div class=\"cards\">");
        let _ = write!(h, "<div class=\"card\"><div class=\"k\">Temps actif sur l'ordinateur</div><div class=\"v\">{}</div></div>", duration(total));
        if period_days > 1 {
            let _ = write!(h, "<div class=\"card\"><div class=\"k\">Jours avec activité</div><div class=\"v\">{active_days}</div></div>");
            if active_days > 0 {
                let _ = write!(h, "<div class=\"card\"><div class=\"k\">Moyenne par jour avec activité</div><div class=\"v\">{}</div></div>", duration(total / active_days as f64));
            }
        }
        if period_days > 7 {
            let weeks = period_days as f64 / 7.0;
            let _ = write!(h, "<div class=\"card\"><div class=\"k\">Moyenne par semaine</div><div class=\"v\">{}</div></div>", duration(total / weeks));
        }
        h.push_str("</div>");

        // Par type de tâche.
        h.push_str("<section><h2>Par type de tâche</h2>");
        let weeks = (period_days > 7).then(|| period_days as f64 / 7.0);
        for c in &summary.by_category {
            let p = pct(c.secs, total);
            let per_week = weeks.map(|w| format!(", soit {} par semaine en moyenne", duration(c.secs / w))).unwrap_or_default();
            let _ = write!(
                h,
                "<div class=\"row\"><div class=\"row-head\"><span class=\"l\">{}</span><span class=\"t\">{}</span></div><div class=\"bar\"><i style=\"width:{:.1}%\"></i></div><p class=\"meta\">{}{}",
                esc(&c.label),
                duration(c.secs),
                p,
                pct_label(p),
                esc(&per_week)
            );
            if !c.apps.is_empty() {
                let apps: Vec<String> = c.apps.iter().map(|a| esc(&a.label)).collect();
                let _ = write!(h, " · {}", apps.join(", "));
            }
            h.push_str("</p>");
            if opts.include_items && !c.items.is_empty() {
                h.push_str("<ul class=\"items\">");
                for it in c.items.iter().take(3) {
                    let _ = write!(h, "<li><span>{}</span><span>{}</span></li>", esc(&it.label), duration(it.secs));
                }
                h.push_str("</ul>");
            }
            h.push_str("</div>");
        }
        h.push_str("</section>");

        // Par application.
        h.push_str("<section><h2>Par application</h2><table>");
        for a in summary.by_app.iter().take(10) {
            let p = pct(a.secs, total);
            let _ = write!(h, "<tr><td>{}</td><td class=\"p\">{}</td><td>{}</td></tr>", esc(&a.label), pct_label(p), duration(a.secs));
        }
        h.push_str("</table></section>");

        if opts.include_sites && !summary.by_site.is_empty() {
            h.push_str("<section><h2>Sites web les plus consultés</h2><table>");
            for s in summary.by_site.iter().take(10) {
                let _ = write!(h, "<tr><td>{}</td><td class=\"p\">{}</td><td>{}</td></tr>", esc(&s.label), pct_label(pct(s.secs, total)), duration(s.secs));
            }
            h.push_str("</table></section>");
        }

        if opts.include_items && !summary.top_items.is_empty() {
            h.push_str("<section><h2>Documents et pages les plus longs</h2><table>");
            for it in summary.top_items.iter().take(10) {
                let _ = write!(h, "<tr><td>{}</td><td>{}</td></tr>", esc(&it.label), duration(it.secs));
            }
            h.push_str("</table></section>");
        }

        if summary.days.len() > 1 {
            let max = summary.days.iter().map(|d| d.secs).fold(0.0, f64::max);
            h.push_str("<section><h2>Jour par jour</h2><div class=\"days\">");
            for d in &summary.days {
                let _ = write!(
                    h,
                    "<div class=\"day\" title=\"{}\"><i style=\"height:{:.1}%\"></i><span>{}</span></div>",
                    esc(&format!("{} : {}", date_long(d.date), duration(d.secs))),
                    pct(d.secs, max),
                    esc(&date_short(d.date))
                );
            }
            h.push_str("</div></section>");
        }
    }

    // Méthode.
    h.push_str("<section><h2>Comment ces chiffres sont calculés</h2><div class=\"note\"><ul>");
    h.push_str("<li>Seule la fenêtre au premier plan compte. Les fenêtres ouvertes en arrière-plan ne sont pas comptées.</li>");
    h.push_str("<li>Sans souris ni clavier pendant le délai choisi dans les réglages, le temps n'est plus compté, même si des fenêtres restent ouvertes. La lecture d'un long document sans toucher la souris peut donc être un peu sous-estimée.</li>");
    h.push_str("<li>Les types de tâche viennent de règles simples (nom d'application, site, mot dans le titre) que vous pouvez modifier. Le temps qu'aucune règle ne range apparaît dans « Autre ».</li>");
    if !summary.has_web_data {
        h.push_str("<li>L'extension navigateur n'était pas active sur cette période : les sites web ne sont connus que par le titre de la fenêtre du navigateur.</li>");
    }
    h.push_str("<li>Ce rapport a été créé sur votre PC. Aucune donnée n'a été envoyée par SKELVIA Pulse.</li>");
    h.push_str("</ul></div></section>");

    if opts.include_audit {
        let _ = write!(
            h,
            "<aside class=\"audit\">Pour analyser ces chiffres avec Adrien, <a href=\"{AUDIT_URL}\" target=\"_blank\" rel=\"noopener\">réserver un audit de 45 minutes gratuit</a>. Sans engagement, et seulement si vous le souhaitez.</aside>"
        );
    }
    h.push_str("<footer>SKELVIA Pulse, logiciel libre et gratuit. Le nom SKELVIA et la mascotte sont réservés.</footer>");
    h.push_str("</main></body></html>");
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use crate::stats::{summarize, Inputs, RawEvent};
    use chrono::{Duration, TimeZone};
    use serde_json::json;

    #[test]
    fn base64_matches_reference() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn escapes_titles() {
        let from = Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap();
        let ev = |data: serde_json::Value| RawEvent { start: from, duration: 3600.0, data: data.as_object().unwrap().clone() };
        let inputs = Inputs {
            window: vec![ev(json!({"app": "EXCEL.EXE", "title": "<script>alert(1)</script> devis - Excel"}))],
            afk: vec![ev(json!({"status": "not-afk"}))],
            web: Default::default(),
        };
        let s = summarize(&inputs, &Settings::default(), from, from + Duration::hours(2));
        let opts = ReportOptions { include_items: true, ..Default::default() };
        let html = render(&s, &opts, &Fonts::default());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(AUDIT_URL));
        // Aucune ressource externe : la seule adresse https est le lien d'audit.
        assert_eq!(html.matches("https://").count(), html.matches(AUDIT_URL).count());
        assert!(!html.contains("src=\"http"));
    }

    #[test]
    fn audit_section_is_optional_and_items_hidden_by_default() {
        let from = Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap();
        let ev = |data: serde_json::Value| RawEvent { start: from, duration: 3600.0, data: data.as_object().unwrap().clone() };
        let inputs = Inputs {
            window: vec![ev(json!({"app": "WINWORD.EXE", "title": "Client Durand.docx - Word"}))],
            afk: vec![ev(json!({"status": "not-afk"}))],
            web: Default::default(),
        };
        let s = summarize(&inputs, &Settings::default(), from, from + Duration::hours(2));
        let opts = ReportOptions { include_audit: false, ..Default::default() };
        let html = render(&s, &opts, &Fonts::default());
        assert!(!html.contains(AUDIT_URL));
        assert!(!html.contains("Client Durand"));
    }
}
