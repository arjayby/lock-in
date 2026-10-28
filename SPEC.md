# Lock In — v1 spec

A personal, macOS-only, menu-bar activity tracker (an ActivityWatch alternative) built with Tauri 2.

## Scope

- Passive tracking + dashboard. Focus features (nudges, limits, blocking) are v2; the tracker loop is
  structured so rules can hook in later.
- macOS only. Single user, local builds signed with a self-signed certificate so permissions persist.
- No categories in v1. Raw history is kept so categories can be applied retroactively later.

## Tracking

- Poll the frontmost window every **2 s**. Record app (grouped by bundle ID, shown by display name)
  and window title. **No URLs.**
- **Away** after **5 min** without keyboard/mouse input (configurable). When away is detected, the
  previous block is trimmed back to the last input, so idle minutes count as away.
- A **locked screen** counts as away immediately. **Sleep** leaves a gap, shown as away.
- **Pause**: 30 min / 1 hour / until resumed. Paused time is stored as a "Paused" block. Pause state
  survives quit/restart.
- Consecutive identical samples extend the current block instead of creating rows.
- Days run midnight to midnight, local time; blocks are split at midnight.
- **Skip titles** list: apps whose titles are never stored (defaults: 1Password, Keychain Access,
  System Settings). Adding an app deletes its already-stored titles. Removing it resumes storing
  titles from then on.
- Changing the away threshold applies going forward only.

## Storage

- One SQLite file in `~/Library/Application Support/com.arj.lockin/`. Local only, kept forever.
- Rust owns tracking, writes and aggregation; the frontend only calls typed commands.

## App shell

- Menu bar only (no Dock icon). Monochrome template icon + today's active time as text (`4h 12m`),
  updated every minute. Active = not away and not paused.
- Menu: today's total, Open dashboard, Pause tracking (30 min / 1 hour / until I resume) or Resume, Quit.
- Closing the dashboard keeps tracking running. Launch at login is enabled on first run.

## Dashboard

React + TypeScript + Vite (pnpm). Follows system light/dark. Opens on today; refreshes every 10 s.

- Date picker.
- **Per-app totals**: bar list with real app icons; clicking an app shows its top window titles.
- **Day timeline**: what was frontmost when; away greyed out; hover tooltip with app, title,
  start–end and duration.
- **Export** button in the header: from/to dates (default: viewed day) → CSV of raw blocks
  (start, end, duration_s, kind, app_name, app_id, title) via the native save dialog.
- **Settings** view: skip-titles list (pick from seen apps), away-after-N-minutes, launch at login.

## Testing

Rust unit tests for block merging, idle backdating, lock, pause and midnight splitting. No UI tests.
