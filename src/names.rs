//! Small standalone helpers with no external dependencies.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current calendar year (UTC). Uses Howard Hinnant's `civil_from_days`
/// algorithm so we don't need a date/time dependency just for a copyright year.
pub fn current_year() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    year_from_days(days)
}

fn year_from_days(days: i64) -> i64 {
    // days is days since 1970-01-01.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y + 1
    } else {
        y
    }
}

#[cfg(test)]
mod tests {
    use super::year_from_days;

    #[test]
    fn known_epochs() {
        assert_eq!(year_from_days(0), 1970); // 1970-01-01
        assert_eq!(year_from_days(18_993), 2022); // 2022-01-01
        assert_eq!(year_from_days(20_454), 2026); // 2026-01-01
    }
}
