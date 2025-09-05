use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs > 60 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn validate_url(url: &str) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("rtsp://") {
        Ok(())
    } else {
        Err("URL must start with http://, https://, or rtsp://".to_string())
    }
}

pub fn generate_unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}