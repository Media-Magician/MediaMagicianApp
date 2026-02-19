use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub time: f64,
    pub duration: f64,
    pub finished: bool,

    pub id: i64,
}
