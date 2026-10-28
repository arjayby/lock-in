import { useCallback, useEffect, useState } from "react";
import { CalendarBlank, CaretLeft, CaretRight, DownloadSimple, Gear } from "@phosphor-icons/react";
import { DaySummary, daySummary } from "./api";
import { addDays, appColors, formatDuration, longDate, today } from "./format";
import { AppList } from "./components/AppList";
import { ErrorAlert } from "./components/ErrorAlert";
import { ExportDialog } from "./components/ExportDialog";
import { SettingsView } from "./components/SettingsView";
import { Timeline } from "./components/Timeline";
import "./styles.css";

const REFRESH_MS = 10_000;

export default function App() {
  const [view, setView] = useState<"day" | "settings">("day");
  const [date, setDate] = useState(today);
  const [summary, setSummary] = useState<DaySummary | null>(null);
  const [version, setVersion] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  const load = useCallback(() => {
    daySummary(date)
      .then((s) => {
        setSummary(s);
        setVersion((v) => v + 1);
        setError(null);
      })
      .catch((e) => setError(String(e)));
  }, [date]);

  useEffect(() => {
    load();
    const timer = setInterval(load, REFRESH_MS);
    window.addEventListener("focus", load);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", load);
    };
  }, [load]);

  const isToday = date === today();
  const color = appColors(summary?.apps ?? []);
  const awayMs = (summary?.blocks ?? [])
    .filter((b) => b.kind === "away")
    .reduce((ms, b) => ms + b.end - b.start, 0);

  return (
    <div className="app">
      <header className="toolbar">
        {view === "settings" ? (
          <h1 className="toolbar-title">Settings</h1>
        ) : (
          <div className="date-nav">
            <button
              className="button"
              data-variant="outline"
              data-size="icon-sm"
              aria-label="Previous day"
              title="Previous day"
              onClick={() => setDate(addDays(date, -1))}
            >
              <CaretLeft size={16} />
            </button>
            <label className="date-field">
              <CalendarBlank size={16} />
              <input
                className="input"
                type="date"
                aria-label="Day"
                value={date}
                max={today()}
                onChange={(e) => e.target.value && setDate(e.target.value)}
              />
            </label>
            <button
              className="button"
              data-variant="outline"
              data-size="icon-sm"
              aria-label="Next day"
              title="Next day"
              disabled={isToday}
              onClick={() => setDate(addDays(date, 1))}
            >
              <CaretRight size={16} />
            </button>
            {!isToday && (
              <button className="button" data-variant="ghost" data-size="sm" onClick={() => setDate(today())}>
                Today
              </button>
            )}
          </div>
        )}
        <div className="toolbar-actions">
          {view === "settings" ? (
            <button className="button" data-size="sm" onClick={() => setView("day")}>
              Done
            </button>
          ) : (
            <>
              <button
                className="button"
                data-variant="outline"
                data-size="sm"
                onClick={() => setExporting(true)}
              >
                <DownloadSimple size={16} />
                Export CSV
              </button>
              <button
                className="button"
                data-variant="ghost"
                data-size="icon-sm"
                aria-label="Settings"
                title="Settings"
                onClick={() => setView("settings")}
              >
                <Gear size={16} />
              </button>
            </>
          )}
        </div>
      </header>

      {view === "settings" ? (
        <main className="content">
          <SettingsView />
        </main>
      ) : (
        <main className="content">
          {error && <ErrorAlert message={error} />}

          <section className="page-head">
            {isToday && <p className="eyebrow">{longDate(date)}</p>}
            <h1 className="page-title">{isToday ? "Today" : longDate(date)}</h1>
          </section>

          <section className="kpis">
            <Kpi label="Active" value={summary ? formatDuration(summary.activeMs) : "–"} />
            <Kpi label="Away" value={summary ? formatDuration(awayMs) : "–"} />
            <Kpi label="Apps used" value={summary ? String(summary.apps.length) : "–"} />
          </section>

          <section className="card">
            <div className="card-header">
              <h2 className="card-title">Timeline</h2>
              <p className="card-description">Hover a block to see the app and window.</p>
              <ul className="legend card-action">
                <li>
                  <span className="swatch away" />
                  Away
                </li>
                <li>
                  <span className="swatch paused" />
                  Paused
                </li>
              </ul>
            </div>
            {summary && <Timeline from={summary.from} blocks={summary.blocks} color={color} />}
          </section>

          <section className="card">
            <div className="card-header">
              <h2 className="card-title">Apps</h2>
              <p className="card-description">Select an app to see time per window title.</p>
            </div>
            {summary && <AppList date={date} apps={summary.apps} version={version} color={color} />}
          </section>
        </main>
      )}
      {exporting && <ExportDialog date={date} onClose={() => setExporting(false)} />}
    </div>
  );
}

function Kpi({ label, value }: { label: string; value: string }) {
  return (
    <div className="card" data-size="sm">
      <p className="eyebrow">{label}</p>
      <p className="metric">{value}</p>
    </div>
  );
}
