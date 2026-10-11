<script lang="ts">
  import { t } from "$lib/i18n.svelte";

  interface Props {
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let { message, confirmLabel, onConfirm, onCancel }: Props = $props();

  let cancelBtn = $state<HTMLButtonElement>();
  $effect(() => cancelBtn?.focus());
</script>

<div
  class="sheet-backdrop"
  role="presentation"
  onmousedown={(e) => e.target === e.currentTarget && onCancel()}
>
  <div class="confirm glass" role="alertdialog" aria-modal="true" aria-label={t("confirm.title")}>
    <p>{message}</p>
    <div class="confirm-actions">
      <button bind:this={cancelBtn} class="pill" onclick={onCancel}>{t("confirm.cancel")}</button>
      <button class="pill danger" onclick={onConfirm}>{confirmLabel}</button>
    </div>
  </div>
</div>
