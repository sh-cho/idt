use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in milliseconds since Unix epoch
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Get current timestamp in seconds since Unix epoch
pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
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
