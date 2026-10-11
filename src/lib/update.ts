// Update check: asks GitHub for the latest release. Notify only; installing
// is left to the package manager (AUR) or install.sh.

export const REPO = "bl4z3-0o/svcdeck";
const LATEST = `https://api.github.com/repos/${REPO}/releases/latest`;
const DISMISSED_KEY = "svcdeck.dismissedUpdate";

export interface Update {
  version: string;
  url: string;
}

/** "v1.2.3" / "1.2.3-rc.1" -> [1, 2, 3]; null when not a version */
export function parseVersion(v: string): [number, number, number] | null {
  const m = /^v?(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$/.exec(v.trim());
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

/** true when `latest` is newer than `current` */
export function isNewer(latest: string, current: string): boolean {
  const a = parseVersion(latest);
  const b = parseVersion(current);
  if (!a || !b) return false;
  for (let i = 0; i < 3; i++) {
    if (a[i] !== b[i]) return a[i] > b[i];
  }
  return false;
}

/** Returns the newer release, or null (also on any network error). */
export async function checkForUpdate(current: string, fetcher: typeof fetch = fetch): Promise<Update | null> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), 8000);
  try {
    const res = await fetcher(LATEST, {
      headers: { Accept: "application/vnd.github+json" },
      signal: ctrl.signal,
    });
    if (!res.ok) return null;
    const data = (await res.json()) as { tag_name?: string; html_url?: string; draft?: boolean; prerelease?: boolean };
    if (!data.tag_name || data.draft || data.prerelease) return null;
    if (!isNewer(data.tag_name, current)) return null;
    return { version: data.tag_name.replace(/^v/, ""), url: data.html_url ?? `https://github.com/${REPO}/releases` };
  } catch {
    return null;
  } finally {
    clearTimeout(timer);
  }
}

export function isDismissed(version: string): boolean {
  try {
    return localStorage.getItem(DISMISSED_KEY) === version;
  } catch {
    return false;
  }
}

export function dismiss(version: string) {
  try {
    localStorage.setItem(DISMISSED_KEY, version);
  } catch {
    // storage unavailable: the banner just shows again next start
  }
}
