#![allow(non_snake_case)]

use std::{collections::HashMap, sync::Mutex};

// #[cfg(not(target_os = "linux"))]
// use tauri::Manager;
#[cfg(target_os = "linux")]
use tauri_runtime_cef;
use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;

mod structs;
mod util;
mod commands {
    pub mod digest;
    pub mod files;
    pub mod wrapProcess;
    pub mod account;
}
mod process {
    pub mod bsf;
    pub mod video;
}

use crate::{
    commands::{
        digest::sha256_digest,
        files::{pick_file, remove_file, write_to_file},
        wrapProcess::process_video,
        account::{start_auth_listener},
    },
    structs::FileReaders, util::cleanCacheDir,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    let mut builder = tauri::Builder::default().runtime(tauri_runtime_cef::Cef::default()
        .sandbox(tauri_runtime_cef::SandboxPolicy::Disabled)
        .command_line_arg("lang", Some("en-US"))
        .command_line_arg("locales-dir-path", Some("/app/bin/locales")));
    #[cfg(not(target_os = "linux"))]
    let mut builder = tauri::Builder::default().runtime(tauri_runtime_wry::Wry {});
    
    #[cfg(desktop)]
    {
        use crate::commands::account::AuthListenerState;

        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .manage(AuthListenerState::default());
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Clean up files that may have stayed from last time
            let _ = cleanCacheDir(app.path().app_cache_dir().unwrap());
            
            // Register all deep links
            #[cfg(not(target_os = "android"))]
            if let Err(e) = app.deep_link().register_all() {
                println!("Warning: failed to register deep links: {}", e);
            }
            
            // Check whether there is a 'current' deep link
            let start_urls = app.deep_link().get_current()?;
            if let Some(urls) = start_urls {
                println!("deep link URLs: {:?}", urls);
            }
            
            // Log deep link triggers
            app.deep_link().on_open_url(|event| {
                let urls = event.urls();
                for url in urls {
                    println!("URL: {}", url);
                }
            });

            Ok(())
        })
        .on_page_load(|webview, payload| {
            use tauri::webview::PageLoadEvent;

            if payload.event() != PageLoadEvent::Finished {
                return;
            }

            let verifyUrl = payload.url();
            if verifyUrl.path() != "/auth/magic-link/verify" {
                return;
            }

            let Ok(callbackJsString) = serde_json::to_string(&format!("https://{}", verifyUrl.host().unwrap().to_string())) else {
                eprintln!("Failed to encode callback url for auth");
                return;
            };

            if let Err(error) = webview.eval(&format!("window.location.replace({});", callbackJsString)) {
                eprintln!("Failed to redirect to dashboard. {}", error);
            }
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(FileReaders(Mutex::new(HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            process_video,
            sha256_digest,
            pick_file,
            remove_file,
            start_auth_listener,
            write_to_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    eprintln!("After build");
}
