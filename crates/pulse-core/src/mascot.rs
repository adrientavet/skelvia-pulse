//! La mascotte SKELVIA en SVG, pour le rapport HTML.
//!
//! Même dessin que `src/mascot.ts` (interface) : un rond noir cerclé d'un fin
//! trait de couleur, éclairé de l'intérieur par des points en demi-teinte.
//! « Au travail » : points vifs, yeux ronds blancs, halo. « En veille » :
//! points pâles, yeux fermés, pas de halo. Le nom et le dessin de la
//! mascotte sont réservés (voir TRADEMARKS.md).

use std::fmt::Write;

const CX: f64 = 100.0;
const CY: f64 = 100.0;

/// Couleur de la mascotte : le bleu SKELVIA.
pub const BLUE: &str = "#2A63F5";

fn inner_points() -> Vec<(f64, f64, f64, f64)> {
    let mut pts = Vec::new();
    let mut x = 24.0;
    while x <= 176.0 {
        let mut y = 24.0;
        while y <= 176.0 {
            let d = (x - CX).hypot(y - CY);
            if (31.0..=74.0).contains(&d) {
                let t = (d - 31.0) / (74.0 - 31.0);
                pts.push((x, y, 0.45 + 2.6 * t, 0.17 + 0.8 * t));
            }
            y += 8.0;
        }
        x += 8.0;
    }
    pts
}

fn halo_points() -> Vec<(f64, f64, f64, f64)> {
    let mut pts = Vec::new();
    let mut x = -29.0;
    while x <= 229.0 {
        let mut y = -29.0;
        while y <= 229.0 {
            let d = (x - CX).hypot(y - CY);
            if (82.0..=122.0).contains(&d) {
                let u = 1.0 - (d - 82.0) / 40.0;
                pts.push((x, y, 0.5 + 2.8 * u, 0.14 + 0.4 * u));
            }
            y += 9.0;
        }
        x += 9.0;
    }
    pts
}

/// SVG complet de la mascotte. `asleep` : état « en veille ».
pub fn svg(color: &str, asleep: bool, size_px: u32, title: &str) -> String {
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg viewBox="-30 -30 260 260" width="{size_px}" height="{size_px}" role="img" aria-label="{title}" xmlns="http://www.w3.org/2000/svg">"#
    );
    if !asleep {
        s.push_str("<g>");
        for (x, y, r, o) in halo_points() {
            let _ = write!(s, r#"<circle cx="{x}" cy="{y}" r="{r:.2}" fill="{color}" fill-opacity="{o:.2}"/>"#);
        }
        s.push_str("</g>");
    }
    let stroke_op = if asleep { 0.17 } else { 0.55 };
    let _ = write!(
        s,
        r##"<circle cx="100" cy="100" r="80" fill="#151517" stroke="{color}" stroke-width="2" stroke-opacity="{stroke_op}"/>"##
    );
    let _ = write!(s, r#"<g opacity="{}">"#, if asleep { 0.3 } else { 1.0 });
    for (x, y, r, o) in inner_points() {
        let _ = write!(s, r#"<circle cx="{x}" cy="{y}" r="{r:.2}" fill="{color}" fill-opacity="{o:.2}"/>"#);
    }
    s.push_str("</g>");
    if asleep {
        for cx in [78, 122] {
            let _ = write!(
                s,
                r#"<ellipse cx="{cx}" cy="101.5" rx="11.5" ry="3.5" fill="{color}"/>"#
            );
        }
    } else {
        for cx in [78, 122] {
            let _ = write!(s, r##"<circle cx="{cx}" cy="101.5" r="10" fill="#fff"/>"##);
        }
    }
    s.push_str("</svg>");
    s
}

/// Version pour les petites tailles (moins de 48 px, icône près de
/// l'horloge) : lueur pleine, trait épais, gros yeux. Sinon, en petit, les
/// points en demi-teinte ressemblent à une tache.
pub fn simple_svg(color: &str, asleep: bool, size_px: u32) -> String {
    let glow = if asleep { 0.25 } else { 0.75 };
    let eyes = if asleep {
        format!(
            r#"<ellipse cx="72" cy="100" rx="18" ry="6" fill="{color}"/><ellipse cx="128" cy="100" rx="18" ry="6" fill="{color}"/>"#
        )
    } else {
        r##"<circle cx="72" cy="100" r="16" fill="#fff"/><circle cx="128" cy="100" r="16" fill="#fff"/>"##.to_string()
    };
    format!(
        r##"<svg viewBox="0 0 200 200" width="{size_px}" height="{size_px}" xmlns="http://www.w3.org/2000/svg"><defs><radialGradient id="g" cx="50%" cy="50%" r="50%"><stop offset="45%" stop-color="#151517"/><stop offset="100%" stop-color="{color}" stop-opacity="{glow}"/></radialGradient></defs><circle cx="100" cy="100" r="92" fill="url(#g)" stroke="{color}" stroke-width="8"/>{eyes}</svg>"##
    )
}

/// Icône de l'application : la mascotte au travail, sans le halo extérieur,
/// cadrée sur le rond.
pub fn icon_svg(color: &str, size_px: u32) -> String {
    svg(color, false, size_px, "SKELVIA Pulse")
        .replacen(r#"viewBox="-30 -30 260 260""#, r#"viewBox="16 16 168 168""#, 1)
        .replacen("<g>", "<g display=\"none\">", 1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_both_states() {
        let awake = super::svg(super::BLUE, false, 64, "Mascotte");
        let asleep = super::svg(super::BLUE, true, 64, "Mascotte");
        assert!(awake.starts_with("<svg") && awake.ends_with("</svg>"));
        assert!(awake.len() > asleep.len(), "le halo n'existe qu'au travail");
        assert!(asleep.contains("<ellipse"));
    }
}
