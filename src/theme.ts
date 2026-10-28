// Appearance preference, kept in localStorage. "system" follows macOS; the resolved theme lands on
// <html data-theme> for tokens.css, and the native window chrome is matched when running in Tauri.
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type ThemePref = "system" | "light" | "dark";

const KEY = "theme";
const systemLight = window.matchMedia("(prefers-color-scheme: light)");

export function getThemePref(): ThemePref {
  try {
    const pref = localStorage.getItem(KEY);
    if (pref === "light" || pref === "dark") return pref;
  } catch {}
  return "system";
}

export function setThemePref(pref: ThemePref) {
  try {
    if (pref === "system") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, pref);
  } catch {}
  apply();
}

export function initTheme() {
  apply();
  systemLight.addEventListener("change", apply);
}

function apply() {
  const pref = getThemePref();
  const light = pref === "light" || (pref === "system" && systemLight.matches);
  document.documentElement.dataset.theme = light ? "light" : "dark";
  if (isTauri()) getCurrentWindow().setTheme(pref === "system" ? null : pref).catch(() => {});
}
