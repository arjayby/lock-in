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

/** A stable color per app, so the list and the timeline match. */
export function appColor(appId: string): string {
  let hash = 0;
  for (const ch of appId) hash = (hash * 31 + ch.charCodeAt(0)) | 0;
  return `hsl(${Math.abs(hash) % 360} 62% 56%)`;
}
