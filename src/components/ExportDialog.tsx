import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { exportCsv } from "../api";
import { today } from "../format";

interface Props {
  date: string;
  onClose: () => void;
}

export function ExportDialog({ date, onClose }: Props) {
  const [from, setFrom] = useState(date);
  const [to, setTo] = useState(date);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    setError(null);
    const name = from === to ? `lock-in-${from}.csv` : `lock-in-${from}-to-${to}.csv`;
    const path = await save({ defaultPath: name, filters: [{ name: "CSV", extensions: ["csv"] }] });
    if (!path) return;
    setBusy(true);
    try {
      await exportCsv(from, to, path);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="backdrop" onClick={onClose}>
      <div
        className="dialog"
        role="dialog"
        aria-labelledby="export-title"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.key === "Escape" && onClose()}
      >
        <h2 id="export-title">Export to CSV</h2>
        <p className="hint">Every recorded block between these dates, including away and paused time.</p>
        <div className="date-range">
          <label>
            From
            <input type="date" value={from} max={to} onChange={(e) => setFrom(e.target.value)} />
          </label>
          <label>
            To
            <input type="date" value={to} min={from} max={today()} onChange={(e) => setTo(e.target.value)} />
          </label>
        </div>
        {error && <p className="error">{error}</p>}
        <div className="dialog-actions">
          <button className="button" onClick={onClose}>
            Cancel
          </button>
          <button className="button primary" disabled={busy || !from || !to} onClick={submit}>
            Export…
          </button>
        </div>
      </div>
    </div>
  );
}
