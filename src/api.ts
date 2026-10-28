import { invoke } from "@tauri-apps/api/core";

export type Kind = "active" | "away" | "paused";

export interface BlockRow {
  start: number;
  end: number;
  kind: Kind;
  appId: string | null;
  appName: string | null;
  title: string | null;
}

export interface AppTotal {
  appId: string;
  name: string;
  ms: number;
}

export interface TitleTotal {
  title: string;
  ms: number;
}

export interface DaySummary {
  from: number;
  to: number;
  activeMs: number;
  apps: AppTotal[];
  blocks: BlockRow[];
}

export const daySummary = (date: string) => invoke<DaySummary>("day_summary", { date });

export const appTitles = (date: string, appId: string) =>
  invoke<TitleTotal[]>("app_titles", { date, appId });

export interface AppInfo {
  id: string;
  name: string;
  path: string | null;
}

export interface Settings {
  awayMinutes: number;
  skipTitles: { id: string; name: string }[];
  launchAtLogin: boolean;
}

export const getSettings = () => invoke<Settings>("get_settings");
export const seenApps = () => invoke<AppInfo[]>("seen_apps");
export const setAwayMinutes = (minutes: number) => invoke<void>("set_away_minutes", { minutes });
export const setSkipTitles = (appIds: string[]) => invoke<void>("set_skip_titles", { appIds });
export const setLaunchAtLogin = (enabled: boolean) => invoke<void>("set_launch_at_login", { enabled });

/** Writes a CSV of blocks from the start of `from` to the end of `to`. Resolves to the row count. */
export const exportCsv = (from: string, to: string, path: string) =>
  invoke<number>("export_csv", { from, to, path });
