<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { api } from "$lib/api";
  import { applyTheme, onSystemSchemeChange } from "$lib/theme";
  import { setLanguage, t } from "$lib/i18n.svelte";
  import type { Action, App, AppInfo, Config, Theme } from "$lib/types";
  import { checkForUpdate, dismiss, isDismissed, type Update } from "$lib/update";
  import type { MenuItem, SheetEntry } from "$lib/ui";
  import Tile from "./components/Tile.svelte";
  import ContextMenu from "./components/ContextMenu.svelte";
  import OutputSheet from "./components/OutputSheet.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import Settings from "./components/Settings.svelte";

  // Grid metrics in logical pixels; the window follows the number of apps
  const CELL_W = 136;
  const CELL_H = 132;
  const PAD = 22;
  const HEADER = 50;
  const SHEET_SIZE = { w: 600, h: 520 };
  const SETTINGS_SIZE = { w: 760, h: 640 };

  interface MenuState {
    x: number;
    y: number;
    app: App | null; // null = background (all apps)
  }

  interface ConfirmState {
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
  }

  interface SheetState {
    title: string;
    entries: SheetEntry[];
  }

  let config = $state<Config | null>(null);
  let error = $state<string | null>(null);
  let health = $state<Record<string, boolean | null>>({});
  let hydeTheme = $state<Theme | null>(null);
  let icons = $state<Record<string, string>>({});
  let busy = $state<Record<string, boolean>>({});
  let menu = $state<MenuState | null>(null);
  let sheet = $state<SheetState | null>(null);
  let confirm = $state<ConfirmState | null>(null);
  let view = $state<"grid" | "settings">("grid");
  let editId = $state<string | null>(null);
  let info = $state<AppInfo | null>(null);
  let update = $state<Update | null>(null);
  /** IDs of selected apps; null = not in selection mode */
  let selection = $state<string[] | null>(null);
  const BANNER_H = 64;
  const SELBAR_H = 52;

  const cols = $derived(
    config ? Math.max(1, Math.min(config.columns || 2, Math.max(config.apps.length, 1))) : 2,
  );
  const anyBusy = $derived(Object.values(busy).some(Boolean));
  const sheetOpen = $derived(sheet !== null);
  const hasAction = (id: string) => config?.apps.some((a) => a.actions.some((x) => x.id === id)) ?? false;
  const canStart = $derived(hasAction("start"));
  const canStop = $derived(hasAction("stop"));
  const selectedApps = $derived(
    selection && config ? config.apps.filter((a) => selection!.includes(a.id)) : [],
  );
  const selCanStart = $derived(selectedApps.some((a) => a.actions.some((x) => x.id === "start")));
  const selCanStop = $derived(selectedApps.some((a) => a.actions.some((x) => x.id === "stop")));

  function gridSize(cfg: Config) {
    const c = Math.max(1, Math.min(cfg.columns || 2, Math.max(cfg.apps.length, 1)));
    const rows = Math.max(1, Math.ceil(cfg.apps.length / c));
    return { w: Math.max(320, c * CELL_W + PAD * 2), h: HEADER + rows * CELL_H + PAD };
  }

  const themePref = $derived(config?.theme ?? "auto");

  async function resize(w: number, h: number) {
    try {
      await api.fitWindow(w, h);
    } catch {
      // Ignore outside of Tauri (e.g. plain browser)
    }
  }

  // --- Theme: HyDE palette (live) or system light/dark ---------------------
  $effect(() => {
    api.getTheme().then((th) => (hydeTheme = th)).catch(() => {});
    const un = listen<Theme>("theme-changed", (e) => (hydeTheme = e.payload));
    const unScheme = onSystemSchemeChange(() => applyTheme(hydeTheme, themePref));
    return () => {
      un.then((f) => f());
      unScheme();
    };
  });

  $effect(() => applyTheme(hydeTheme, themePref));
  $effect(() => setLanguage(config?.language ?? "auto"));

  // Hover only while the pointer really is inside the window; WebKitGTK does
  // not always report leaving the window to the elements.
  $effect(() => {
    const root = document.documentElement;
    const enter = () => (root.dataset.pointer = "in");
    const leave = () => delete root.dataset.pointer;
    root.addEventListener("pointerover", enter);
    root.addEventListener("pointerleave", leave);
    window.addEventListener("blur", leave);
    return () => {
      root.removeEventListener("pointerover", enter);
      root.removeEventListener("pointerleave", leave);
      window.removeEventListener("blur", leave);
    };
  });

  // No hover color while the context menu is open (see app.css)
  $effect(() => {
    const root = document.documentElement;
    if (menu) root.dataset.menu = "open";
    else delete root.dataset.menu;
  });

  // --- Configuration ------------------------------------------------------
  async function reload() {
    try {
      config = await api.loadConfig();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    reload();
  });

  // Reload icons when the configuration changes
  $effect(() => {
    const cfg = config;
    if (!cfg) return;
    let cancelled = false;
    const specs = cfg.apps.map((a) => [a.id, a.icon.trim()] as const);
    (async () => {
      const next: Record<string, string> = {};
      await Promise.all(
        specs.map(async ([id, spec]) => {
          if (!spec) return;
          try {
            next[id] = await api.loadIcon(spec);
          } catch {
            // Not found: glyph or default icon is shown
          }
        }),
      );
      if (!cancelled) icons = next;
    })();
    return () => {
      cancelled = true;
    };
  });

  // Fit the window to the current view
  $effect(() => {
    if (!config) return;
    const g = gridSize(config);
    const extra = (update ? BANNER_H : 0) + (selection ? SELBAR_H : 0);
    if (view === "settings") resize(SETTINGS_SIZE.w, SETTINGS_SIZE.h);
    else if (sheetOpen) resize(Math.max(g.w, SHEET_SIZE.w), Math.max(g.h + extra, SHEET_SIZE.h));
    else resize(g.w, g.h + extra);
  });

  // --- Update check (once per start, notify only) -------------------------
  let updateChecked = false;
  $effect(() => {
    if (!config || updateChecked) return;
    updateChecked = true;
    const enabled = config.updateCheck;
    api
      .appInfo()
      .then(async (i) => {
        info = i;
        if (!enabled) return;
        const u = await checkForUpdate(i.version);
        if (u && !isDismissed(u.version)) update = u;
      })
      .catch(() => {});
  });

  function dismissUpdate() {
    if (update) dismiss(update.version);
    update = null;
  }

  // --- Status dots --------------------------------------------------------
  // The backend reads the saved config and runs each app's status check
  let polling = false;
  async function pollHealth() {
    if (polling) return;
    polling = true;
    try {
      const res = await api.checkHealth();
      const map: Record<string, boolean | null> = {};
      for (const h of res) map[h.id] = h.running;
      health = map;
    } catch {
      // next round tries again
    } finally {
      polling = false;
    }
  }

  $effect(() => {
    if (!config) return;
    pollHealth();
    const t = setInterval(pollHealth, 4000);
    return () => clearInterval(t);
  });

  // --- Actions ------------------------------------------------------------
  function setEntry(index: number, patch: Partial<SheetEntry>) {
    if (sheet) sheet.entries[index] = { ...sheet.entries[index], ...patch };
  }

  /** Runs (app, action) pairs one after another and shows the output */
  async function runSequence(title: string, jobs: { app: App; action: Action }[]) {
    const capture = jobs.filter((j) => j.action.mode === "capture");
    // Terminal actions only open a window, no output in SvcDeck
    for (const j of jobs.filter((j) => j.action.mode === "terminal")) {
      api.runAction(j.app.id, j.action.id).catch((e) => {
        sheet = {
          title: j.app.name,
          entries: [{ label: j.action.label, state: "fail", output: String(e) }],
        };
      });
    }
    if (capture.length === 0) return;

    sheet = {
      title,
      entries: capture.map((j) => ({
        label: capture.length > 1 ? `${j.app.name} · ${j.action.label}` : j.action.label,
        state: "pending",
        output: "",
      })),
    };

    for (let i = 0; i < capture.length; i++) {
      const { app, action } = capture[i];
      setEntry(i, { state: "running" });
      busy[app.id] = true;
      try {
        const r = await api.runAction(app.id, action.id);
        setEntry(i, {
          state: r.ok ? "ok" : "fail",
          output: r.output || (r.ok ? t("sheet.done") : t("sheet.exit", { code: r.code ?? "?" })),
        });
      } catch (e) {
        setEntry(i, { state: "fail", output: String(e) });
      } finally {
        busy[app.id] = false;
        pollHealth();
      }
    }
  }

  function withConfirm(needed: boolean, message: string, label: string, run: () => void) {
    if (needed) confirm = { message, confirmLabel: label, onConfirm: run };
    else run();
  }

  function runAppAction(app: App, action: Action) {
    withConfirm(action.confirm, t("confirm.action", { app: app.name, action: action.label }), action.label, () =>
      runSequence(`${app.name} · ${action.label}`, [{ app, action }]),
    );
  }

  /** Same action ID across all apps (e.g. "start"); stop runs in reverse */
  function runAll(actionId: string, title: string) {
    if (config) runFor(config.apps, actionId, title);
  }

  /** Same action ID across the given apps, in config order; stop runs in reverse */
  function runFor(apps: App[], actionId: string, title: string) {
    let jobs = apps.flatMap((app) => {
      const action = app.actions.find((a) => a.id === actionId);
      return action ? [{ app, action }] : [];
    });
    if (actionId === "stop") jobs = jobs.reverse();
    if (jobs.length === 0) return;
    const needsConfirm = jobs.some((j) => j.action.confirm);
    withConfirm(needsConfirm, t("confirm.all", { title, apps: jobs.map((j) => j.app.name).join(", ") }), title, () =>
      runSequence(title, jobs),
    );
  }

  function open(app: App) {
    if (app.url) openUrl(app.url).catch((e) => (error = String(e)));
  }

  // --- Selection ----------------------------------------------------------
  function toggleSelected(id: string) {
    const cur = selection ?? [];
    selection = cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id];
  }

  function toggleSelectAll() {
    if (!config || !selection) return;
    selection = selection.length === config.apps.length ? [] : config.apps.map((a) => a.id);
  }

  // Drop IDs of apps that no longer exist (e.g. after editing the config)
  $effect(() => {
    const ids = config?.apps.map((a) => a.id) ?? [];
    if (selection && selection.some((id) => !ids.includes(id))) {
      selection = selection.filter((id) => ids.includes(id));
    }
  });

  /** Left click: open the address, or run the first action if there is none.
   *  In selection mode (or with Ctrl/Shift) it toggles the selection instead. */
  function activate(app: App, e?: MouseEvent) {
    if (selection || e?.ctrlKey || e?.metaKey || e?.shiftKey) toggleSelected(app.id);
    else if (app.url) open(app);
    else if (app.actions[0]) runAppAction(app, app.actions[0]);
  }

  function openSettings(id: string | null) {
    editId = id;
    view = "settings";
  }

  // --- Context menus ------------------------------------------------------
  function actionIcon(a: Action): string {
    if (a.mode === "terminal") return "›_";
    if (a.id.includes("stop")) return "■";
    if (a.id.includes("restart")) return "↻";
    if (a.id.includes("start")) return "▶";
    if (a.id.includes("status")) return "ⓘ";
    return "•";
  }

  function menuItems(m: MenuState): MenuItem[] {
    if (m.app) {
      const app = m.app;
      const running = health[app.id];
      const header: MenuItem =
        running === null || running === undefined
          ? { kind: "header", label: app.name }
          : {
              kind: "header",
              label: app.name,
              hint: running ? t("status.running") : t("status.off"),
              tone: running ? "ok" : "off",
            };
      return [
        header,
        ...(app.url
          ? ([
              { kind: "item", label: t("menu.open"), icon: "↗", onSelect: () => open(app) },
              { kind: "separator" },
            ] satisfies MenuItem[])
          : []),
        ...app.actions.map<MenuItem>((a) => ({
          kind: "item",
          label: a.label,
          icon: actionIcon(a),
          danger: a.id === "stop" || a.confirm,
          onSelect: () => runAppAction(app, a),
        })),
        { kind: "separator" },
        { kind: "item", label: t("menu.edit"), icon: "✎", onSelect: () => openSettings(app.id) },
      ];
    }
    return [
      { kind: "header", label: config?.title ?? "SvcDeck" },
      { kind: "item", label: t("menu.startAll"), icon: "▶", onSelect: () => runAll("start", t("menu.startAll")) },
      { kind: "item", label: t("menu.stopAll"), icon: "■", danger: true, onSelect: () => runAll("stop", t("menu.stopAll")) },
      { kind: "item", label: t("menu.statusAll"), icon: "ⓘ", onSelect: () => runAll("status", t("menu.statusAll")) },
      { kind: "item", label: t("menu.select"), icon: "☑", onSelect: () => (selection = selection ?? []) },
      { kind: "separator" },
      { kind: "item", label: t("menu.settings"), icon: "⚙", onSelect: () => openSettings(null) },
      { kind: "item", label: t("menu.reload"), icon: "↻", onSelect: () => reload() },
    ];
  }

  // Escape closes the topmost layer
  function onKey(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (confirm) confirm = null;
    else if (menu) menu = null;
    else if (sheet) sheet = null;
    else if (selection) selection = null;
  }

  async function saveSettings(next: Config) {
    await api.saveConfig(next);
    config = next;
    view = "grid";
    pollHealth();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if view === "settings" && config}
  <Settings {config} initialAppId={editId} onCancel={() => (view = "grid")} onSave={saveSettings} />
{:else}
  <main
    class="shell"
    oncontextmenu={(e) => {
      e.preventDefault();
      menu = { x: e.clientX, y: e.clientY, app: null };
    }}
  >
    <header class="titlebar" data-tauri-drag-region>
      <h1 data-tauri-drag-region>{config?.title ?? "SvcDeck"}</h1>
      <div class="titlebar-actions">
        <button
          class="icon-btn"
          aria-label={t("menu.startAll")}
          title={t("menu.startAll")}
          disabled={!canStart}
          onclick={() => runAll("start", t("menu.startAll"))}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5.5v13l10.5-6.5Z" /></svg>
        </button>
        <button
          class="icon-btn"
          aria-label={t("menu.stopAll")}
          title={t("menu.stopAll")}
          disabled={!canStop}
          onclick={() => runAll("stop", t("menu.stopAll"))}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="6.5" y="6.5" width="11" height="11" rx="1.5" /></svg>
        </button>
        <button
          class="icon-btn"
          aria-label={t("sel.mode")}
          title={t("sel.mode")}
          aria-pressed={selection !== null}
          onclick={() => (selection = selection ? null : [])}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="4" />
            <path d="m8.5 12.5 2.5 2.5 4.5-5" />
          </svg>
        </button>
        <button class="icon-btn" aria-label={t("app.settings")} onclick={() => openSettings(null)}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z" />
            <path
              d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z"
            />
          </svg>
        </button>
        <button class="icon-btn close" aria-label={t("app.close")} onclick={() => getCurrentWindow().close()}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </div>
    </header>

    {#if error}
      <div class="banner" role="alert">{error}</div>
    {/if}

    {#if update}
      <div class="update" role="status">
        <div class="update-text">
          <strong>{t("upd.available", { version: update.version })}</strong>
          {#if info?.install === "package"}
            <small>{t("upd.hintPackage")}</small>
          {:else if info?.install === "user"}
            <small>{t("upd.hintUser")}</small>
          {/if}
        </div>
        <button class="pill" onclick={() => update && openUrl(update.url)}>{t("upd.details")}</button>
        <button class="icon-btn" aria-label={t("upd.dismiss")} onclick={dismissUpdate}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </div>
    {/if}

    <section class="grid" style:grid-template-columns="repeat({cols}, 1fr)" aria-busy={anyBusy}>
      {#each config?.apps ?? [] as app (app.id)}
        <Tile
          {app}
          icon={icons[app.id]}
          running={app.id in health ? health[app.id] : undefined}
          busy={!!busy[app.id]}
          selected={selection ? selection.includes(app.id) : undefined}
          onActivate={(e) => activate(app, e)}
          onMenu={(x, y) => (menu = { x, y, app })}
        />
      {/each}
    </section>

    {#if selection}
      <div class="selbar" role="toolbar" aria-label={t("sel.count", { n: selection.length })}>
        <button
          class="icon-btn"
          aria-label={config && selection.length === config.apps.length ? t("sel.none") : t("sel.all")}
          title={config && selection.length === config.apps.length ? t("sel.none") : t("sel.all")}
          onclick={toggleSelectAll}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="4" />
            {#if config && selection.length === config.apps.length}
              <path d="m8.5 12.5 2.5 2.5 4.5-5" />
            {:else if selection.length > 0}
              <path d="M8.5 12h7" />
            {/if}
          </svg>
        </button>
        <span class="selbar-count">{t("sel.count", { n: selection.length })}</span>
        <button
          class="pill primary"
          title={t("sel.startTitle")}
          aria-label={t("sel.startTitle")}
          disabled={!selCanStart}
          onclick={() => runFor(selectedApps, "start", t("sel.startTitle"))}
        >
          ▶<span class="selbar-label"> {t("sel.start")}</span>
        </button>
        <button
          class="pill danger"
          title={t("sel.stopTitle")}
          aria-label={t("sel.stopTitle")}
          disabled={!selCanStop}
          onclick={() => runFor(selectedApps, "stop", t("sel.stopTitle"))}
        >
          ■<span class="selbar-label"> {t("sel.stop")}</span>
        </button>
        <button class="icon-btn" aria-label={t("sel.done")} title={t("sel.done")} onclick={() => (selection = null)}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </div>
    {/if}
    {#if config && config.apps.length === 0}
      <p class="empty">{t("app.empty")}</p>
    {/if}

    {#if menu}
      <ContextMenu x={menu.x} y={menu.y} items={menuItems(menu)} onClose={() => (menu = null)} />
    {/if}

    {#if sheet}
      <OutputSheet title={sheet.title} entries={sheet.entries} onClose={() => (sheet = null)} />
    {/if}

    {#if confirm}
      <ConfirmDialog
        message={confirm.message}
        confirmLabel={confirm.confirmLabel}
        onCancel={() => (confirm = null)}
        onConfirm={() => {
          const run = confirm!.onConfirm;
          confirm = null;
          run();
        }}
      />
    {/if}
  </main>
{/if}
