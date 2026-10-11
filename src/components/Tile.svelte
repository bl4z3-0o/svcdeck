<script lang="ts">
  import defaultIcon from "../assets/default-icon.svg";
  import { t } from "$lib/i18n.svelte";
  import type { App } from "$lib/types";

  interface Props {
    app: App;
    icon?: string;
    /** null = no status check configured */
    running: boolean | null | undefined;
    busy: boolean;
    onActivate: () => void;
    onMenu: (x: number, y: number) => void;
  }

  let { app, icon, running, busy, onActivate, onMenu }: Props = $props();

  // Icon > explicit glyph > default icon
  const glyph = $derived(app.glyph.trim().slice(0, 4));
  const statusText = $derived(
    busy
      ? t("status.working")
      : running === true
        ? t("status.running")
        : running === false
          ? t("status.off")
          : t("status.unknown"),
  );
  const what = $derived(
    app.url ? t("tile.open", { name: app.name }) : app.actions[0] ? t("tile.run", { action: app.actions[0].label }) : app.name,
  );

  function onKeydown(e: KeyboardEvent) {
    // Context menu via keyboard (menu key or Shift+F10)
    if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
      e.preventDefault();
      const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
      onMenu(r.left + r.width / 2, r.top + r.height / 2);
    }
  }
</script>

<button
  class="tile"
  data-running={running === true ? "true" : running === false ? "false" : "none"}
  data-busy={busy ? "true" : "false"}
  aria-label="{what}. {t('tile.label', { name: app.name, status: statusText })}"
  onclick={onActivate}
  oncontextmenu={(e) => {
    e.preventDefault();
    e.stopPropagation();
    onMenu(e.clientX, e.clientY);
  }}
  onkeydown={onKeydown}
>
  <span class="tile-icon">
    {#if icon}
      <img src={icon} alt="" draggable="false" />
    {:else if glyph}
      <span class="glyph" data-len={glyph.length}>{glyph}</span>
    {:else}
      <img src={defaultIcon} alt="" draggable="false" />
    {/if}
    {#if running !== null}
      <span class="status-dot" aria-hidden="true"></span>
    {/if}
    {#if busy}
      <span class="spinner" aria-hidden="true"></span>
    {/if}
  </span>
  <span class="tile-label">{app.name}</span>
</button>
