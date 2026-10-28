import type { AppTotal } from "./api";

/** `4h 12m`, `12m`, or `<1m`. */
export function formatDuration(ms: number): string {
  const minutes = Math.floor(ms / 60_000);
  if (minutes < 1) return "<1m";
  if (minutes < 60) return `${minutes}m`;
  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
}

export function formatClock(ms: number): string {
  return new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

/** Local `YYYY-MM-DD`. */
export function isoDate(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function today(): string {
  return isoDate(new Date());
}

export function addDays(date: string, days: number): string {
  const [y, m, d] = date.split("-").map(Number);
  return isoDate(new Date(y, m - 1, d + days));
}

export function longDate(date: string): string {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString([], {
    weekday: "long",
    month: "long",
    day: "numeric",
  });
}

const CHART_SLOTS = 5;

/** Chart colours by rank: the day's top five apps keep their own slot, the rest share Other. */
export function appColors(apps: AppTotal[]): (appId: string) => string {
  const slots = new Map(apps.slice(0, CHART_SLOTS).map((a, i) => [a.appId, `var(--chart-${i + 1})`]));
  return (appId) => slots.get(appId) ?? "var(--chart-other)";
}
