use serde::{Serialize, Serializer};

#[derive(Debug, Clone)]
pub enum Category {
    DB(String),
    Redis(String),
    API(String),
    System,
    BusinessLogic(String),
    Generic,
    Cache(String),
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Category::DB(detail) => serializer.serialize_str(&format!("DB:{}", detail)),
            Category::Redis(detail) => serializer.serialize_str(&format!("Redis:{}", detail)),
            Category::API(detail) => serializer.serialize_str(&format!("API:{}", detail)),
            Category::System => serializer.serialize_str("System"),
            Category::BusinessLogic(detail) => {
                serializer.serialize_str(&format!("BusinessLogic:{}", detail))
            }
            Category::Generic => serializer.serialize_str("Generic"),
            Category::Cache(detail) => serializer.serialize_str(&format!("Cache:{}", detail)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Info,
    Debug,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Info => write!(f, "info"),
            Level::Debug => write!(f, "debug"),
            Level::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: Level,
    pub category: Category,
    pub api_name: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub label: String,
    pub value: serde_json::Value,
    pub hostname: String,
    pub message_number: u64,
    pub env: String,
}

impl LogEntry {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "level": self.level.to_string(),
            "category": self.category,
            "apiName": self.api_name,
            "requestId": self.request_id,
            "sessionId": self.session_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "label": self.label,
            "value": self.value,
            "hostname": self.hostname,
            "messageNumber": self.message_number,
            "env": self.env,
        })
    }
}
