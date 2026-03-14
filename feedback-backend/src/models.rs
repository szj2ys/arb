use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct FeedbackSubmit {
    pub category: String, // "bug", "feature", "praise"
    pub content: String,
    pub contact: Option<String>,
    pub version: String,
    pub os: String,
    pub honeypot: Option<String>, // Anti-spam field
}

#[derive(Debug, Serialize)]
pub struct Feedback {
    pub id: String,
    pub category: String,
    pub content: String,
    pub contact: Option<String>,
    pub version: String,
    pub os: String,
    pub status: String, // "new", "acknowledged", "fixed", "closed"
    pub created_at: String,
    pub response: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Serialize)]
pub struct PublicFeedback {
    pub id: String,
    pub category: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
    pub response: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdate {
    pub status: String,
    pub response: Option<String>,
    pub is_public: Option<bool>,
}
