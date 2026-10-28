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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::{Activity, Block, Kind, Op, Window};

    fn block(app: &str, title: &str) -> Op {
        let window = Window { app_id: app.into(), app_name: app.into(), title: title.into() };
        Op::Start(Block { start: 0, end: 1000, activity: Activity { kind: Kind::Active, window: Some(window) } })
    }

    #[test]
    fn adding_to_skip_list_erases_only_new_apps_titles() {
        let mut db = Db::open_in_memory().unwrap();
        db.apply(&[block("a", "secret")]).unwrap();
        db.close_open();
        db.apply(&[block("b", "kept")]).unwrap();
        let shared = Shared::load(db).unwrap();

        shared.set_skip_titles(vec!["a".into()]).unwrap();
        let titles = |s: &Shared| -> Vec<Option<String>> {
            s.db.lock().unwrap().blocks(0, 10_000).unwrap().into_iter().map(|b| b.title).collect()
        };
        assert_eq!(titles(&shared), vec![Some("".into()), Some("kept".into())]);
        assert_eq!(shared.settings.lock().unwrap().skip_titles, vec!["a".to_string()]);

        // Removing doesn't bring anything back.
        shared.set_skip_titles(vec![]).unwrap();
        assert_eq!(titles(&shared), vec![Some("".into()), Some("kept".into())]);
    }

    #[test]
    fn pause_and_settings_persist() {
        let shared = Shared::load(Db::open_in_memory().unwrap()).unwrap();
        assert_eq!(shared.pause(), Pause::Off);
        assert_eq!(shared.settings.lock().unwrap().skip_titles.len(), DEFAULT_SKIP_TITLES.len());

        shared.set_pause(Pause::Until(42)).unwrap();
        shared.set_away_minutes(10).unwrap();
        let db = shared.db.into_inner().unwrap();
        let reloaded = Shared::load(db).unwrap();
        assert_eq!(reloaded.pause(), Pause::Until(42));
        assert_eq!(reloaded.settings.lock().unwrap().away_minutes, 10);
    }
}
