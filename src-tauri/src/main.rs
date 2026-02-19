// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use ffmpeg_sidecar::download::auto_download;

fn main() {
    #[cfg(not(target_os = "android"))]
    auto_download().expect("Failed to auto-download ffmpeg sidecar.");

    media_magician_lib::run();
}
