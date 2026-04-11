/// Format a byte count into a human-readable string (e.g., "1.5 GiB").
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let mut value = bytes as f64;
    for unit in UNITS {
        if value < 1024.0 {
            return if value.fract() < 0.05 {
                format!("{:.0} {}", value, unit)
            } else {
                format!("{:.1} {}", value, unit)
            };
        }
        value /= 1024.0;
    }
    format!("{:.1} PiB", value)
}

/// Format a percentage value (0.0–100.0) with one decimal place.
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format a duration in seconds into a human-readable string (e.g., "3d 12:05:30").
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {:02}:{:02}:{:02}", days, hours, minutes, secs)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1 KiB");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(1073741824), "1 GiB");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(0.0), "0.0%");
        assert_eq!(format_percentage(99.9), "99.9%");
        assert_eq!(format_percentage(100.0), "100.0%");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "00:00:00");
        assert_eq!(format_duration(3661), "01:01:01");
        assert_eq!(format_duration(90061), "1d 01:01:01");
    }
}
