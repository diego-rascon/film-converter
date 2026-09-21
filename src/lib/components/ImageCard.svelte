<script lang="ts">
  import Icon from "./Icon.svelte";
  import ImageMenu from "./ImageMenu.svelte";
  import StatusChip from "./StatusChip.svelte";
  import type { ImageItem } from "$lib/types";

  interface Props {
    image: ImageItem;
    /** Show the untouched scan instead of the developed result. */
    showOriginal: boolean;
    onopen: () => void;
    ontoggleSelect: (event: MouseEvent) => void;
    oninfo: () => void;
    onreveal: () => void;
    onremove: () => void;
  }

  let {
    image,
    showOriginal,
    onopen,
    ontoggleSelect,
    oninfo,
    onreveal,
    onremove,
  }: Props = $props();

  let source = $derived(showOriginal ? image.original : image.developed);
</script>

<figure class:selected={image.selected}>
  <div class="frame">
    <!-- The whole thumbnail opens the viewer; the checkbox below handles
         selection, so this button carries only the "open" role. -->
    <button class="surface" onclick={onopen} aria-label="Open {image.name}">
      {#if image.previewStatus === "ready" && source}
        <img src={source} alt={image.name} loading="lazy" />
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

    <span class="pick" class:visible={image.selected}>
      <input
        type="checkbox"
        checked={image.selected}
        onclick={ontoggleSelect}
        aria-label="Select {image.name}"
      />
    </span>
  </div>

  <ImageMenu
    variant="overlay"
    name={image.name}
    {oninfo}
    {onreveal}
    {onremove}
  />

  <figcaption>
    <span class="name" title={image.path}>{image.name}</span>
    <StatusChip {image} compact />
  </figcaption>
</figure>

<style>
  figure {
    /* The actions menu is a child of the figure rather than of the frame:
       its popover would be clipped by the frame's `overflow: hidden`. */
    position: relative;
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

  .shimmer {
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

  .pick {
    position: absolute;
    top: 7px;
    left: 7px;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    color: #fff;
    opacity: 0;
    cursor: pointer;
    transition: opacity 0.12s ease;
  }

  /* The glass behind it is dark whatever the theme, so an unchecked box takes
     its contrast from the chip rather than from the surface tokens, which
     would vanish into it in dark mode. Checked keeps the accent fill. */
  .pick input[type="checkbox"]:not(:checked) {
    background-color: rgba(255, 255, 255, 0.16);
    border-color: rgba(255, 255, 255, 0.8);
  }

  .pick input[type="checkbox"]:not(:checked):hover {
    background-color: rgba(255, 255, 255, 0.28);
    border-color: #fff;
  }

  /* Hovering anywhere on the figure — the caption included — brings both
     overlays up, so they appear and disappear together. The menu reveals
     itself the same way; it owns its own opacity while it is open. */
  figure:hover .pick,
  .pick:focus-within,
  .pick.visible {
    opacity: 1;
  }

  figure:hover :global(.image-menu.overlay) {
    opacity: 1;
  }

  figcaption {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-width: 0;
    padding: 0 2px;
  }

  .name {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
