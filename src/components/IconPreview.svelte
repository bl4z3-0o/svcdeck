<script lang="ts">
  import defaultIcon from "../assets/default-icon.svg";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";

  interface Props {
    spec: string;
    glyph: string;
  }

  let { spec, glyph }: Props = $props();

  // A result belongs to exactly one input; stale results are ignored
  let loaded = $state<{ spec: string; src: string | null }>({ spec: "", src: null });

  $effect(() => {
    const s = spec.trim();
    if (!s) return;
    let cancelled = false;
    const timer = setTimeout(() => {
      api
        .loadIcon(s)
        .then((src) => !cancelled && (loaded = { spec: s, src }))
        .catch(() => !cancelled && (loaded = { spec: s, src: null }));
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  const current = $derived(spec.trim() !== "" && loaded.spec === spec.trim());
  const src = $derived(current ? loaded.src : null);
  const missing = $derived(current && loaded.src === null);
  const short = $derived(glyph.trim().slice(0, 4));
</script>

<div class="preview">
  <span class="tile-icon">
    {#if src}
      <img {src} alt="" />
    {:else if short}
      <span class="glyph" data-len={short.length}>{short}</span>
    {:else}
      <img src={defaultIcon} alt="" />
    {/if}
  </span>
  {#if missing}
    <small class="warn">{t("set.iconMissing")}</small>
  {/if}
</div>
