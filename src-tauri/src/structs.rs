use std::{collections::HashMap, fs::File, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri_plugin_fs::FilePath;
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ConnectionState {
    Disconnected,
    Connected,
}

#[derive(Debug, Clone)]
pub struct ObsState {
    pub connection: watch::Receiver<ConnectionState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptConfig {
    LazyPrompt,
    NoPrompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSettings {
    pub enabled: bool,
    pub promptConfig: PromptConfig,
    pub password: Option<String>,
    pub port: Option<u16>,
}
impl Default for ClipSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            promptConfig: PromptConfig::NoPrompt,
            password: None,
            port: None,
        }
    }
}

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
    pub id: Option<String>,
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
    pub cuts: Option<VideoParts>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoParts(pub Vec<u32>);

#[derive(Debug, Clone)]
pub struct KeepSegment {
    // all 3 are in seconds
    pub start: f64,
    pub end: f64,
    pub outStart: f64,
}

pub struct FileReaders(pub Mutex<HashMap<String, File>>);

#[derive(Default)]
pub struct VideoRegistry(pub Mutex<HashMap<String, FilePath>>);
