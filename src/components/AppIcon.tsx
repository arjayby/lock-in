import { useEffect, useState } from "react";
import { appIcon } from "../api";
import { appColor } from "../format";

// One request per app for the life of the window.
const cache = new Map<string, Promise<string | null>>();

function loadIcon(appId: string): Promise<string | null> {
  let icon = cache.get(appId);
  if (!icon) {
    icon = appIcon(appId).catch(() => null);
    cache.set(appId, icon);
  }
  return icon;
}

export function AppIcon({ appId, name }: { appId: string; name: string }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    loadIcon(appId).then((icon) => live && setSrc(icon));
    return () => {
      live = false;
    };
  }, [appId]);

  if (src) return <img className="app-icon" src={src} alt="" />;
  return (
    <span className="app-icon initial" style={{ background: appColor(appId) }} aria-hidden>
      {name.charAt(0).toUpperCase()}
    </span>
  );
}
