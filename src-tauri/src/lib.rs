mod commands;
mod db;
mod export;
mod icons;
mod probe;
mod service;
mod state;
mod time;
mod tracker;
mod tray;

use std::sync::Arc;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use db::Db;
use state::Shared;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let shared = Arc::new(Shared::load(Db::open(&dir.join("lock-in.sqlite3"))?)?);
            app.manage(shared.clone());
            enable_launch_at_login_once(app.handle(), &shared)?;

            tray::create(app.handle())?;
            service::start(app.handle().clone(), shared, tray::refresh);

            // Dev builds open the dashboard right away instead of waiting for the menu.
            #[cfg(debug_assertions)]
            tray::show_dashboard(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the dashboard only hides it; tracking keeps running.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::day_summary,
            commands::app_titles,
            commands::get_settings,
            commands::seen_apps,
            commands::set_away_minutes,
            commands::set_skip_titles,
            commands::set_launch_at_login,
            commands::export_csv,
            commands::app_icon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Turns on launch at login the first time the app runs. Dev builds skip it so the debug
/// binary doesn't get registered.
fn enable_launch_at_login_once(app: &tauri::AppHandle, shared: &Shared) -> Result<(), Box<dyn std::error::Error>> {
    const KEY: &str = "launch_at_login_set";
    let db = shared.db.lock().unwrap();
    if cfg!(debug_assertions) || db.setting(KEY)?.is_some() {
        return Ok(());
    }
    app.autolaunch().enable()?;
    db.set_setting(KEY, "1")?;
    Ok(())
}
