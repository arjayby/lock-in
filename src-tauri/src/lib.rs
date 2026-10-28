mod db;
mod probe;
mod service;
mod state;
mod time;
mod tracker;
mod tray;

use std::sync::Arc;

use tauri::{Manager, WindowEvent};

use db::Db;
use state::Shared;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let shared = Arc::new(Shared::load(Db::open(&dir.join("lock-in.sqlite3"))?)?);
            app.manage(shared.clone());

            tray::create(app.handle())?;
            service::start(app.handle().clone(), shared, tray::refresh);
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the dashboard only hides it; tracking keeps running.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
