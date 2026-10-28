//! The background loop: sample every 2 s, feed the tracker, store the result.

use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::AppHandle;

use crate::state::{Pause, Shared};
use crate::tracker::{Sample, Tracker};
use crate::{probe, time};

const POLL: Duration = Duration::from_secs(2);

/// Called once a minute and whenever the pause state changes on its own.
pub type OnTick = fn(&AppHandle);

pub fn start(app: AppHandle, shared: Arc<Shared>, on_tick: OnTick) {
    thread::spawn(move || {
        let mut tracker = Tracker::new(0, Box::new(time::day_start));
        let mut seen_apps = HashSet::new();
        let mut last_minute = -1;

        loop {
            let now = time::now_ms();

            let mut pause = shared.pause();
            if let Pause::Until(t) = pause
                && now >= t
            {
                pause = Pause::Off;
                if let Err(e) = shared.set_pause(pause) {
                    eprintln!("failed to end pause: {e}");
                }
                last_minute = -1;
            }

            {
                let settings = shared.settings.lock().unwrap();
                tracker.set_away_after_ms(settings.away_minutes * 60_000);
                tracker.set_skip_titles(settings.skip_titles.clone());
            }

            let paused = pause != Pause::Off;
            let front = if paused { None } else { probe::front_app() };
            let sample = Sample {
                at: now,
                idle_ms: probe::idle_ms(),
                locked: probe::screen_locked(),
                paused,
                window: front.as_ref().map(|f| f.window.clone()),
            };
            let ops = tracker.observe(&sample);

            {
                let mut db = shared.db.lock().unwrap();
                if let Some(f) = &front
                    && seen_apps.insert(f.window.app_id.clone())
                    && let Err(e) = db.upsert_app(&f.window.app_id, &f.window.app_name, f.bundle_path.as_deref())
                {
                    eprintln!("failed to save app: {e}");
                }
                if let Err(e) = db.apply(&ops) {
                    eprintln!("failed to save sample: {e}");
                }
            }

            if now / 60_000 != last_minute {
                last_minute = now / 60_000;
                on_tick(&app);
            }

            thread::sleep(POLL);
        }
    });
}
