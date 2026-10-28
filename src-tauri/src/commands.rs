//! Commands the dashboard calls. Dates are local `YYYY-MM-DD` strings.

use std::sync::Arc;

use chrono::NaiveDate;
use serde::Serialize;
use tauri::State;

use crate::db::{AppTotal, BlockRow, TitleTotal};
use crate::state::Shared;
use crate::time;

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
