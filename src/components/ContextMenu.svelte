<script lang="ts">
  import { tick } from "svelte";
  import type { MenuItem } from "$lib/ui";

  interface Props {
    x: number;
    y: number;
    items: MenuItem[];
    onClose: () => void;
  }

  let { x, y, items, onClose }: Props = $props();

  let el = $state<HTMLDivElement>();
  let pos = $state({ left: 0, top: 0 });

  // Keep inside the window and focus the first item
  $effect(() => {
    const px = x;
    const py = y;
    pos = { left: px, top: py };
    tick().then(() => {
      if (!el) return;
      const { width, height } = el.getBoundingClientRect();
      const m = 8;
      pos = {
        left: Math.max(m, Math.min(px, window.innerWidth - width - m)),
        top: Math.max(m, Math.min(py, window.innerHeight - height - m)),
      };
      el.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
    });
  });

  function onWindowDown(e: MouseEvent) {
    if (!el?.contains(e.target as Node)) onClose();
  }

  // Arrow key navigation
  function onKeydown(e: KeyboardEvent) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const btns = Array.from(el?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
    const i = btns.indexOf(document.activeElement as HTMLButtonElement);
    const next = e.key === "ArrowDown" ? (i + 1) % btns.length : (i - 1 + btns.length) % btns.length;
    btns[next]?.focus();
  }
</script>

<svelte:window onmousedown={onWindowDown} onblur={onClose} />

<div
  bind:this={el}
  class="menu glass"
  role="menu"
  tabindex="-1"
  style:left="{pos.left}px"
  style:top="{pos.top}px"
  onkeydown={onKeydown}
  oncontextmenu={(e) => {
    e.preventDefault();
    e.stopPropagation();
  }}
>
  {#each items as it, i (i)}
    {#if it.kind === "separator"}
      <div class="menu-sep" role="separator"></div>
    {:else if it.kind === "header"}
      <div class="menu-header">
        <span>{it.label}</span>
        {#if it.hint}
          <span class="menu-hint" data-tone={it.tone}>{it.hint}</span>
        {/if}
      </div>
    {:else}
      <button
        role="menuitem"
        class="menu-item"
        data-danger={it.danger ? "true" : undefined}
        disabled={it.disabled}
        onclick={() => {
          onClose();
          it.onSelect();
        }}
      >
        <span class="menu-icon" aria-hidden="true">{it.icon}</span>
        {it.label}
      </button>
    {/if}
  {/each}
</div>
