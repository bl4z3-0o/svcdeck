// Turns the HyDE/wallbash palette (or a built-in fallback) into CSS variables.
//
// The order of the accent shades (1xa1..1xa9) flips between light and dark
// palettes. Instead of a fixed shade we pick the one with a suitable contrast
// against the background, which stays readable with any theme.
import type { Theme, ThemePref } from "./types";

type RGB = [number, number, number];

function hexToRgb(hex: string): RGB {
  const h = hex.replace("#", "");
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)) as RGB;
}

function luminance([r, g, b]: RGB): number {
  const f = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

function contrast(a: RGB, b: RGB): number {
  const [l1, l2] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (l1 + 0.05) / (l2 + 0.05);
}

const rgbStr = (c: RGB) => c.join(" ");

/** Accent of group g (1..4) whose contrast is closest to the target */
function pickAccent(colors: Record<string, string>, g: number, bg: RGB, target: number): RGB | null {
  let best: RGB | null = null;
  let bestDiff = Infinity;
  for (let i = 1; i <= 9; i++) {
    const hex = colors[`${g}xa${i}`];
    if (!hex) continue;
    const c = hexToRgb(hex);
    const diff = Math.abs(contrast(c, bg) - target);
    if (diff < bestDiff) {
      bestDiff = diff;
      best = c;
    }
  }
  return best;
}

// Fallback without HyDE: neutral palette in the style of macOS
const FALLBACK = {
  dark: { bg: [28, 28, 32], fg: [245, 245, 247], accent: [10, 132, 255], accent2: [191, 90, 242] },
  light: { bg: [246, 246, 248], fg: [29, 29, 31], accent: [0, 113, 227], accent2: [175, 82, 222] },
} satisfies Record<string, Record<string, RGB>>;

const darkQuery = typeof window === "undefined" ? null : window.matchMedia("(prefers-color-scheme: dark)");

/**
 * pref "auto": HyDE colors when available, otherwise the system light/dark
 * preference. "light"/"dark": built-in palette in that mode.
 */
export function applyTheme(theme: Theme | null, pref: ThemePref = "auto") {
  const root = document.documentElement;
  let mode: "dark" | "light";
  let bg: RGB, fg: RGB, accent: RGB, accent2: RGB;

  const useHyde = pref === "auto" && theme?.available && theme.colors.pry1 && theme.colors.txt1;
  if (useHyde) {
    bg = hexToRgb(theme!.colors.pry1);
    fg = hexToRgb(theme!.colors.txt1);
    // Derive the mode from the real brightness, not only dcol_mode
    mode = luminance(bg) > 0.35 ? "light" : "dark";
    accent = pickAccent(theme!.colors, 1, bg, 4) ?? FALLBACK[mode].accent;
    accent2 = pickAccent(theme!.colors, 3, bg, 3.2) ?? FALLBACK[mode].accent2;
  } else {
    mode = pref === "light" || pref === "dark" ? pref : darkQuery?.matches === false ? "light" : "dark";
    ({ bg, fg, accent, accent2 } = FALLBACK[mode]);
  }

  // Text on accent surfaces: black or white, whichever reads better
  const onAccent: RGB =
    contrast(accent, [255, 255, 255]) >= contrast(accent, [0, 0, 0]) ? [255, 255, 255] : [0, 0, 0];

  root.dataset.mode = mode;
  root.style.setProperty("--bg", rgbStr(bg));
  root.style.setProperty("--fg", rgbStr(fg));
  root.style.setProperty("--accent", rgbStr(accent));
  root.style.setProperty("--accent-2", rgbStr(accent2));
  root.style.setProperty("--on-accent", rgbStr(onAccent));
}

/** Calls fn when the system switches between light and dark */
export function onSystemSchemeChange(fn: () => void): () => void {
  darkQuery?.addEventListener("change", fn);
  return () => darkQuery?.removeEventListener("change", fn);
}
