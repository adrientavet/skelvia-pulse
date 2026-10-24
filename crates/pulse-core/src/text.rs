//! Petits outils de texte : comparaison sans accents ni majuscules, domaine
//! d'une adresse web.

/// Met en minuscules, retire les accents courants du français et unifie les
/// apostrophes, pour que « Factures », « facture » et « FACTURÉ » se
/// comparent simplement.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        for l in c.to_lowercase() {
            let r = match l {
                'à' | 'â' | 'ä' | 'á' | 'ã' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'î' | 'ï' | 'í' => 'i',
                'ô' | 'ö' | 'ó' | 'õ' => 'o',
                'ù' | 'û' | 'ü' | 'ú' => 'u',
                'ÿ' => 'y',
                'ç' => 'c',
                'œ' => {
                    out.push('o');
                    'e'
                }
                'æ' => {
                    out.push('a');
                    'e'
                }
                '\u{2019}' | '\u{2018}' | '`' | '\u{b4}' => '\'',
                '\u{a0}' | '\u{202f}' => ' ',
                other => other,
            };
            out.push(r);
        }
    }
    out
}

/// Vrai si `needle` apparaît dans `haystack` au début d'un mot. Les deux
/// textes doivent déjà être passés par [`fold`].
///
/// « devis » trouve « Devis_Martin.xlsx » et « mes-devis », mais « plan » ne
/// trouve pas « déplanter ». Un texte vide ne trouve jamais rien.
pub fn contains_word(haystack: &str, needle: &str) -> bool {
    let needle = needle.trim();
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let at = start + pos;
        let before = haystack[..at].chars().next_back();
        if before.is_none_or(|c| !c.is_alphanumeric()) {
            return true;
        }
        // Avance d'un caractère entier (UTF-8).
        start = at + haystack[at..].chars().next().map_or(1, |c| c.len_utf8());
    }
    false
}

/// Domaine d'une adresse web (`https://www.boamp.fr/avis` → `boamp.fr`).
/// Renvoie `None` pour les pages internes du navigateur et les fichiers
/// locaux (`chrome://`, `file://`, `about:`…).
pub fn domain_of(url: &str) -> Option<String> {
    let lower = url.trim().to_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next()?;
    // Retire un éventuel « utilisateur@ » et le port.
    let host = host.rsplit('@').next()?;
    let host = host.split(':').next()?;
    let host = host.strip_prefix("www.").unwrap_or(host);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_removes_accents_and_case() {
        assert_eq!(fold("Facturé À Œuvre"), "facture a oeuvre");
        assert_eq!(fold("Appel d’offres"), "appel d'offres");
    }

    #[test]
    fn word_start_matching() {
        assert!(contains_word(&fold("Devis_Martin.xlsx - Excel"), "devis"));
        assert!(contains_word(&fold("mes-devis 2026"), "devis"));
        assert!(contains_word(&fold("Factures impayées"), "facture"));
        assert!(!contains_word(&fold("déplanter"), "plan"));
        assert!(!contains_word("abc", ""));
        assert!(contains_word(&fold("CCTP lot 2.pdf"), "cctp"));
    }

    #[test]
    fn domains() {
        assert_eq!(domain_of("https://www.boamp.fr/avis/123").as_deref(), Some("boamp.fr"));
        assert_eq!(domain_of("http://localhost:5600/").as_deref(), Some("localhost"));
        assert_eq!(domain_of("https://user@mail.google.com/x").as_deref(), Some("mail.google.com"));
        assert_eq!(domain_of("chrome://settings"), None);
        assert_eq!(domain_of("file:///C:/CCTP.pdf"), None);
    }
}
