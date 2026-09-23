<script lang="ts" module>
  import type { ImageStatus } from "$types";

  const labels: Record<ImageStatus, string> = {
    pending: "Ready",
    developing: "Developing",
    done: "Developed",
    error: "Failed",
  };
</script>

<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import type { ImageItem } from "$types";

  interface Props {
    image: ImageItem;
    /** Icon only, for the tight space under a grid card. */
    compact?: boolean;
  }

  let { image, compact = false }: Props = $props();
</script>

{#if image.status !== "pending" || !compact}
  <span
    class="chip {image.status}"
    class:compact
    title={image.error ?? labels[image.status]}
  >
    {#if image.status === "done"}
      <Icon name="check" size={12} />
    {:else if image.status === "error"}
      <Icon name="alert" size={12} />
    {:else if image.status === "developing"}
      <span class="spinner"></span>
    {/if}
    {#if !compact}
      <span>{labels[image.status]}</span>
    {/if}
  </span>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex: none;
    padding: 2px 8px;
    border-radius: 99px;
    font-size: 11px;
    font-weight: 500;
    background: var(--surface-sunken);
    color: var(--text-muted);
  }

  .chip.compact {
    padding: 3px;
  }

  .chip.done {
    background: transparent;
    color: var(--success);
  }

  .chip.error {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .chip.developing {
    color: var(--accent);
    background: var(--accent-soft);
  }

  .spinner {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    animation: spin 0.7s linear infinite;
  }
</style>
