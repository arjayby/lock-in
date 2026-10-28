import { appColor } from "../format";

export function AppIcon({ appId, name }: { appId: string; name: string }) {
  return (
    <span className="app-icon initial" style={{ background: appColor(appId) }} aria-hidden>
      {name.charAt(0).toUpperCase()}
    </span>
  );
}
