<script lang="ts">
  import Icon from "./Icon.svelte";
  import StatusChip from "./StatusChip.svelte";
  import type { ImageItem } from "$lib/types";

  interface Props {
    image: ImageItem;
    showOriginal: boolean;
    onopen: () => void;
    ontoggleSelect: (event: MouseEvent) => void;
    onremove: () => void;
  }

  let { image, showOriginal, onopen, ontoggleSelect, onremove }: Props =
    $props();

  let source = $derived(showOriginal ? image.original : image.developed);

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    const units = ["KB", "MB", "GB"];
    let value = bytes / 1024;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unit]}`;
  }
</script>

<div class="row" class:selected={image.selected}>
  <input
    type="checkbox"
    checked={image.selected}
    onclick={ontoggleSelect}
    aria-label="Select {image.name}"
  />

  <button class="open" onclick={onopen}>
    <span class="thumb">
      {#if image.previewStatus === "ready" && source}
        <img src={source} alt="" loading="lazy" />
      {:else if image.previewStatus === "error"}
        <Icon name="alert" size={14} />
      {:else}
        <span class="shimmer"></span>
      {/if}
    </span>

    <span class="name" title={image.path}>{image.name}</span>

    <span class="meta">
      {#if image.width && image.height}
        {image.width} × {image.height}
      {/if}
    </span>

    <span class="meta">{formatBytes(image.bytes)}</span>
  </button>

  <StatusChip {image} />

  <button
    class="icon-btn small"
    onclick={onremove}
    aria-label="Remove {image.name}"
    title="Remove"
  >
    <Icon name="close" size={15} />
  </button>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 10px 5px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
  }

  .row:hover {
    background: var(--surface-2);
  }

  .row.selected {
    background: var(--accent-soft);
    border-color: var(--accent);
  }

  .open {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
    text-align: left;
  }

  .thumb {
    display: grid;
    place-items: center;
    width: 52px;
    height: 38px;
    flex: none;
    border-radius: 4px;
    overflow: hidden;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    color: var(--danger);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .shimmer {
    width: 100%;
    height: 100%;
    background: linear-gradient(
      100deg,
      var(--surface-sunken) 30%,
      var(--surface-2) 50%,
      var(--surface-sunken) 70%
    );
    background-size: 220% 100%;
    animation: sweep 1.3s ease-in-out infinite;
  }

  @keyframes sweep {
    to {
      background-position: -120% 0;
    }
  }

  .name {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta {
    flex: none;
    width: 92px;
    font-size: 11.5px;
    font-variant-numeric: tabular-nums;
    color: var(--text-faint);
    text-align: right;
  }

  .icon-btn.small {
    width: 28px;
    height: 28px;
  }

  /* The metadata columns are the first thing to go when space is tight. */
  @media (max-width: 720px) {
    .meta {
      display: none;
    }
  }
</style>
