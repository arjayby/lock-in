// Sample data for previewing the dashboard in a plain browser (`pnpm dev`), outside Tauri.
import { mockIPC } from "@tauri-apps/api/mocks";
import type { BlockRow, DaySummary } from "./api";

const MIN = 60_000;
const APPS: [string, string, string[]][] = [
  ["com.microsoft.VSCode", "Visual Studio Code", ["tracker.rs — lock-in", "App.tsx — lock-in", "styles.css — lock-in"]],
  ["com.google.Chrome", "Google Chrome", ["Tauri v2 docs", "rusqlite - Rust", "GitHub"]],
  ["com.tinyspeck.slackmacgap", "Slack", ["#general", "DM with Sam"]],
  ["com.apple.Terminal", "Terminal", ["zsh", "cargo test"]],
  ["com.1password.1password", "1Password", [""]],
];

function sampleDay(date: string): DaySummary {
  const [y, m, d] = date.split("-").map(Number);
  const from = new Date(y, m - 1, d).getTime();
  const blocks: BlockRow[] = [];
  let seed = d * 7919;
  const rand = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  let t = from + 9 * 60 * MIN + Math.floor(rand() * 40) * MIN;
  const end = Math.min(from + 18 * 60 * MIN, Date.now());
  while (t < end) {
    const len = Math.floor(2 + rand() * 30) * MIN;
    const r = rand();
    if (r < 0.08) {
      blocks.push({ start: t, end: t + len, kind: "away", appId: null, appName: null, title: null });
    } else if (r < 0.1) {
      blocks.push({ start: t, end: t + len, kind: "paused", appId: null, appName: null, title: null });
    } else {
      const [appId, appName, titles] = APPS[Math.floor(rand() * rand() * APPS.length)];
      const title = titles[Math.floor(rand() * titles.length)];
      blocks.push({ start: t, end: t + len, kind: "active", appId, appName, title });
    }
    t += len;
  }

  const totals = new Map<string, { appId: string; name: string; ms: number }>();
  let activeMs = 0;
  for (const b of blocks) {
    if (b.kind !== "active" || !b.appId) continue;
    activeMs += b.end - b.start;
    const cur = totals.get(b.appId) ?? { appId: b.appId, name: b.appName!, ms: 0 };
    cur.ms += b.end - b.start;
    totals.set(b.appId, cur);
  }
  return { from, to: from + 24 * 60 * MIN, activeMs, apps: [...totals.values()].sort((a, b) => b.ms - a.ms), blocks };
}

const settings = {
  awayMinutes: 5,
  skipTitles: [{ id: "com.1password.1password", name: "1Password" }],
  launchAtLogin: true,
};

mockIPC((cmd, args) => {
  const a = args as Record<string, string>;
  switch (cmd) {
    case "get_settings":
      return settings;
    case "seen_apps":
      return APPS.map(([id, name]) => ({ id, name, path: null }));
    case "set_away_minutes":
      settings.awayMinutes = Number(a.minutes);
      return;
    case "set_launch_at_login":
      settings.launchAtLogin = Boolean(a.enabled);
      return;
    case "set_skip_titles": {
      const ids = a.appIds as unknown as string[];
      settings.skipTitles = ids.map((id) => ({ id, name: APPS.find(([i]) => i === id)?.[1] ?? id }));
      return;
    }
    case "day_summary":
      return sampleDay(a.date);
    case "app_titles": {
      const ms = new Map<string, number>();
      for (const b of sampleDay(a.date).blocks) {
        if (b.appId === a.appId) ms.set(b.title ?? "", (ms.get(b.title ?? "") ?? 0) + b.end - b.start);
      }
      return [...ms].map(([title, ms]) => ({ title, ms })).sort((x, y) => y.ms - x.ms);
    }
  }
});
