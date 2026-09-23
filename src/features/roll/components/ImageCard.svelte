<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import ImageMenu from "./ImageMenu.svelte";
  import StatusChip from "$components/image/StatusChip.svelte";
  import type { ImageTileProps } from "../types";
  import { openOnEnter, previewPriority, previewSource } from "../utils/tile";

  /** One scan in grid view. Takes the same props a row does. */
  let {
    image,
    anySelected,
    showOriginal,
    onpick,
    onopen,
    ontoggleSelect,
    oninfo,
    onreveal,
    onremove,
  }: ImageTileProps = $props();

  let source = $derived(previewSource(image, showOriginal));
</script>

<figure class:selected={image.selected} {@attach previewPriority(image.path)}>
  <div class="frame">
    <!-- A file manager's gestures: one click on the thumbnail picks the
         image, a double click opens it. The checkbox below is the way to
         build a selection without holding a key down. -->
    <button
      class="surface tile"
      onclick={onpick}
      ondblclick={onopen}
      onkeydown={openOnEnter(onopen)}
      aria-label="Open {image.name}"
    >
      {#if image.previewStatus === "ready" && source}
        <img src={source} alt={image.name} loading="lazy" decoding="async" />
      {:else if image.previewStatus === "error"}
        <span class="placeholder error">
          <Icon name="alert" size={20} />
          Could not read
        </span>
      {:else}
        <span class="placeholder shimmer"></span>
      {/if}
    </button>

    {#if showOriginal && image.previewStatus === "ready"}
      <span class="badge">Before</span>
    {/if}

    <input
      type="checkbox"
      class="pick"
      class:visible={anySelected}
      checked={image.selected}
      onclick={ontoggleSelect}
      aria-label="Select {image.name}"
    />
  </div>

  <figcaption>
    <span class="name" title={image.path}>{image.name}</span>
    <StatusChip {image} compact />
    <ImageMenu
      variant="caption"
      name={image.name}
      {oninfo}
      {onreveal}
      {onremove}
    />
  </figcaption>
</figure>

<style>
  figure {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-width: 0;
  }

  .frame {
    position: relative;
    border-radius: var(--radius);
    overflow: hidden;
    border: 1px solid var(--border);
    background: var(--surface-sunken);
    transition:
      border-color 0.12s ease,
      box-shadow 0.12s ease;
  }

  figure.selected .frame {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .frame:hover {
    border-color: var(--border-strong);
  }

  .surface {
    display: block;
    width: 100%;
    aspect-ratio: 4 / 3;
    background: var(--surface-sunken);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    height: 100%;
    font-size: 11.5px;
    color: var(--text-faint);
  }

  .placeholder.error {
    color: var(--danger);
  }

  .badge {
    position: absolute;
    left: 8px;
    bottom: 8px;
    padding: 2px 7px;
    border-radius: 99px;
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    color: #fff;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    pointer-events: none;
  }

  /* Floats directly on the picture rather than inside a glass container of
     its own, which would put a second rounded square around the box's own
     corners. Its shape is the one every checkbox in the app already has
     (`input[type="checkbox"]` in controls.css); only its position and its
     unchecked colours are added here. */
  .pick {
    position: absolute;
    top: 8px;
    left: 8px;
    opacity: 0;
    transition:
      opacity 0.12s ease,
      background-color 0.12s ease,
      border-color 0.12s ease;
  }

  /* The same frosted background the badge and the viewer's glass use, rather
     than a white fill of its own — it is what keeps this checkbox reading
     as the same object as everything else that floats over a picture. A
     bright white border read as a halo against it, so the outline is the
     same soft hairline the viewer's pods draw instead. Checked keeps the
     global accent fill and tick, unchanged. */
  .pick:not(:checked) {
    background-color: var(--glass);
    backdrop-filter: var(--glass-blur);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .pick:not(:checked):hover:not(:disabled) {
    background-color: var(--glass-hover);
    border-color: rgba(255, 255, 255, 0.55);
  }

  /* Hovering anywhere on the figure — the caption included — brings the box
     and the menu up together, since they are the card's two controls. The
     box also stays up on every card once anything is picked: with a
     selection running, the next click is likely to extend it. The menu owns
     its own opacity while it is open, so the pointer can leave the card
     without the popover's own button vanishing under it. */
  figure:hover .pick,
  .pick:focus-visible,
  .pick.visible {
    opacity: 1;
  }

  figure:hover :global(.image-menu.caption) {
    opacity: 1;
  }

  /* The caption reads in the list's order — name, status, actions — so an
     image's controls are in the same sequence in both views. The menu ends
     it rather than floating over the thumbnail: the picture is what the
     grid is for, and the popover still escapes `.frame`'s `overflow:
     hidden` down here. */
  figcaption {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    /* The name's ink sits 2px inside the frame; the menu gives that back at
       its end, where the glyph's box is wider than its ink. */
    padding: 0 2px;
  }

  .name {
    /* Takes the slack, so the chip and the menu stay on the right edge
       however short the name is. */
    flex: 1;
    min-width: 0;
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
