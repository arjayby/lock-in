//! Commands the dashboard calls. Dates are local `YYYY-MM-DD` strings.

use std::sync::Arc;

use base64::prelude::*;
use chrono::NaiveDate;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::db::{AppInfo, AppTotal, BlockRow, TitleTotal};
use crate::state::{DEFAULT_SKIP_TITLES, Shared};
use crate::{export, icons, time};

type Result<T> = std::result::Result<T, String>;

fn day(date: &str) -> Result<(i64, i64)> {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    Ok(time::day_range(date))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySummary {
    from: i64,
    to: i64,
    active_ms: i64,
    apps: Vec<AppTotal>,
    blocks: Vec<BlockRow>,
}

#[tauri::command]
pub fn day_summary(shared: State<Arc<Shared>>, date: String) -> Result<DaySummary> {
    let (from, to) = day(&date)?;
    let db = shared.db.lock().unwrap();
    let run = || -> rusqlite::Result<DaySummary> {
        Ok(DaySummary {
            from,
            to,
            active_ms: db.active_ms(from, to)?,
            apps: db.app_totals(from, to)?,
            blocks: db.blocks(from, to)?,
        })
    };
    run().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn app_titles(shared: State<Arc<Shared>>, date: String, app_id: String) -> Result<Vec<TitleTotal>> {
    let (from, to) = day(&date)?;
    shared.db.lock().unwrap().app_titles(&app_id, from, to, 20).map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipApp {
    id: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    away_minutes: i64,
    skip_titles: Vec<SkipApp>,
    launch_at_login: bool,
}

#[tauri::command]
pub fn get_settings(app: AppHandle, shared: State<Arc<Shared>>) -> Result<SettingsView> {
    let settings = shared.settings.lock().unwrap().clone();
    let apps = shared.db.lock().unwrap().apps().map_err(|e| e.to_string())?;
    let name = |id: &str| {
        apps.iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone())
            .or_else(|| DEFAULT_SKIP_TITLES.iter().find(|(d, _)| *d == id).map(|(_, n)| n.to_string()))
            .unwrap_or_else(|| id.to_string())
    };
    Ok(SettingsView {
        away_minutes: settings.away_minutes,
        skip_titles: settings.skip_titles.iter().map(|id| SkipApp { id: id.clone(), name: name(id) }).collect(),
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
    })
}

#[tauri::command]
pub fn seen_apps(shared: State<Arc<Shared>>) -> Result<Vec<AppInfo>> {
    shared.db.lock().unwrap().apps().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_away_minutes(shared: State<Arc<Shared>>, minutes: i64) -> Result<()> {
    if !(1..=120).contains(&minutes) {
        return Err("Away time must be between 1 and 120 minutes".into());
    }
    shared.set_away_minutes(minutes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_skip_titles(shared: State<Arc<Shared>>, app_ids: Vec<String>) -> Result<()> {
    shared.set_skip_titles(app_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<()> {
    let launcher = app.autolaunch();
    if enabled { launcher.enable() } else { launcher.disable() }.map_err(|e| e.to_string())
}

/// Writes blocks from the start of `from` to the end of `to` as CSV. Returns the row count.
#[tauri::command]
pub fn export_csv(shared: State<Arc<Shared>>, from: String, to: String, path: String) -> Result<usize> {
    let (start, _) = day(&from)?;
    let (_, end) = day(&to)?;
    if end <= start {
        return Err("The end date is before the start date".into());
    }
    let blocks = shared.db.lock().unwrap().blocks(start, end).map_err(|e| e.to_string())?;
    std::fs::write(&path, export::to_csv(&blocks)).map_err(|e| e.to_string())?;
    Ok(blocks.len())
}

/// An app's icon as a PNG data URL, or `None` if it has none. Extracted once, then cached.
#[tauri::command]
pub fn app_icon(shared: State<Arc<Shared>>, app_id: String) -> Result<Option<String>> {
    let db = shared.db.lock().unwrap();
    let png = match db.icon(&app_id).map_err(|e| e.to_string())? {
        Some(cached) => cached,
        None => {
            let path = db.apps().map_err(|e| e.to_string())?.into_iter().find(|a| a.id == app_id).and_then(|a| a.path);
            let png = path.and_then(|p| icons::app_icon_png(&p));
            db.set_icon(&app_id, png.as_deref()).map_err(|e| e.to_string())?;
            png
        }
    };
    Ok(png.map(|png| format!("data:image/png;base64,{}", BASE64_STANDARD.encode(png))))
}
