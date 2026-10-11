<script lang="ts">
  import { untrack } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";
  import { buildApp, needsTarget, TEMPLATE_KINDS, type TemplateKind } from "$lib/templates";
  import type { Config } from "$lib/types";
  import IconPreview from "./IconPreview.svelte";

  interface Props {
    config: Config;
    initialAppId: string | null;
    onSave: (config: Config) => Promise<void>;
    onCancel: () => void;
  }

  let { config, initialAppId, onSave, onCancel }: Props = $props();

  const GENERAL = "__general__";
  const NEW = "__new__";

  // Working copy; only applied with "Save"
  let draft = $state<Config>(untrack(() => $state.snapshot(config) as Config));
  let selected = $state<string>(untrack(() => initialAppId ?? GENERAL));
  let error = $state<string | null>(null);
  let saving = $state(false);
  let path = $state("");
  let detected = $state<string | null>(null);

  // "New app" form
  let tplKind = $state<TemplateKind>("systemd-user");
  let tplName = $state("");
  let tplTarget = $state("");
  let tplDir = $state("");

  $effect(() => {
    api.configPath().then((p) => (path = p)).catch(() => {});
    api.detectedTerminal().then((d) => (detected = d)).catch(() => {});
  });

  const appIndex = $derived(draft.apps.findIndex((a) => a.id === selected));
  const app = $derived(appIndex >= 0 ? draft.apps[appIndex] : null);
  const targetKind = $derived(needsTarget(tplKind));
  const canCreate = $derived(
    tplName.trim() !== "" &&
      (targetKind === null || tplTarget.trim() !== "") &&
      (tplKind !== "compose" || tplDir.trim() !== ""),
  );

  function slug(s: string) {
    return (
      s
        .toLowerCase()
        .normalize("NFKD")
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-|-$/g, "") || "app"
    );
  }

  function uniqueId(base: string, taken: string[]) {
    let id = base;
    for (let n = 2; taken.includes(id); n++) id = `${base}-${n}`;
    return id;
  }

  function moveApp(dir: -1 | 1) {
    const j = appIndex + dir;
    if (appIndex < 0 || j < 0 || j >= draft.apps.length) return;
    const [moved] = draft.apps.splice(appIndex, 1);
    draft.apps.splice(j, 0, moved);
  }

  function startNew() {
    tplName = "";
    tplTarget = "";
    tplDir = "";
    selected = NEW;
  }

  function createFromTemplate() {
    if (!canCreate) return;
    const id = uniqueId(slug(tplName), draft.apps.map((a) => a.id));
    draft.apps.push(
      buildApp({ kind: tplKind, id, name: tplName.trim(), target: tplTarget, dir: tplDir.trim() }),
    );
    selected = id;
  }

  function removeApp() {
    if (appIndex < 0) return;
    draft.apps.splice(appIndex, 1);
    selected = GENERAL;
  }

  function addAction() {
    if (!app) return;
    app.actions.push({
      id: uniqueId("action", app.actions.map((a) => a.id)),
      label: t("act.new"),
      command: "",
      mode: "capture",
      confirm: false,
    });
  }

  async function pickIcon() {
    const file = await openDialog({
      title: t("set.iconDialog"),
      multiple: false,
      directory: false,
      filters: [{ name: t("set.images"), extensions: ["png", "svg", "jpg", "jpeg", "webp", "gif", "ico", "avif"] }],
    });
    if (typeof file === "string" && app) app.icon = file;
  }

  async function pickDir(current: string): Promise<string | null> {
    const dir = await openDialog({
      title: t("set.dirDialog"),
      directory: true,
      multiple: false,
      defaultPath: current || undefined,
    });
    return typeof dir === "string" ? dir : null;
  }

  async function save() {
    saving = true;
    error = null;
    try {
      await onSave($state.snapshot(draft) as Config);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<main class="settings">
  <header class="titlebar" data-tauri-drag-region>
    <h1 data-tauri-drag-region>{t("set.title")}</h1>
    <div class="titlebar-actions">
      <button class="pill" onclick={onCancel}>{t("set.cancel")}</button>
      <button class="pill primary" onclick={save} disabled={saving}>
        {saving ? t("set.saving") : t("set.save")}
      </button>
    </div>
  </header>

  <div class="settings-body">
    <nav class="sidebar" aria-label={t("set.apps")}>
      <button
        class="side-item"
        aria-current={selected === GENERAL ? "true" : undefined}
        onclick={() => (selected = GENERAL)}>{t("set.general")}</button
      >
      <div class="side-sep">{t("set.apps")}</div>
      {#each draft.apps as a (a)}
        <button
          class="side-item"
          aria-current={selected === a.id ? "true" : undefined}
          onclick={() => (selected = a.id)}>{a.name || a.id}</button
        >
      {/each}
      <button class="side-item add" aria-current={selected === NEW ? "true" : undefined} onclick={startNew}
        >{t("set.addApp")}</button
      >
    </nav>

    <section class="pane">
      {#if error}
        <div class="banner" role="alert">{error}</div>
      {/if}

      {#if selected === GENERAL}
        <div class="row">
          <div class="field">
            <label for="s-title">{t("set.windowTitle")}</label>
            <input id="s-title" bind:value={draft.title} />
          </div>
          <div class="field">
            <label for="s-cols">{t("set.columns")}</label>
            <input
              id="s-cols"
              type="number"
              min="1"
              max="6"
              value={draft.columns}
              oninput={(e) => (draft.columns = Math.max(1, Math.min(6, Number(e.currentTarget.value) || 1)))}
            />
            <small>{t("set.columnsHint")}</small>
          </div>
        </div>
        <div class="row">
          <div class="field">
            <label for="s-lang">{t("set.language")}</label>
            <select id="s-lang" bind:value={draft.language}>
              <option value="auto">{t("set.langAuto")}</option>
              <option value="en">English</option>
              <option value="de">Deutsch</option>
            </select>
          </div>
          <div class="field">
            <label for="s-theme">{t("set.theme")}</label>
            <select id="s-theme" bind:value={draft.theme}>
              <option value="auto">{t("set.themeAuto")}</option>
              <option value="light">{t("set.themeLight")}</option>
              <option value="dark">{t("set.themeDark")}</option>
            </select>
          </div>
        </div>
        <div class="field">
          <label for="s-term">{t("set.terminal")}</label>
          <input
            id="s-term"
            class="mono"
            placeholder={detected ?? ""}
            value={draft.terminal.join(" ")}
            oninput={(e) => (draft.terminal = e.currentTarget.value.split(/\s+/).filter(Boolean))}
          />
          <small>{t("set.terminalHint", { detected: detected ?? t("set.terminalNone") })}</small>
        </div>
        <p class="note">{t("set.storedIn", { path })}</p>
        <p class="note warn-note">{t("set.security")}</p>
      {/if}

      {#if selected === NEW}
        <h3 class="section">{t("tpl.title")}</h3>
        <div class="row">
          <div class="field">
            <label for="t-kind">{t("tpl.type")}</label>
            <select id="t-kind" bind:value={tplKind}>
              {#each TEMPLATE_KINDS as k (k.kind)}
                <option value={k.kind}>{k.label()}</option>
              {/each}
            </select>
          </div>
          <div class="field">
            <label for="t-name">{t("set.name")}</label>
            <input id="t-name" bind:value={tplName} placeholder={t("tpl.newName")} />
          </div>
        </div>
        {#if targetKind}
          <div class="field">
            <label for="t-target">{targetKind === "unit" ? t("tpl.unit") : t("tpl.container")}</label>
            <input
              id="t-target"
              class="mono"
              bind:value={tplTarget}
              placeholder={targetKind === "unit" ? t("tpl.unitPh") : t("tpl.containerPh")}
            />
          </div>
        {/if}
        {#if tplKind === "compose"}
          <div class="field">
            <label for="t-dir">{t("tpl.composeDir")}</label>
            <div class="with-btn">
              <input id="t-dir" class="mono" bind:value={tplDir} />
              <button class="pill" onclick={async () => (tplDir = (await pickDir(tplDir)) ?? tplDir)}
                >{t("set.choose")}</button
              >
            </div>
          </div>
        {/if}
        <button class="pill primary" onclick={createFromTemplate} disabled={!canCreate}>{t("tpl.create")}</button>
      {/if}

      {#if app}
        <div class="app-head">
          <IconPreview spec={app.icon} glyph={app.glyph} />
          <div class="app-head-tools">
            <button class="pill" onclick={() => moveApp(-1)} disabled={appIndex === 0}>{t("set.moveUp")}</button>
            <button class="pill" onclick={() => moveApp(1)} disabled={appIndex === draft.apps.length - 1}
              >{t("set.moveDown")}</button
            >
            <button class="pill danger" onclick={removeApp}>{t("set.remove")}</button>
          </div>
        </div>

        <div class="row">
          <div class="field">
            <label for="a-name">{t("set.name")}</label>
            <input id="a-name" bind:value={app.name} />
          </div>
          <div class="field">
            <label for="a-id">{t("set.id")}</label>
            <input
              id="a-id"
              class="mono"
              value={app.id}
              oninput={(e) => {
                const next = slug(e.currentTarget.value);
                app.id = next;
                selected = next;
              }}
            />
            <small>{t("set.idHint")}</small>
          </div>
        </div>

        <div class="field">
          <label for="a-url">{t("set.url")}</label>
          <input id="a-url" class="mono" placeholder="http://127.0.0.1:8080" bind:value={app.url} />
          <small>{t("set.urlHint")}</small>
        </div>

        <div class="row">
          <div class="field">
            <label for="a-icon">{t("set.icon")}</label>
            <div class="with-btn">
              <input id="a-icon" class="mono" bind:value={app.icon} />
              <button class="pill" onclick={pickIcon}>{t("set.choose")}</button>
            </div>
            <small>{t("set.iconHint")}</small>
          </div>
          <div class="field">
            <label for="a-glyph">{t("set.glyph")}</label>
            <input id="a-glyph" maxlength="4" bind:value={app.glyph} />
            <small>{t("set.glyphHint")}</small>
          </div>
        </div>

        <div class="field">
          <label for="a-dir">{t("set.dir")}</label>
          <div class="with-btn">
            <input id="a-dir" class="mono" bind:value={app.dir} />
            <button
              class="pill"
              onclick={async () => {
                const d = await pickDir(app.dir);
                if (d) app.dir = d;
              }}>{t("set.choose")}</button
            >
          </div>
          <small>{t("set.dirHint")}</small>
        </div>

        <div class="row">
          <div class="field">
            <label for="a-hcmd">{t("set.healthCmd")}</label>
            <input
              id="a-hcmd"
              class="mono"
              placeholder="systemctl --user is-active --quiet foo"
              bind:value={app.healthCommand}
            />
            <small>{t("set.healthCmdHint")}</small>
          </div>
          <div class="field">
            <label for="a-health">{t("set.healthPort")}</label>
            <input id="a-health" class="mono" placeholder="127.0.0.1:8080" bind:value={app.health} />
            <small>{t("set.healthPortHint")}</small>
          </div>
        </div>

        <h3 class="section">{t("set.actions")}</h3>
        <div class="actions-list">
          {#each app.actions as a, ai (a)}
            <div class="action-card">
              <div class="row">
                <div class="field">
                  <label for="act-label-{ai}">{t("set.actLabel")}</label>
                  <input id="act-label-{ai}" bind:value={a.label} />
                </div>
                <div class="field">
                  <label for="act-id-{ai}">{t("set.id")}</label>
                  <input
                    id="act-id-{ai}"
                    class="mono"
                    value={a.id}
                    oninput={(e) => (a.id = slug(e.currentTarget.value))}
                  />
                </div>
              </div>
              <div class="field">
                <label for="act-cmd-{ai}">{t("set.actCommand")}</label>
                <input id="act-cmd-{ai}" class="mono" bind:value={a.command} />
              </div>
              <div class="action-flags">
                <label>
                  <input
                    type="checkbox"
                    checked={a.mode === "terminal"}
                    onchange={(e) => (a.mode = e.currentTarget.checked ? "terminal" : "capture")}
                  />
                  {t("set.actTerminal")}
                </label>
                <label>
                  <input type="checkbox" bind:checked={a.confirm} />
                  {t("set.actConfirm")}
                </label>
                <button class="link danger" onclick={() => app.actions.splice(ai, 1)}>{t("set.remove")}</button>
              </div>
            </div>
          {/each}
          <button class="pill" onclick={addAction}>{t("set.addAction")}</button>
        </div>
      {/if}
    </section>
  </div>
</main>
