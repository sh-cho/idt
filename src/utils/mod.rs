use chrono::Utc;

/// Get current timestamp in milliseconds since Unix epoch
pub fn current_timestamp_ms() -> u64 {
    Utc::now().timestamp_millis() as u64
}

/// Get current timestamp in seconds since Unix epoch
pub fn current_timestamp_secs() -> u64 {
    Utc::now().timestamp() as u64
}

/// Format milliseconds as human-readable duration
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{} ms", ms)
    } else if ms < 60_000 {
        format!("{:.2} seconds", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{:.2} minutes", ms as f64 / 60_000.0)
    } else if ms < 86_400_000 {
        format!("{:.2} hours", ms as f64 / 3_600_000.0)
    } else {
        format!("{:.2} days", ms as f64 / 86_400_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp_ms() {
        let ts = current_timestamp_ms();
        assert!(ts > 1_000_000_000_000); // after 2001
    }

    #[test]
    fn test_current_timestamp_secs() {
        let ts = current_timestamp_secs();
        assert!(ts > 1_000_000_000); // after 2001
    }

    #[test]
    fn test_format_duration_ms_millis() {
        assert_eq!(format_duration_ms(500), "500 ms");
        assert_eq!(format_duration_ms(0), "0 ms");
        assert_eq!(format_duration_ms(999), "999 ms");
    }

    #[test]
    fn test_format_duration_ms_seconds() {
        assert_eq!(format_duration_ms(1000), "1.00 seconds");
        assert_eq!(format_duration_ms(30_000), "30.00 seconds");
    }

    #[test]
    fn test_format_duration_ms_minutes() {
        assert_eq!(format_duration_ms(60_000), "1.00 minutes");
        assert_eq!(format_duration_ms(120_000), "2.00 minutes");
    }

    #[test]
    fn test_format_duration_ms_hours() {
        assert_eq!(format_duration_ms(3_600_000), "1.00 hours");
        assert_eq!(format_duration_ms(7_200_000), "2.00 hours");
    }

    #[test]
    fn test_format_duration_ms_days() {
        assert_eq!(format_duration_ms(86_400_000), "1.00 days");
        assert_eq!(format_duration_ms(172_800_000), "2.00 days");
    }
}
