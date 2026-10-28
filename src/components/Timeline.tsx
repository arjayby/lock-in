import { useState } from "react";
import { BlockRow } from "../api";
import { formatClock, formatDuration } from "../format";

const HOUR = 3_600_000;

interface Props {
  from: number;
  blocks: BlockRow[];
  color: (appId: string) => string;
}

interface Hover {
  block: BlockRow;
  /** Pointer position as a fraction of the track width. */
  x: number;
}

export function Timeline({ from, blocks, color }: Props) {
  const [hover, setHover] = useState<Hover | null>(null);

  if (blocks.length === 0) return <p className="empty">Nothing recorded on this day.</p>;

  // Show whole hours around the recorded blocks.
  const first = Math.floor((blocks[0].start - from) / HOUR);
  const last = Math.max(first + 1, Math.ceil((blocks[blocks.length - 1].end - from) / HOUR));
  const start = from + first * HOUR;
  const span = (last - first) * HOUR;
  const pct = (t: number) => ((t - start) / span) * 100;
  const hours = Array.from({ length: last - first + 1 }, (_, i) => first + i);
  const labelEvery = Math.ceil(hours.length / 12);

  return (
    <div className="timeline">
      <div
        className="track"
        onMouseLeave={() => setHover(null)}
        onMouseMove={(e) => {
          const rect = e.currentTarget.getBoundingClientRect();
          const x = (e.clientX - rect.left) / rect.width;
          const t = start + x * span;
          const block = blocks.find((b) => b.start <= t && t < b.end);
          setHover(block ? { block, x } : null);
        }}
      >
        {blocks.map((b) => (
          <span
            key={b.start}
            className={`segment ${b.kind}`}
            style={{
              left: `${pct(b.start)}%`,
              width: `${pct(b.end) - pct(b.start)}%`,
              background: b.kind === "active" && b.appId ? color(b.appId) : undefined,
            }}
          />
        ))}
        {hover && <Readout {...hover} />}
      </div>
      <div className="hours">
        {hours.map((h, i) =>
          i % labelEvery === 0 ? (
            <span key={h} style={{ left: `${(i / (hours.length - 1)) * 100}%` }}>
              {formatClock(from + h * HOUR)}
            </span>
          ) : null,
        )}
      </div>
    </div>
  );
}

function Readout({ block, x }: Hover) {
  const label =
    block.kind === "active" ? (block.appName ?? block.appId) : block.kind === "away" ? "Away" : "Paused";
  return (
    <div className={`readout ${x > 0.6 ? "left" : ""}`} style={{ left: `${x * 100}%` }}>
      <strong>{label}</strong>
      {block.kind === "active" && block.title && <span className="readout-title">{block.title}</span>}
      <span className="readout-time">
        {formatClock(block.start)} – {formatClock(block.end)} · {formatDuration(block.end - block.start)}
      </span>
    </div>
  );
}
