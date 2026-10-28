//! Turns 2-second samples of "what is happening right now" into time blocks.
//!
//! This module is pure: it knows nothing about macOS or SQLite. The tracker loop feeds it
//! [`Sample`]s and applies the [`Op`]s it returns to the database.

use serde::Serialize;

/// Samples further apart than this mean the tracker wasn't running (sleep, app quit), so the
/// open block is closed at the last sample instead of being stretched over the gap.
pub const MAX_GAP_MS: i64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Active,
    Away,
    Paused,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Active => "active",
            Kind::Away => "away",
            Kind::Paused => "paused",
        }
    }

    pub fn parse(s: &str) -> Kind {
        match s {
            "active" => Kind::Active,
            "paused" => Kind::Paused,
            _ => Kind::Away,
        }
    }
}

/// The frontmost app and window at sample time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub app_id: String,
    pub app_name: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Sample {
    /// Milliseconds since the Unix epoch.
    pub at: i64,
    /// Milliseconds since the last keyboard or mouse input.
    pub idle_ms: i64,
    pub locked: bool,
    pub paused: bool,
    pub window: Option<Window>,
}

/// What a block is about. Two samples with equal activities belong to the same block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub kind: Kind,
    /// `None` for away and paused blocks.
    pub window: Option<Window>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub start: i64,
    pub end: i64,
    pub activity: Activity,
}

/// A change for the storage layer to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Set the end of the open block. If that leaves it empty (end <= start), delete it.
    SetEnd(i64),
    /// Insert a new open block. The previous open block is final.
    Start(Block),
}

pub struct Tracker {
    away_after_ms: i64,
    skip_titles: Vec<String>,
    /// Returns the start of the local day containing a timestamp.
    day_start: Box<dyn Fn(i64) -> i64 + Send>,
    current: Option<Block>,
}

impl Tracker {
    pub fn new(away_after_ms: i64, day_start: Box<dyn Fn(i64) -> i64 + Send>) -> Self {
        Self {
            away_after_ms,
            skip_titles: Vec::new(),
            day_start,
            current: None,
        }
    }

    pub fn set_away_after_ms(&mut self, ms: i64) {
        self.away_after_ms = ms;
    }

    pub fn set_skip_titles(&mut self, app_ids: Vec<String>) {
        self.skip_titles = app_ids;
    }

    pub fn observe(&mut self, s: &Sample) -> Vec<Op> {
        let mut ops = Vec::new();

        if self.current.as_ref().is_some_and(|cur| s.at - cur.end > MAX_GAP_MS) {
            self.current = None;
        }

        // Split at midnight so no block spans two days.
        if let Some(cur) = &mut self.current {
            let midnight = (self.day_start)(s.at);
            if midnight > cur.start {
                cur.end = midnight;
                ops.push(Op::SetEnd(midnight));
                let next = Block { start: midnight, end: midnight, activity: cur.activity.clone() };
                ops.push(Op::Start(next.clone()));
                self.current = Some(next);
            }
        }

        let (activity, since) = self.classify(s);

        match &mut self.current {
            Some(cur) if cur.activity == activity => {
                cur.end = s.at;
                ops.push(Op::SetEnd(s.at));
            }
            Some(cur) => {
                // Backdate the switch (to the last input, when going idle), but never before
                // the open block started.
                let boundary = since.clamp(cur.start, s.at);
                cur.end = boundary;
                ops.push(Op::SetEnd(boundary));
                let next = Block { start: boundary, end: s.at, activity };
                ops.push(Op::Start(next.clone()));
                self.current = Some(next);
            }
            None => {
                let next = Block { start: s.at, end: s.at, activity };
                ops.push(Op::Start(next.clone()));
                self.current = Some(next);
            }
        }

        ops
    }

