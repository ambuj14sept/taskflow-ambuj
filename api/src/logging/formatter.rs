use std::sync::atomic::{AtomicU64, Ordering};

use super::types::{Category, Level, LogEntry};

thread_local! {
    static REQUEST_COUNTER: AtomicU64 = const { AtomicU64::new(0) };
}

/// Get the next message number for the current thread/request
pub fn next_message_number() -> u64 {
    REQUEST_COUNTER.with(|counter| counter.fetch_add(1, Ordering::SeqCst) + 1)
}

/// Reset the message counter (call at start of each request)
pub fn reset_message_counter() {
    REQUEST_COUNTER.with(|counter| counter.store(0, Ordering::SeqCst));
}

/// Log a structured entry
pub fn log(level: Level, category: Category, label: impl Into<String>, value: serde_json::Value) {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let entry = LogEntry {
        level,
        category,
        api_name: None,
        request_id: None,
        session_id: None,
        label: label.into(),
        value,
        hostname,
        message_number: next_message_number(),
        env: std::env::var("ENV").unwrap_or_else(|_| "dev".to_string()),
    };

    let json = entry.to_json();
    match level {
        Level::Info => tracing::info!("{}", json),
        Level::Debug => tracing::debug!("{}", json),
        Level::Error => tracing::error!("{}", json),
    }
}

/// Log with request context
pub fn log_with_context(
    level: Level,
    category: Category,
    api_name: &str,
    request_id: &str,
    session_id: Option<&str>,
    label: impl Into<String>,
    value: serde_json::Value,
) {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let entry = LogEntry {
        level,
        category,
        api_name: Some(api_name.to_string()),
        request_id: Some(request_id.to_string()),
        session_id: session_id.map(|s| s.to_string()),
        label: label.into(),
        value,
        hostname,
        message_number: next_message_number(),
        env: std::env::var("ENV").unwrap_or_else(|_| "dev".to_string()),
    };

    let json = entry.to_json();
    match level {
        Level::Info => tracing::info!("{}", json),
        Level::Debug => tracing::debug!("{}", json),
        Level::Error => tracing::error!("{}", json),
    }
}

/// Initialize the tracing subscriber with JSON output
pub fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
}
