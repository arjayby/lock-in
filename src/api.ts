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
