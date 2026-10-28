import { useEffect, useState } from "react";
import { CircleNotch, DownloadSimple, X } from "@phosphor-icons/react";
import { save } from "@tauri-apps/plugin-dialog";
import { exportCsv } from "../api";
import { today } from "../format";
import { ErrorAlert } from "./ErrorAlert";

interface Props {
  date: string;
  onClose: () => void;
}

export function ExportDialog({ date, onClose }: Props) {
  const [from, setFrom] = useState(date);
  const [to, setTo] = useState(date);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

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
    <>
      <div className="overlay" onClick={onClose} />
      <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="export-title">
        <div className="dialog-header">
          <h2 id="export-title" className="dialog-title">
            Export to CSV
          </h2>
          <p className="dialog-description">
            Every recorded block between these dates, including away and paused time.
          </p>
        </div>
        <div className="date-range">
          <label className="field">
            From
            <input
              className="input"
              type="date"
              value={from}
              max={to}
              onChange={(e) => setFrom(e.target.value)}
            />
          </label>
          <label className="field">
            To
            <input
              className="input"
              type="date"
              value={to}
              min={from}
              max={today()}
              onChange={(e) => setTo(e.target.value)}
            />
          </label>
        </div>
        {error && <ErrorAlert message={error} />}
        <div className="dialog-footer">
          <button className="button" data-variant="outline" onClick={onClose}>
            Cancel
          </button>
          <button className="button" disabled={busy || !from || !to} onClick={submit}>
            {busy ? <CircleNotch size={16} className="spin" /> : <DownloadSimple size={16} />}
            {busy ? "Exporting" : "Export"}
          </button>
        </div>
        <button
          className="button dialog-close"
          data-variant="ghost"
          data-size="icon-sm"
          aria-label="Close"
          title="Close"
          onClick={onClose}
        >
          <X size={16} />
        </button>
      </div>
    </>
  );
}
