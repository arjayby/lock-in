//! Menu bar icon, today's total and the pause menu.

use std::sync::Arc;

use chrono::{Local, TimeZone};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::state::{Pause, Shared};
use crate::time;

const TRAY_ID: &str = "main";

struct TrayMenu {
    status: MenuItem<Wry>,
    pause: Submenu<Wry>,
    resume: MenuItem<Wry>,
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let item = |id: &str, text: &str, enabled: bool| MenuItem::with_id(app, id, text, enabled, None::<&str>);

    let status = item("status", "Today: 0m", false)?;
    let pause = Submenu::with_items(
        app,
        "Pause Tracking",
        true,
        &[
            &item("pause_30", "For 30 Minutes", true)?,
            &item("pause_60", "For 1 Hour", true)?,
            &item("pause_forever", "Until I Resume", true)?,
        ],
    )?;
    let resume = item("resume", "Resume Tracking", false)?;
    let menu = Menu::with_items(
        app,
        &[
            &status,
            &PredefinedMenuItem::separator(app)?,
            &item("open", "Open Dashboard", true)?,
            &pause,
            &resume,
            &PredefinedMenuItem::separator(app)?,
            &item("quit", "Quit Lock In", true)?,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(include_bytes!("../icons/tray.png"))?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .build(app)?;

    app.manage(TrayMenu { status, pause, resume });
    refresh(app);
    Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
    let now = time::now_ms();
    let pause = match id {
        "open" => return show_dashboard(app),
        "quit" => return app.exit(0),
        "pause_30" => Pause::Until(now + 30 * 60_000),
        "pause_60" => Pause::Until(now + 60 * 60_000),
        "pause_forever" => Pause::Forever,
        "resume" => Pause::Off,
        _ => return,
    };
    if let Err(e) = app.state::<Arc<Shared>>().set_pause(pause) {
        eprintln!("failed to save pause: {e}");
    }
    refresh(app);
}

pub fn show_dashboard(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Updates today's total and the pause items.
pub fn refresh(app: &AppHandle) {
    let shared = app.state::<Arc<Shared>>();
    let (from, to) = time::day_range(time::date_of(time::now_ms()));
    let active = shared.db.lock().unwrap().active_ms(from, to).unwrap_or_default();
    let total = time::format_duration(active);
    let pause = shared.pause();

    let status = match pause {
        Pause::Off => format!("Today: {total}"),
        Pause::Until(t) => {
            let until = Local.timestamp_millis_opt(t).single().map(|d| d.format("%H:%M").to_string());
            format!("Paused until {}", until.unwrap_or_default())
        }
        Pause::Forever => "Paused".to_string(),
    };
    let title = if pause == Pause::Off { total } else { format!("⏸ {total}") };

    let menu = app.state::<TrayMenu>();
    let _ = menu.status.set_text(status);
    let _ = menu.pause.set_enabled(pause == Pause::Off);
    let _ = menu.resume.set_enabled(pause != Pause::Off);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_title(Some(title));
    }
}
