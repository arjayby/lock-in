import { useEffect, useState } from "react";
import { CaretUpDown } from "@phosphor-icons/react";
import {
  AppInfo,
  Settings,
  getSettings,
  seenApps,
  setAwayMinutes,
  setLaunchAtLogin,
  setSkipTitles,
} from "../api";
import { AppIcon } from "./AppIcon";
import { ErrorAlert } from "./ErrorAlert";

export function SettingsView() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [away, setAway] = useState("");
  const [error, setError] = useState<string | null>(null);

  const load = () =>
    Promise.all([getSettings(), seenApps()])
      .then(([s, a]) => {
        setSettings(s);
        setApps(a);
        setAway(String(s.awayMinutes));
      })
      .catch((e) => setError(String(e)));

  useEffect(() => {
    load();
  }, []);

  const run = (action: Promise<void>) =>
    action.then(() => (setError(null), load())).catch((e) => setError(String(e)));

  if (!settings) return error ? <ErrorAlert message={error} /> : null;

  const skipIds = settings.skipTitles.map((a) => a.id);
  const addable = apps.filter((a) => !skipIds.includes(a.id));

  const saveAway = () => {
    const minutes = Number(away);
    if (minutes !== settings.awayMinutes) run(setAwayMinutes(minutes));
  };

  return (
    <>
      {error && <ErrorAlert message={error} />}

      <section className="card">
        <div className="card-header">
          <h2 className="card-title">Away</h2>
          <p className="card-description">When idle time stops counting as active.</p>
        </div>
        <div className="card-body">
          <label className="setting-row">
            <span>Count me as away after</span>
            <input
              className="input"
              type="number"
              min={1}
              max={120}
              value={away}
              onChange={(e) => setAway(e.target.value)}
              onBlur={saveAway}
              onKeyDown={(e) => e.key === "Enter" && saveAway()}
            />
            <span>minutes without keyboard or mouse input.</span>
          </label>
          <p className="hint">Applies from now on. Past days aren't recalculated.</p>
        </div>
      </section>

      <section className="card">
        <div className="card-header">
          <h2 className="card-title">Startup</h2>
        </div>
        <label className="setting-row toggle">
          <button
            className="switch"
            role="switch"
            aria-checked={settings.launchAtLogin}
            onClick={() => run(setLaunchAtLogin(!settings.launchAtLogin))}
          >
            <span className="switch-thumb" />
          </button>
          <span>Launch Lock In when I log in</span>
        </label>
      </section>

      <section className="card">
        <div className="card-header">
          <h2 className="card-title">Skip window titles</h2>
          <p className="card-description">
            Time in these apps still counts, but their window titles are never saved. Adding an app
            also deletes the titles already saved for it.
          </p>
        </div>
        <div className="card-body">
          {settings.skipTitles.length > 0 && (
            <ul className="skip-list">
              {settings.skipTitles.map((app) => (
                <li key={app.id}>
                  <AppIcon appId={app.id} name={app.name} />
                  <span className="app-name">{app.name}</span>
                  <button
                    className="button"
                    data-variant="outline"
                    data-size="xs"
                    onClick={() => run(setSkipTitles(skipIds.filter((id) => id !== app.id)))}
                  >
                    Remove
                  </button>
                </li>
              ))}
            </ul>
          )}
          <div className="select">
            <select
              className="input"
              aria-label="Add an app"
              value=""
              disabled={addable.length === 0}
              onChange={(e) => e.target.value && run(setSkipTitles([...skipIds, e.target.value]))}
            >
              <option value="">Add an app you've used…</option>
              {addable.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
            <CaretUpDown size={16} />
          </div>
        </div>
      </section>
    </>
  );
}
