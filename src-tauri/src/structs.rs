use std::{collections::HashMap, fs::File, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri_plugin_fs::FilePath;

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

#[derive(Debug, Clone, Serialize)]
pub struct PickedFile {
    pub path: FilePath,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStats {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub path: FilePath,
    pub name: String,
    pub id: String,
    pub crf: Option<u8>,

    pub stats: VideoStats,
}

pub struct FileReaders(pub Mutex<HashMap<String, File>>);
