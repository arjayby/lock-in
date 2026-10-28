mod db;
mod probe;
mod service;
mod state;
mod time;
mod tracker;

use std::sync::Arc;

use tauri::Manager;

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

            service::start(app.handle().clone(), shared, |_| {});
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
