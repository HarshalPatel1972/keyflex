import type { Theme } from "./api";

const STORAGE_KEY = "keyflex-theme";
const systemLight = window.matchMedia("(prefers-color-scheme: light)");
let current: Theme = "system";

function paint() {
  const light = current === "light" || (current === "system" && systemLight.matches);
  document.documentElement.dataset.theme = light ? "light" : "dark";
}

// Follow Windows when it switches between light and dark.
systemLight.addEventListener("change", paint);

/** The theme chosen last time, known before the settings have loaded. */
export function rememberedTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "light" || stored === "dark" ? stored : "system";
}

export function applyTheme(theme: Theme) {
  current = theme;
  localStorage.setItem(STORAGE_KEY, theme);
  paint();
}
