use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub id: String,

    pub r#type: String,
    pub stage: String,
    pub progress: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
}