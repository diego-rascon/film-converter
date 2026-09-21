<script lang="ts">
  import Icon from "./Icon.svelte";
  import ImageMenu from "./ImageMenu.svelte";
  import StatusChip from "./StatusChip.svelte";
  import { formatBytes } from "$lib/session.svelte";
  import type { ImageItem } from "$lib/types";

  interface Props {
    image: ImageItem;
    /** Every other row, so the eye can follow one across the columns. */
    striped: boolean;
    showOriginal: boolean;
    onopen: () => void;
    ontoggleSelect: (event: MouseEvent) => void;
    oninfo: () => void;
    onreveal: () => void;
    onremove: () => void;
  }

  let {
    image,
    striped,
    showOriginal,
    onopen,
    ontoggleSelect,
    oninfo,
    onreveal,
    onremove,
  }: Props = $props();

  let source = $derived(showOriginal ? image.original : image.developed);
</script>

<div class="row" class:striped class:selected={image.selected}>
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

    <span class="meta dims">
      {#if image.width && image.height}
        {image.width} × {image.height}
      {/if}
    </span>

    <span class="meta size">{formatBytes(image.bytes)}</span>
  </button>

  <span class="status">
    <StatusChip {image} />
  </span>

  <ImageMenu name={image.name} {oninfo} {onreveal} {onremove} />
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px var(--gutter);
  }

  /* These three carry equal specificity, so source order is what decides:
     a hovered stripe reads as hovered, and a selected row as selected
     whether it is striped, hovered or both. */
  .row.striped {
    background: var(--stripe);
  }

  .row:hover {
    background: var(--surface-2);
  }

  .row.selected {
    background: var(--accent-soft);
    box-shadow: inset 2px 0 0 var(--accent);
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
    width: var(--col-thumb, 52px);
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
    width: var(--col-meta, 92px);
    font-size: 11.5px;
    font-variant-numeric: tabular-nums;
    color: var(--text-faint);
    text-align: right;
  }

  /* Fixed so the chip lines up under the header rather than tracking its
     own label, which changes width with the status. */
  .status {
    display: flex;
    align-items: center;
    flex: none;
    width: var(--col-status, 104px);
  }

  /* The dimensions are the first thing to go when space is tight — at the
     window's 720px minimum the file size still fits, and it is the column
     the header can sort by. ListHeader drops it at the same width. */
  @media (max-width: 720px) {
    .dims {
      display: none;
    }
  }
</style>
