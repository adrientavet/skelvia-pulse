//! Le blip SKELVIA (la mascotte) en SVG, pour le rapport HTML et les icônes.
//!
//! En 3D depuis le 24/10/2026 : même modèle que `src/blip3d.ts` (interface) et
//! que skelvia.fr, un galet noir couvert de petites billes de sa couleur, un
//! fin trait de contour, deux yeux posés sur la surface et, au travail, des
//! billes en orbite. Le calcul 3D est fait une fois en JavaScript
//! (`vsl-motion/blip3d/exporter-svg.mjs`) : les dessins sont rangés dans
//! `assets/` et ce module ne fait que poser la taille, le titre et la couleur.
//! Le nom et le dessin du blip sont réservés (voir TRADEMARKS.md).

/// Couleur du blip : le bleu SKELVIA.
pub const BLUE: &str = "#2A63F5";

const TRAVAIL: &str = include_str!("../assets/blip-travail.svg");
const VEILLE: &str = include_str!("../assets/blip-veille.svg");
const SIMPLE_TRAVAIL: &str = include_str!("../assets/blip-simple-travail.svg");
const SIMPLE_VEILLE: &str = include_str!("../assets/blip-simple-veille.svg");
const ICONE: &str = include_str!("../assets/blip-icone.svg");

/// Pose la taille, le titre (échappé) et la couleur sur un dessin rangé.
fn poser(base: &str, color: &str, size_px: u32, title: &str) -> String {
    let titre = title
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;");
    base.trim_end()
        .replace("{TAILLE}", &size_px.to_string())
        .replace("{TITRE}", &titre)
        .replace(BLUE, color)
}

/// SVG complet du blip. `asleep` : état « en veille ».
pub fn svg(color: &str, asleep: bool, size_px: u32, title: &str) -> String {
    poser(if asleep { VEILLE } else { TRAVAIL }, color, size_px, title)
}

/// Version pour les petites tailles (moins de 48 px, icône près de
/// l'horloge) : sphère lisse éclairée de sa couleur, gros yeux, sans billes.
pub fn simple_svg(color: &str, asleep: bool, size_px: u32) -> String {
    poser(if asleep { SIMPLE_VEILLE } else { SIMPLE_TRAVAIL }, color, size_px, "")
}

/// Icône de l'application : le blip au travail, sans les billes en orbite,
/// cadré sur le galet.
pub fn icon_svg(color: &str, size_px: u32) -> String {
    poser(ICONE, color, size_px, "SKELVIA Pulse")
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_both_states() {
        let awake = super::svg(super::BLUE, false, 64, "Blip");
        let asleep = super::svg(super::BLUE, true, 64, "Blip");
        assert!(awake.starts_with("<svg") && awake.ends_with("</svg>"));
        assert!(awake.len() > asleep.len(), "les billes en orbite n'existent qu'au travail");
        assert!(asleep.contains("<ellipse"));
        assert!(!awake.contains("{TAILLE}") && awake.contains(r#"width="64""#));
    }
}