    /// What the sample means, and since when it has been true.
    fn classify(&self, s: &Sample) -> (Activity, i64) {
        let away = Activity { kind: Kind::Away, window: None };
        if s.paused {
            return (Activity { kind: Kind::Paused, window: None }, s.at);
        }
        if s.locked {
            return (away, s.at);
        }
        if s.idle_ms >= self.away_after_ms {
            return (away, s.at - s.idle_ms);
        }
        match &s.window {
            Some(w) => {
                let mut w = w.clone();
                if self.skip_titles.contains(&w.app_id) {
                    w.title.clear();
                }
                (Activity { kind: Kind::Active, window: Some(w) }, s.at)
            }
            None => (away, s.at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400_000;
    const MIN: i64 = 60_000;

    fn tracker() -> Tracker {
        Tracker::new(5 * MIN, Box::new(|t| t - t.rem_euclid(DAY)))
    }

    fn win(app: &str, title: &str) -> Option<Window> {
        Some(Window { app_id: app.into(), app_name: app.into(), title: title.into() })
    }

    fn sample(at: i64, window: Option<Window>) -> Sample {
        Sample { at, idle_ms: 0, locked: false, paused: false, window }
    }

    fn active(app: &str, title: &str) -> Activity {
        Activity { kind: Kind::Active, window: win(app, title) }
    }

    fn away() -> Activity {
        Activity { kind: Kind::Away, window: None }
    }

    fn start(start: i64, end: i64, activity: Activity) -> Op {
        Op::Start(Block { start, end, activity })
    }

    /// Feeds a sample every 2 s from `from` to `to` inclusive, with input stopping at
    /// `last_input`. Returns the ops of the last sample.
    fn feed(t: &mut Tracker, from: i64, to: i64, last_input: i64, window: Option<Window>) -> Vec<Op> {
        let mut ops = Vec::new();
        let mut at = from;
        while at <= to {
            let mut s = sample(at, window.clone());
            s.idle_ms = (at - last_input).max(0);
            ops = t.observe(&s);
            at += 2000;
        }
        ops
    }

    #[test]
    fn same_window_extends_block() {
        let mut t = tracker();
        assert_eq!(t.observe(&sample(0, win("a", "x"))), vec![start(0, 0, active("a", "x"))]);
        assert_eq!(t.observe(&sample(2000, win("a", "x"))), vec![Op::SetEnd(2000)]);
        assert_eq!(t.observe(&sample(4000, win("a", "x"))), vec![Op::SetEnd(4000)]);
    }

    #[test]
    fn window_change_starts_new_block() {
        let mut t = tracker();
        t.observe(&sample(0, win("a", "x")));
        assert_eq!(
            t.observe(&sample(2000, win("a", "y"))),
            vec![Op::SetEnd(2000), start(2000, 2000, active("a", "y"))]
        );
        assert_eq!(
            t.observe(&sample(4000, win("b", "y"))),
            vec![Op::SetEnd(4000), start(4000, 4000, active("b", "y"))]
        );
    }

    #[test]
    fn going_idle_backdates_to_last_input() {
        let mut t = tracker();
        // Input stops at 1 min; at 6 min the 5-minute threshold is reached.
        let ops = feed(&mut t, 0, 6 * MIN, MIN, win("a", "x"));
        assert_eq!(ops, vec![Op::SetEnd(MIN), start(MIN, 6 * MIN, away())]);
        // Still idle: the away block grows.
        let ops = feed(&mut t, 6 * MIN + 2000, 6 * MIN + 2000, MIN, win("a", "x"));
        assert_eq!(ops, vec![Op::SetEnd(6 * MIN + 2000)]);
        // Input again.
        assert_eq!(
            t.observe(&sample(6 * MIN + 4000, win("a", "x"))),
            vec![Op::SetEnd(6 * MIN + 4000), start(6 * MIN + 4000, 6 * MIN + 4000, active("a", "x"))]
        );
    }

    #[test]
    fn backdating_never_goes_before_open_block() {
        let mut t = tracker();
        // Input stops at 1 min, then a window appears on its own at 4 min.
        feed(&mut t, 0, 4 * MIN - 2000, MIN, win("a", "x"));
        let ops = feed(&mut t, 4 * MIN, 6 * MIN, MIN, win("b", "y"));
        // The open block becomes empty (the store deletes it) and away starts where it began.
        assert_eq!(ops, vec![Op::SetEnd(4 * MIN), start(4 * MIN, 6 * MIN, away())]);
    }

    #[test]
    fn lock_is_away_immediately() {
        let mut t = tracker();
        t.observe(&sample(0, win("a", "x")));
        let mut s = sample(2000, win("a", "x"));
        s.locked = true;
        assert_eq!(t.observe(&s), vec![Op::SetEnd(2000), start(2000, 2000, away())]);
    }

    #[test]
    fn pause_wins_over_everything() {
        let mut t = tracker();
        t.observe(&sample(0, win("a", "x")));
        let mut s = sample(2000, win("a", "x"));
        s.paused = true;
        s.locked = true;
        s.idle_ms = 10 * MIN;
        let paused = Activity { kind: Kind::Paused, window: None };
        assert_eq!(t.observe(&s), vec![Op::SetEnd(2000), start(2000, 2000, paused)]);
    }

    #[test]
    fn gap_closes_block_without_stretching() {
        let mut t = tracker();
        t.observe(&sample(0, win("a", "x")));
        t.observe(&sample(2000, win("a", "x")));
        // Slept for an hour: the old block stays ending at 2000.
        assert_eq!(
            t.observe(&sample(60 * MIN, win("a", "x"))),
            vec![start(60 * MIN, 60 * MIN, active("a", "x"))]
        );
    }

    #[test]
    fn splits_at_midnight() {
        let mut t = tracker();
        t.observe(&sample(DAY - 2000, win("a", "x")));
        assert_eq!(
            t.observe(&sample(DAY + 1000, win("a", "x"))),
            vec![Op::SetEnd(DAY), start(DAY, DAY, active("a", "x")), Op::SetEnd(DAY + 1000)]
        );
    }

    #[test]
    fn midnight_split_then_window_change() {
        let mut t = tracker();
        t.observe(&sample(DAY - 2000, win("a", "x")));
        assert_eq!(
            t.observe(&sample(DAY + 1000, win("b", "y"))),
            vec![
                Op::SetEnd(DAY),
                start(DAY, DAY, active("a", "x")),
                Op::SetEnd(DAY + 1000),
                start(DAY + 1000, DAY + 1000, active("b", "y")),
            ]
        );
    }

    #[test]
    fn skipped_apps_have_no_title() {
        let mut t = tracker();
        t.set_skip_titles(vec!["a".into()]);
        assert_eq!(t.observe(&sample(0, win("a", "secret"))), vec![start(0, 0, active("a", ""))]);
        // Different titles in a skipped app don't split blocks.
        assert_eq!(t.observe(&sample(2000, win("a", "other"))), vec![Op::SetEnd(2000)]);
    }

    #[test]
    fn no_front_window_is_away() {
        let mut t = tracker();
        assert_eq!(t.observe(&sample(0, None)), vec![start(0, 0, away())]);
    }
}
