//! State shared by the tracker loop, the tray and the dashboard commands.

use std::sync::Mutex;

use rusqlite::Result;

use crate::db::Db;

/// Apps whose window titles are never stored, until the user changes the list.
pub const DEFAULT_SKIP_TITLES: [(&str, &str); 4] = [
    ("com.1password.1password", "1Password"),
    ("com.agilebits.onepassword7", "1Password 7"),
    ("com.apple.keychainaccess", "Keychain Access"),
    ("com.apple.systempreferences", "System Settings"),
];

const AWAY_MINUTES: &str = "away_minutes";
const SKIP_TITLES: &str = "skip_titles";
const PAUSED_UNTIL: &str = "paused_until";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pause {
    Off,
    Until(i64),
    Forever,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub away_minutes: i64,
    pub skip_titles: Vec<String>,
}

pub struct Shared {
    pub db: Mutex<Db>,
    pub settings: Mutex<Settings>,
    pause: Mutex<Pause>,
}

impl Shared {
    pub fn load(db: Db) -> Result<Self> {
        let away_minutes = db.setting(AWAY_MINUTES)?.and_then(|v| v.parse().ok()).unwrap_or(5);
        let skip_titles = match db.setting(SKIP_TITLES)? {
            Some(v) => serde_json::from_str(&v).unwrap_or_default(),
            None => DEFAULT_SKIP_TITLES.iter().map(|(id, _)| id.to_string()).collect(),
        };
        let pause = match db.setting(PAUSED_UNTIL)?.as_deref() {
            None => Pause::Off,
            Some("forever") => Pause::Forever,
            Some(v) => v.parse().map(Pause::Until).unwrap_or(Pause::Off),
        };
        Ok(Self {
            db: Mutex::new(db),
            settings: Mutex::new(Settings { away_minutes, skip_titles }),
            pause: Mutex::new(pause),
        })
    }

    pub fn pause(&self) -> Pause {
        *self.pause.lock().unwrap()
    }

    /// Sets and persists the pause, so it survives quitting.
    pub fn set_pause(&self, pause: Pause) -> Result<()> {
        let db = self.db.lock().unwrap();
        match pause {
            Pause::Off => db.delete_setting(PAUSED_UNTIL)?,
            Pause::Until(t) => db.set_setting(PAUSED_UNTIL, &t.to_string())?,
            Pause::Forever => db.set_setting(PAUSED_UNTIL, "forever")?,
        }
        *self.pause.lock().unwrap() = pause;
        Ok(())
    }

    pub fn set_away_minutes(&self, minutes: i64) -> Result<()> {
        self.db.lock().unwrap().set_setting(AWAY_MINUTES, &minutes.to_string())?;
        self.settings.lock().unwrap().away_minutes = minutes;
        Ok(())
    }

    /// Replaces the skip-titles list and erases stored titles of newly added apps.
    pub fn set_skip_titles(&self, app_ids: Vec<String>) -> Result<()> {
        let db = self.db.lock().unwrap();
        let mut settings = self.settings.lock().unwrap();
        for id in app_ids.iter().filter(|id| !settings.skip_titles.contains(id)) {
            db.blank_titles(id)?;
        }
        db.set_setting(SKIP_TITLES, &serde_json::to_string(&app_ids).unwrap())?;
        settings.skip_titles = app_ids;
        Ok(())
    }
}
