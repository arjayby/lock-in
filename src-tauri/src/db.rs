//! SQLite storage for blocks, apps, icons and settings.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Result, params};
use serde::Serialize;

use crate::tracker::{Block, Kind, Op};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS apps (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT
);
CREATE TABLE IF NOT EXISTS blocks (
    id     INTEGER PRIMARY KEY,
    start  INTEGER NOT NULL,
    end    INTEGER NOT NULL,
    kind   TEXT NOT NULL,
    app_id TEXT,
    title  TEXT
);
CREATE INDEX IF NOT EXISTS blocks_start ON blocks(start);
CREATE TABLE IF NOT EXISTS icons (
    app_id TEXT PRIMARY KEY,
    png    BLOB
);
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRow {
    pub start: i64,
    pub end: i64,
    pub kind: Kind,
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTotal {
    pub app_id: String,
    pub name: String,
    pub ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleTotal {
    pub title: String,
    pub ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
}

pub struct Db {
    conn: Connection,
    /// Row of the block the tracker is currently extending.
    open_id: Option<i64>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn, open_id: None })
    }

    pub fn apply(&mut self, ops: &[Op]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for op in ops {
            match op {
                Op::SetEnd(end) => {
                    let Some(id) = self.open_id else { continue };
                    tx.execute("UPDATE blocks SET end = ?1 WHERE id = ?2", params![end, id])?;
                    tx.execute("DELETE FROM blocks WHERE id = ?1 AND end <= start", [id])?;
                }
                Op::Start(Block { start, end, activity }) => {
                    let w = activity.window.as_ref();
                    tx.execute(
                        "INSERT INTO blocks (start, end, kind, app_id, title) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            start,
                            end,
                            activity.kind.as_str(),
                            w.map(|w| &w.app_id),
                            w.map(|w| &w.title),
                        ],
                    )?;
                    self.open_id = Some(tx.last_insert_rowid());
                }
            }
        }
        tx.commit()
    }

    /// Forgets the open block so the next `Start` doesn't touch it.
    pub fn close_open(&mut self) {
        self.open_id = None;
    }

    pub fn upsert_app(&self, id: &str, name: &str, path: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO apps (id, name, path) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, path = excluded.path",
            params![id, name, path],
        )?;
        Ok(())
    }

    pub fn apps(&self) -> Result<Vec<AppInfo>> {
        let mut stmt = self.conn.prepare("SELECT id, name, path FROM apps ORDER BY name COLLATE NOCASE")?;
        stmt.query_map([], |r| Ok(AppInfo { id: r.get(0)?, name: r.get(1)?, path: r.get(2)? }))?
            .collect()
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
            .optional()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn delete_setting(&self, key: &str) -> Result<()> {
        self.conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
        Ok(())
    }

    /// Erases stored titles for an app (when it's added to the skip-titles list).
    pub fn blank_titles(&self, app_id: &str) -> Result<()> {
        self.conn.execute("UPDATE blocks SET title = '' WHERE app_id = ?1", [app_id])?;
        Ok(())
    }

    /// Blocks starting in `[from, to)`, oldest first. Blocks never span midnight, so a day's
    /// blocks are exactly those that start in it.
    pub fn blocks(&self, from: i64, to: i64) -> Result<Vec<BlockRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT b.start, b.end, b.kind, b.app_id, a.name, b.title
             FROM blocks b LEFT JOIN apps a ON a.id = b.app_id
             WHERE b.start >= ?1 AND b.start < ?2 ORDER BY b.start",
        )?;
        stmt.query_map(params![from, to], |r| {
            Ok(BlockRow {
                start: r.get(0)?,
                end: r.get(1)?,
                kind: Kind::parse(&r.get::<_, String>(2)?),
                app_id: r.get(3)?,
                app_name: r.get(4)?,
                title: r.get(5)?,
            })
        })?
        .collect()
    }

    /// Active time per app in `[from, to)`, most used first.
    pub fn app_totals(&self, from: i64, to: i64) -> Result<Vec<AppTotal>> {
        let mut stmt = self.conn.prepare(
            "SELECT b.app_id, COALESCE(a.name, b.app_id), SUM(b.end - b.start) AS ms
             FROM blocks b LEFT JOIN apps a ON a.id = b.app_id
             WHERE b.kind = 'active' AND b.start >= ?1 AND b.start < ?2
             GROUP BY b.app_id ORDER BY ms DESC",
        )?;
        stmt.query_map(params![from, to], |r| Ok(AppTotal { app_id: r.get(0)?, name: r.get(1)?, ms: r.get(2)? }))?
            .collect()
    }

    /// Time per window title for one app in `[from, to)`, most used first.
    pub fn app_titles(&self, app_id: &str, from: i64, to: i64, limit: i64) -> Result<Vec<TitleTotal>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(title, ''), SUM(end - start) AS ms FROM blocks
             WHERE kind = 'active' AND app_id = ?1 AND start >= ?2 AND start < ?3
             GROUP BY 1 ORDER BY ms DESC LIMIT ?4",
        )?;
        stmt.query_map(params![app_id, from, to, limit], |r| Ok(TitleTotal { title: r.get(0)?, ms: r.get(1)? }))?
            .collect()
    }

    pub fn active_ms(&self, from: i64, to: i64) -> Result<i64> {
        self.conn.query_row(
            "SELECT COALESCE(SUM(end - start), 0) FROM blocks
             WHERE kind = 'active' AND start >= ?1 AND start < ?2",
            params![from, to],
            |r| r.get(0),
        )
    }

    pub fn icon(&self, app_id: &str) -> Result<Option<Option<Vec<u8>>>> {
        self.conn
            .query_row("SELECT png FROM icons WHERE app_id = ?1", [app_id], |r| r.get(0))
            .optional()
    }

    /// Stores an icon, or `None` to remember that the app has none.
    pub fn set_icon(&self, app_id: &str, png: Option<&[u8]>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO icons (app_id, png) VALUES (?1, ?2)
             ON CONFLICT(app_id) DO UPDATE SET png = excluded.png",
            params![app_id, png],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::{Activity, Window};

    fn active(app: &str, title: &str) -> Activity {
        Activity {
            kind: Kind::Active,
            window: Some(Window { app_id: app.into(), app_name: app.into(), title: title.into() }),
        }
    }

    fn start(start: i64, end: i64, activity: Activity) -> Op {
        Op::Start(Block { start, end, activity })
    }

    #[test]
    fn applies_ops_and_totals_active_time() {
        let mut db = Db::open_in_memory().unwrap();
        db.upsert_app("a", "App A", None).unwrap();
        db.apply(&[start(0, 0, active("a", "x"))]).unwrap();
        db.apply(&[Op::SetEnd(4000)]).unwrap();
        db.apply(&[Op::SetEnd(4000), start(4000, 6000, Activity { kind: Kind::Away, window: None })]).unwrap();
        db.apply(&[Op::SetEnd(9000), start(9000, 10_000, active("a", "y"))]).unwrap();

        assert_eq!(db.active_ms(0, 100_000).unwrap(), 5000);
        assert_eq!(
            db.app_totals(0, 100_000).unwrap(),
            vec![AppTotal { app_id: "a".into(), name: "App A".into(), ms: 5000 }]
        );
        assert_eq!(
            db.app_titles("a", 0, 100_000, 10).unwrap(),
            vec![TitleTotal { title: "x".into(), ms: 4000 }, TitleTotal { title: "y".into(), ms: 1000 }]
        );
        assert_eq!(db.blocks(0, 100_000).unwrap().len(), 3);
    }

    #[test]
    fn empty_blocks_are_deleted() {
        let mut db = Db::open_in_memory().unwrap();
        db.apply(&[start(4000, 4000, active("a", "x"))]).unwrap();
        db.apply(&[Op::SetEnd(4000), start(4000, 6000, Activity { kind: Kind::Away, window: None })]).unwrap();
        let blocks = db.blocks(0, 100_000).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind, Kind::Away);
    }

    #[test]
    fn closed_block_is_not_touched() {
        let mut db = Db::open_in_memory().unwrap();
        db.apply(&[start(0, 2000, active("a", "x"))]).unwrap();
        db.close_open();
        db.apply(&[Op::SetEnd(99_000)]).unwrap();
        assert_eq!(db.blocks(0, 100_000).unwrap()[0].end, 2000);
    }

    #[test]
    fn blanks_titles_and_stores_settings() {
        let mut db = Db::open_in_memory().unwrap();
        db.apply(&[start(0, 2000, active("a", "secret"))]).unwrap();
        db.blank_titles("a").unwrap();
        assert_eq!(db.blocks(0, 100_000).unwrap()[0].title.as_deref(), Some(""));

        assert_eq!(db.setting("k").unwrap(), None);
        db.set_setting("k", "1").unwrap();
        db.set_setting("k", "2").unwrap();
        assert_eq!(db.setting("k").unwrap().as_deref(), Some("2"));
        db.delete_setting("k").unwrap();
        assert_eq!(db.setting("k").unwrap(), None);
    }
}
