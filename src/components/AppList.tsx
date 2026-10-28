import { useEffect, useState } from "react";
import { CaretRight } from "@phosphor-icons/react";
import { AppTotal, TitleTotal, appTitles } from "../api";
import { formatDuration } from "../format";
import { AppIcon } from "./AppIcon";

interface Props {
  date: string;
  apps: AppTotal[];
  /** Changes on every refresh, so open title lists refresh too. */
  version: number;
  color: (appId: string) => string;
}

export function AppList({ date, apps, version, color }: Props) {
  const [openApp, setOpenApp] = useState<string | null>(null);

  if (apps.length === 0) return <p className="empty">No app activity on this day.</p>;

  const max = apps[0].ms;
  return (
    <ul className="app-list">
      {apps.map((app) => (
        <li key={app.appId}>
          <button
            className="app-row"
            aria-expanded={openApp === app.appId}
            onClick={() => setOpenApp(openApp === app.appId ? null : app.appId)}
          >
            <AppIcon appId={app.appId} name={app.name} />
            <span className="app-name">{app.name}</span>
            <span className="app-bar">
              <span
                style={{ width: `${(app.ms / max) * 100}%`, background: color(app.appId) }}
              />
            </span>
            <span className="app-time">{formatDuration(app.ms)}</span>
            <CaretRight size={16} />
          </button>
          {openApp === app.appId && <Titles date={date} appId={app.appId} version={version} />}
        </li>
      ))}
    </ul>
  );
}

function Titles({ date, appId, version }: { date: string; appId: string; version: number }) {
  const [titles, setTitles] = useState<TitleTotal[] | null>(null);

  useEffect(() => {
    appTitles(date, appId).then(setTitles).catch(console.error);
  }, [date, appId, version]);

  if (!titles) return null;
  return (
    <ul className="titles">
      {titles.map((t) => (
        <li key={t.title}>
          <span className={t.title ? "title" : "title untitled"}>{t.title || "No title"}</span>
          <span className="app-time">{formatDuration(t.ms)}</span>
        </li>
      ))}
    </ul>
  );
}
