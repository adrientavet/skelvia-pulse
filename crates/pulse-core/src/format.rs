//! Mise en forme des durées et des dates, en français.

use chrono::{Datelike, NaiveDate};

/// « 6 h 05 », « 45 min », « moins d'une minute ».
pub fn duration(secs: f64) -> String {
    let total_min = (secs / 60.0).round() as i64;
    if secs < 30.0 {
        return "moins d'une minute".to_string();
    }
    let h = total_min / 60;
    let m = total_min % 60;
    if h == 0 {
        format!("{m} min")
    } else if m == 0 {
        format!("{h} h")
    } else {
        format!("{h} h {m:02}")
    }
}

/// Version courte pour la pastille : « 3 h 12 », « 8 min ».
pub fn duration_short(secs: f64) -> String {
    if secs < 60.0 {
        return "0 min".to_string();
    }
    duration(secs)
}

const MONTHS: [&str; 12] = [
    "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre",
    "octobre", "novembre", "décembre",
];
const DAYS: [&str; 7] = ["lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche"];

/// « 24 octobre 2026 ».
pub fn date_long(d: NaiveDate) -> String {
    format!("{} {} {}", d.day(), MONTHS[d.month0() as usize], d.year())
}

/// « lun. 24/10 ».
pub fn date_short(d: NaiveDate) -> String {
    let day = DAYS[d.weekday().num_days_from_monday() as usize];
    format!("{}. {:02}/{:02}", &day[..3], d.day(), d.month())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(duration(10.0), "moins d'une minute");
        assert_eq!(duration(45.0 * 60.0), "45 min");
        assert_eq!(duration(6.0 * 3600.0 + 5.0 * 60.0), "6 h 05");
        assert_eq!(duration(2.0 * 3600.0), "2 h");
        assert_eq!(duration_short(20.0), "0 min");
    }

    #[test]
    fn dates() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        assert_eq!(date_long(d), "24 octobre 2026");
        assert_eq!(date_short(d), "sam. 24/10");
    }
}
