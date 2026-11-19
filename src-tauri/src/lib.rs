#![allow(non_snake_case)]

use std::{collections::HashMap, sync::Mutex};

use tauri::{Manager, Emitter, async_runtime::spawn_blocking};
use tauri_plugin_deep_link::DeepLinkExt;
#[cfg(target_os = "linux")]
use tauri_runtime_cef;

mod structs;
mod util;
mod commands {
    pub mod account;
    pub mod digest;
    pub mod files;
    pub mod wrapProcess;
    pub mod state;
}
mod process {
    pub mod bsf;
    pub mod video;
}
mod utility {
    #[cfg(not(target_os = "android"))]
    pub mod obs;
    pub mod servefile;
}

use crate::{
    commands::{
        account::start_auth_listener,
        digest::sha256_digest,
        files::{pick_file, remove_file, write_to_file},
        wrapProcess::process_video,
        state::obs_state,
    }, structs::{FileReaders, ObsState, VideoRegistry}, util::cleanCacheDir, utility::servefile::serve,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    let mut builder = tauri::Builder::default().runtime(
        tauri_runtime_cef::Cef::default()
            .sandbox(tauri_runtime_cef::SandboxPolicy::Disabled)
            .command_line_arg("lang", Some("en-US"))
            .command_line_arg("locales-dir-path", Some("/app/bin/locales")),
    );
    #[cfg(not(target_os = "linux"))]
    let mut builder = tauri::Builder::default().runtime(tauri_runtime_wry::Wry {});

    #[cfg(desktop)]
    {
        use crate::commands::account::AuthListenerState;

        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                let _ = app
                    .get_webview_window("main")
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

            // Handle listening to OBS replay buffers
            #[cfg(not(target_os = "android"))]
            {
                use tauri_plugin_store::StoreExt;
                use crate::{structs::ClipSettings, utility::obs::obsListen};

                let store = app.store("settings.json").map_err(|e| e.to_string())?;
                let clipSettings = store.get("clipsettings").and_then(|v| serde_json::from_value::<ClipSettings>(v).ok()).unwrap_or_default();
                println!("Clip settings: {:#?}", clipSettings);

                let (state_tx, state_rx) = tokio::sync::watch::channel(crate::structs::ConnectionState::Disconnected);
                app.manage(ObsState { connection: state_rx.clone() });

                if clipSettings.enabled {
                    let handle = app.handle().clone();
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<std::path::PathBuf>();

                    tauri::async_runtime::spawn(obsListen(tx, state_tx, clipSettings));
                    tauri::async_runtime::spawn(async move {
                        while let Some(path) = rx.recv().await {
                            use std::time::{SystemTime, UNIX_EPOCH};
                            use tauri_plugin_fs::FilePath;

                            let id = format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
                            handle.state::<VideoRegistry>().0.lock().unwrap().insert(id.clone(), FilePath::Path(path.clone()));

                            let _ = handle.emit("replay-saved", (id, path.to_string_lossy().to_string()));
                        }
                    });
                }
            }

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

            let Ok(callbackJsString) = serde_json::to_string(&format!(
                "https://{}",
                verifyUrl.host().unwrap().to_string()
            )) else {
                eprintln!("Failed to encode callback url for auth");
                return;
            };

            if let Err(error) =
                webview.eval(&format!("window.location.replace({});", callbackJsString))
            {
                eprintln!("Failed to redirect to dashboard. {}", error);
            }
        })
        .register_asynchronous_uri_scheme_protocol("video", |ctx, req, responder| {
            let app = ctx.app_handle().clone();
            spawn_blocking(move || responder.respond(serve(&app, &req)));
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(FileReaders(Mutex::new(HashMap::new())))
        .manage(VideoRegistry(Mutex::new(HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            process_video,
            sha256_digest,
            pick_file,
            remove_file,
            start_auth_listener,
            write_to_file,

            obs_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    eprintln!("After build");
}
