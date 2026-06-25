use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: String,
    pub content: String,
    pub mood: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Log {
    pub fn new(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            mood: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}
