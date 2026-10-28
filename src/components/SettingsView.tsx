import { useEffect, useState } from "react";
import {
  AppInfo,
  Settings,
  getSettings,
  seenApps,
  setAwayMinutes,
  setLaunchAtLogin,
  setSkipTitles,
} from "../api";

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

  if (!settings) return error ? <p className="error">{error}</p> : null;

  const skipIds = settings.skipTitles.map((a) => a.id);
  const addable = apps.filter((a) => !skipIds.includes(a.id));

  const saveAway = () => {
    const minutes = Number(away);
    if (minutes !== settings.awayMinutes) run(setAwayMinutes(minutes));
  };

  return (
    <>
      {error && <p className="error">{error}</p>}

      <section className="card setting">
        <h2>Away</h2>
        <label className="setting-row">
          <span>Count me as away after</span>
          <input
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
      </section>

      <section className="card setting">
        <h2>Startup</h2>
        <label className="setting-row">
          <input
            type="checkbox"
            checked={settings.launchAtLogin}
            onChange={(e) => run(setLaunchAtLogin(e.target.checked))}
          />
          <span>Launch Lock In when I log in</span>
        </label>
      </section>

      <section className="card setting">
        <h2>Skip window titles</h2>
        <p className="hint">
          Time in these apps still counts, but their window titles are never saved. Adding an app
          also deletes the titles already saved for it.
        </p>
        <ul className="skip-list">
          {settings.skipTitles.map((app) => (
            <li key={app.id}>
              <span className="app-name">{app.name}</span>
              <button
                className="button"
                onClick={() => run(setSkipTitles(skipIds.filter((id) => id !== app.id)))}
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
        <select
          className="select"
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
      </section>
    </>
  );
}
