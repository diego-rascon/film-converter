<script lang="ts" module>
  /** The row's own controls, which answer a click for themselves. */
  const CONTROLS = '.image-menu, input[type="checkbox"]';
</script>

<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import ImageMenu from "./ImageMenu.svelte";
  import StatusChip from "$components/image/StatusChip.svelte";
  import { formatBytes } from "$utils/format";
  import type { ImageTileProps } from "../types";
  import { openOnEnter, previewSource } from "../utils/tile";

  /** One scan in list view. Takes a card's props plus the stripe. */
  interface Props extends ImageTileProps {
    /** Every other row, so the eye can follow one across the columns. */
    striped: boolean;
  }

  let {
    image,
    striped,
    anySelected,
    showOriginal,
    onpick,
    onopen,
    ontoggleSelect,
    oninfo,
    onreveal,
    onremove,
  }: Props = $props();

  let source = $derived(previewSource(image, showOriginal));

  /**
   * The whole row is the image's hit target — the gutters, the gaps and the
   * status column included — the way a file manager's rows are, rather than
   * only the cells. The tick box and the menu are the exception, and one test
   * for them here beats a `stopPropagation` on each: it covers the menu's
   * slot around its glyph as well as the glyph, and it covers `dblclick`,
   * which a stopped `click` does not.
   */
  function onRow(handle: (event: MouseEvent) => void) {
    return (event: MouseEvent) => {
      if (!(event.target as HTMLElement).closest(CONTROLS)) handle(event);
    };
  }
</script>

<div
  class="row tile"
  class:striped
  class:picking={anySelected}
  class:selected={image.selected}
  onclick={onRow(onpick)}
  ondblclick={onRow(onopen)}
  role="presentation"
>
  <input
    type="checkbox"
    checked={image.selected}
    onclick={ontoggleSelect}
    aria-label="Select {image.name}"
  />

  <!-- The pointer is the row's business, so this button is here for the
       keyboard: it is the row's tab stop, Enter opens the image and Space
       raises a click the row picks on. -->
  <button class="open" onkeydown={openOnEnter(onopen)}>
    <span class="thumb">
      {#if image.previewStatus === "ready" && source}
        <img src={source} alt="" loading="lazy" decoding="async" />
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
    /* All of it answers the pointer, so all of it says so. */
    cursor: pointer;
    /* The header's padding exactly, both ends — the checkbox on the gutter
       and the menu button's 6px of slack paid back at the far end — or the
       two stop sharing one column grid. */
    padding: 5px calc(var(--gutter) - 6px) 5px var(--gutter);
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

  /* The box is flanked by the gutter: the window's line at one end, the same
     distance from the thumbnail at the other — the flex gap plus this. The
     header's checkbox carries the identical margin, or the two stop sharing
     one column grid. */
  .row > input[type="checkbox"] {
    margin-right: calc(var(--gutter) - 10px);
    /* And it stays out of the way until it is wanted: under the pointer, or
       on every row at once as soon as something is picked, the way a card's
       box comes up on hover. Faded rather than hidden, so the column it holds
       never collapses and the rows keep the header's grid. */
    opacity: 0;
    /* The base rule's transition restated, since a `transition` here replaces
       it outright and the fill would otherwise snap when the box is ticked. */
    transition:
      opacity 0.12s ease,
      background-color 0.12s ease,
      border-color 0.12s ease;
  }

  .row:hover > input[type="checkbox"],
  .row.picking > input[type="checkbox"],
  .row > input[type="checkbox"]:focus-visible {
    opacity: 1;
  }

  .open {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
    text-align: left;
  }

  /* One target, one ring. `.open` is only the row's tab stop, so the ring it
     would draw for itself boxes the cells and stops short of the status
     column — which is inside the row's hit area just as much as the name is.
     The row wears it instead, drawn 2px *inside* rather than the base rule's
     2px out: the rows are full-bleed, so an outward ring is clipped at the
     window's edge and laps over the rows above and below. */
  .open:focus-visible {
    outline: none;
  }

  .row:has(.open:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
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
     the header can sort by. ViewHeader drops it at the same width. */
  @media (max-width: 720px) {
    .dims {
      display: none;
    }
  }
</style>
