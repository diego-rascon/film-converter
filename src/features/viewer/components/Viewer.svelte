<script lang="ts">
  import { untrack } from "svelte";

  import ViewerDetails from "./ViewerDetails.svelte";
  import ViewerHeader from "./ViewerHeader.svelte";
  import ViewerStage from "./ViewerStage.svelte";
  import type { Mode } from "../types";
  import { SharpPreviews } from "../state/sharp.svelte";
  import { ZoomPan } from "../state/zoom.svelte";
  import { session } from "$state/session.svelte";
  import type { ImageItem } from "$types";

  /**
   * The fullscreen comparison. This file owns what the viewer *is* — which
   * image, which mode, the sharper pair, the keyboard — and hands the
   * drawing to the three pieces under `viewer/`.
   */
  interface Props {
    image: ImageItem;
    index: number;
    total: number;
    /** The grid's before/after toggle, which is the mode the viewer opens in. */
    showOriginal: boolean;
    /** Shows a file in the file manager; the page owns the failure notice. */
    onreveal: (path: string) => void;
  }

  let { image, index, total, showOriginal, onreveal }: Props = $props();

  // Opens on whichever side the grid or list was showing, so clicking an
  // image does not change what is on screen. The viewer is mounted per
  // opening, so this is read once — untracked — and the switch is free to
  // move from there.
  let mode = $state<Mode>(untrack(() => (showOriginal ? "before" : "after")));
  let details = $state(false);

  const zoom = new ZoomPan();
  const sharp = new SharpPreviews();

  // A larger pair for whichever image is on screen, once the viewer has
  // settled on it. The card-sized preview stays up until it arrives, so
  // there is never a blank frame.
  $effect(() => sharp.fetch(image.path));

  let pair = $derived(
    sharp.get(image.path) ?? { original: image.original, developed: image.developed },
  );
  let ready = $derived(Boolean(pair.developed && pair.original));

  // A new scan arrives fitted, not at wherever the last one was left.
  $effect(() => {
    void image.path;
    zoom.reset();
  });

  // The pan limit is measured on the fitted picture, so the stage needs to
  // know its shape.
  $effect(() => {
    zoom.aspect =
      image.width && image.height ? image.width / image.height : null;
  });

  function onkeydown(event: KeyboardEvent) {
    switch (event.key) {
      case "Escape":
        session.closeViewer();
        break;
      case "ArrowLeft":
        event.preventDefault();
        session.stepViewer(-1);
        break;
      case "ArrowRight":
        event.preventDefault();
        session.stepViewer(1);
        break;
      case "b":
      case "B":
        mode = mode === "before" ? "after" : "before";
        break;
      case "i":
      case "I":
        details = !details;
        break;
      case "+":
      case "=":
        event.preventDefault();
        zoom.by(ZoomPan.STEP);
        break;
      case "-":
      case "_":
        event.preventDefault();
        zoom.by(1 / ZoomPan.STEP);
        break;
      case "0":
        event.preventDefault();
        zoom.reset();
        break;
      case "Delete":
      case "Backspace":
        event.preventDefault();
        session.remove([image.path]);
        break;
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="viewer">
  <ViewerHeader
    {image}
    {index}
    {total}
    {mode}
    {details}
    onmode={(next) => (mode = next)}
    ontoggleDetails={() => (details = !details)}
    onreveal={() => onreveal(image.path)}
  />

  <div class="body">
    <ViewerStage {image} {pair} {ready} {mode} {total} {zoom} />

    {#if details}
      <ViewerDetails {image} onclose={() => (details = false)} />
    {/if}
  </div>

  <footer>
    <kbd>←</kbd><kbd>→</kbd> browse
    <span class="sep">·</span>
    <kbd>+</kbd><kbd>−</kbd> zoom
    <span class="sep">·</span>
    <kbd>B</kbd> before / after
    <span class="sep">·</span>
    <kbd>I</kbd> details
    <span class="sep">·</span>
    <kbd>Del</kbd> remove
    <span class="sep">·</span>
    <kbd>Esc</kbd> close
  </footer>
</div>

<style>
  /* The viewer covers the app rather than replacing it, so the picture sits
     on a blurred, darkened cast of the grid it was opened from. The plain
     background is the fallback: where backdrop-filter is missing the same
     rule has to still read as a dark scrim, so it carries more black. */
  .viewer {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    flex-direction: column;
    background: rgba(9, 9, 11, 0.9);
    animation: fade-in 0.13s ease;
  }

  @supports (backdrop-filter: blur(1px)) or
    (-webkit-backdrop-filter: blur(1px)) {
    .viewer {
      background: var(--viewer-backdrop);
      -webkit-backdrop-filter: blur(34px) saturate(1.3);
      backdrop-filter: blur(34px) saturate(1.3);
    }
  }

  /* The one inset in the viewer's body, and the gutter every floating
     control in the stage is already on: the picture clears the window by the
     same margin its own pods do, so the two line up on one frame. Padding
     here rather than on the stage keeps the stage's box and the picture's
     box the same rectangle, which is what the pan clamp, the wipe and the
     wheel's zoom origin all measure against. */
  .body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--gutter);
    min-height: 0;
    padding: var(--gutter);
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    flex: none;
    height: 36px;
    font-size: 11.5px;
    color: var(--text-faint);
    border-top: 1px solid var(--border);
    background: var(--surface);
  }

  kbd {
    display: inline-grid;
    place-items: center;
    min-width: 19px;
    height: 19px;
    padding: 0 5px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    font-family: inherit;
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .sep {
    margin: 0 5px;
    opacity: 0.5;
  }

  @media (max-width: 720px) {
    footer {
      display: none;
    }
  }
</style>
