use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub source: Option<String>,
    pub context_window: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl Idea {
    pub fn new(title: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            content: None,
            source: None,
            context_window: None,
            tags: vec![],
            created_at: Utc::now(),
        }
    }
}
