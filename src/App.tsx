import { useCallback, useEffect, useState } from "react";
import { DaySummary, daySummary } from "./api";
import { addDays, formatDuration, longDate, today } from "./format";
import { AppList } from "./components/AppList";
import { Timeline } from "./components/Timeline";
import "./styles.css";

const REFRESH_MS = 10_000;

export default function App() {
  const [date, setDate] = useState(today);
  const [summary, setSummary] = useState<DaySummary | null>(null);
  const [version, setVersion] = useState(0);
  const [error, setError] = useState<string | null>(null);

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

  return (
    <div className="app">
      <header className="header">
        <div className="date-nav">
          <button className="icon-button" aria-label="Previous day" onClick={() => setDate(addDays(date, -1))}>
            ‹
          </button>
          <input
            type="date"
            value={date}
            max={today()}
            onChange={(e) => e.target.value && setDate(e.target.value)}
          />
          <button
            className="icon-button"
            aria-label="Next day"
            disabled={isToday}
            onClick={() => setDate(addDays(date, 1))}
          >
            ›
          </button>
          {!isToday && (
            <button className="button" onClick={() => setDate(today())}>
              Today
            </button>
          )}
        </div>
      </header>

      <main className="content">
        {error && <p className="error">{error}</p>}

        <section className="summary">
          <h1>{isToday ? "Today" : longDate(date)}</h1>
          <p className="total">
            {summary ? formatDuration(summary.activeMs) : "–"}
            <span className="muted"> active</span>
          </p>
        </section>

        <section className="card">
          <h2>Timeline</h2>
          {summary && <Timeline from={summary.from} blocks={summary.blocks} />}
        </section>

        <section className="card">
          <h2>Apps</h2>
          {summary && <AppList date={date} apps={summary.apps} version={version} />}
        </section>
      </main>
    </div>
  );
}
