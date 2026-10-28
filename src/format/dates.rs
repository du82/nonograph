use chrono::{DateTime, Utc};

// e.g. "October 28, 2026".
pub(crate) fn display(dt: DateTime<Utc>) -> String {
    dt.format("%B %d, %Y").to_string()
}

// ISO-8601
pub(crate) fn iso(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT00:00:00+00:00").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn display_uses_full_month_name() {
        assert_eq!(display(at(2026, 10, 28)), "October 28, 2026");
        assert_eq!(display(at(2024, 1, 15)), "January 15, 2024");
    }

    #[test]
    fn iso_is_midnight_utc() {
        assert_eq!(iso(at(2024, 1, 15)), "2024-01-15T00:00:00+00:00");
        assert_eq!(iso(at(2026, 10, 28)), "2026-10-28T00:00:00+00:00");
    }
}
