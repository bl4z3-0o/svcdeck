<script lang="ts">
  import { tick } from "svelte";
  import { parseAnsi } from "$lib/ansi";
  import { t } from "$lib/i18n.svelte";
  import type { SheetEntry } from "$lib/ui";

  interface Props {
    title: string;
    entries: SheetEntry[];
    onClose: () => void;
  }

  let { title, entries, onClose }: Props = $props();

  const stateText = (s: SheetEntry["state"]) =>
    s === "pending" ? t("sheet.pending") : s === "running" ? t("sheet.running") : s === "ok" ? t("sheet.ok") : t("sheet.fail");

  let body = $state<HTMLDivElement>();
  let closeBtn = $state<HTMLButtonElement>();
  const running = $derived(entries.some((e) => e.state === "running" || e.state === "pending"));

  $effect(() => closeBtn?.focus());

  // Scroll to the bottom on new output
  $effect(() => {
    // Dependency: output and state of every entry
    entries.forEach((e) => [e.output, e.state]);
    tick().then(() => body?.scrollTo({ top: body.scrollHeight, behavior: "smooth" }));
  });

  function copy() {
    const plain = entries
      .map((e) => `## ${e.label}\n${e.output.replace(/\x1b\[[0-9;]*[A-Za-z]/g, "")}`)
      .join("\n\n");
    navigator.clipboard?.writeText(plain).catch(() => {});
  }
</script>

<div
  class="sheet-backdrop"
  role="presentation"
  onmousedown={(e) => e.target === e.currentTarget && onClose()}
>
  <div class="sheet glass" role="dialog" aria-modal="true" aria-label={title}>
    <header class="sheet-head">
      <h2>{title}</h2>
      <div class="sheet-tools">
        <button class="pill" onclick={copy} disabled={running}>{t("sheet.copy")}</button>
        <button bind:this={closeBtn} class="pill" onclick={onClose}>{t("sheet.close")}</button>
      </div>
    </header>
    <div class="sheet-body" bind:this={body} aria-live="polite">
      {#each entries as e, i (i)}
        <article class="entry" data-state={e.state}>
          <div class="entry-head">
            <span class="entry-state" aria-hidden="true"></span>
            <span class="entry-label">{e.label}</span>
            <span class="entry-status">{stateText(e.state)}</span>
          </div>
          {#if e.output}
            <pre class="output">{#each parseAnsi(e.output) as s, j (j)}<span
                  data-color={s.color}
                  data-bold={s.bold ? "true" : undefined}>{s.text}</span
                >{/each}</pre>
          {/if}
        </article>
      {/each}
    </div>
  </div>
</div>
