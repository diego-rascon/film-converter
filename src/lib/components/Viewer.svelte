<script lang="ts">
  import Icon from "./Icon.svelte";
  import ImageFacts from "./ImageFacts.svelte";
  import StatusChip from "./StatusChip.svelte";
  import WindowControls from "./WindowControls.svelte";
  import { FULL_PREVIEW_EDGE, buildPreview } from "$lib/api";
  import { session } from "$lib/session.svelte";
  import type { ImageItem } from "$lib/types";

  interface Props {
    image: ImageItem;
    index: number;
    total: number;
    /** Shows the image in the file manager; the page owns the failure notice. */
    onreveal: () => void;
  }

  let { image, index, total, onreveal }: Props = $props();

  type Mode = "before" | "wipe" | "after";
  let mode = $state<Mode>("wipe");
  /**
   * The details panel is a sidebar, not a dialog: it stays open while the
   * arrows move through the roll, and `ImageFacts` re-reads each file as it
   * arrives, so the metadata can be browsed alongside the pictures.
   */
  let details = $state(false);
  /** Position of the wipe divider, 0-100. */
  let wipe = $state(50);
  let dragging = $state(false);
  let stage = $state<HTMLDivElement | null>(null);

  /** Higher-resolution pair for this image, once it has been fetched. */
  let sharp = $state<{ path: string; original: string; developed: string } | null>(
    null,
  );

  // Fetch a larger pair for whichever image is on screen. The card-sized
  // preview stays visible until it arrives, so there is no blank frame.
  $effect(() => {
    const path = image.path;
    if (sharp?.path === path) return;

    let cancelled = false;
    buildPreview(path, FULL_PREVIEW_EDGE)
      .then((preview) => {
        if (cancelled) return;
        sharp = {
          path,
          original: preview.original,
          developed: preview.developed,
        };
      })
      .catch(() => {
        // The card preview is already on screen; nothing more to show.
      });

    return () => {
      cancelled = true;
    };
  });

  let pair = $derived(
    sharp?.path === image.path
      ? sharp
      : { original: image.original, developed: image.developed },
  );

  function setWipeFrom(clientX: number) {
    if (!stage) return;
    const box = stage.getBoundingClientRect();
    const ratio = (clientX - box.left) / box.width;
    wipe = Math.min(100, Math.max(0, ratio * 100));
  }

  function onpointerdown(event: PointerEvent) {
    if (mode !== "wipe") return;
    dragging = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    setWipeFrom(event.clientX);
  }

  function onpointermove(event: PointerEvent) {
    if (dragging) setWipeFrom(event.clientX);
  }

  function onpointerup(event: PointerEvent) {
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
  }

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
      case "Delete":
      case "Backspace":
        event.preventDefault();
        session.remove([image.path]);
        break;
    }
  }
</script>

<svelte:window on:keydown={onkeydown} />

<div class="viewer">
  <header data-tauri-drag-region>
    <!-- What changes the view, on the gutter, mirroring the toolbar's own
         left end. The switch keeps its own order: before on the left and
         after on the right is the order the wipe reveals them in. -->
    <div class="left" data-tauri-drag-region>
      <div class="modes" role="group" aria-label="Comparison mode">
        <button class:active={mode === "before"} onclick={() => (mode = "before")}>
          Before
        </button>
        <button class:active={mode === "wipe"} onclick={() => (mode = "wipe")}>
          Compare
        </button>
        <button class:active={mode === "after"} onclick={() => (mode = "after")}>
          After
        </button>
      </div>
    </div>

    <div class="title" data-tauri-drag-region>
      <span class="name" title={image.path} data-tauri-drag-region>{image.name}</span>
      <span class="position" data-tauri-drag-region>
        {index + 1} of {total}
        {#if image.width && image.height}
          <span class="dim" data-tauri-drag-region>· {image.width} × {image.height}</span>
        {/if}
      </span>
    </div>

    <div class="right" data-tauri-drag-region>
      <!-- The same three the toolbar carries, in the same order and at the
           same end of the bar, so an image's actions are where they were
           before it was opened. Details is a toggle here rather than a
           dialog, but it is still the image's properties, so it keeps its
           place. The gap to the window's own buttons is the bar's, not the
           cluster's 2px: remove must not end up flush against close. -->
      <div class="actions" data-tauri-drag-region>
        <button
          class="icon-btn"
          class:on={details}
          onclick={() => (details = !details)}
          aria-pressed={details}
          aria-label="Details"
          title="Details (I)"
        >
          <Icon name="info" />
        </button>
        <button
          class="icon-btn"
          onclick={onreveal}
          aria-label="Show in folder"
          title="Show in folder"
        >
          <Icon name="folder" />
        </button>
        <button
          class="icon-btn danger"
          onclick={() => session.remove([image.path])}
          aria-label="Remove from session"
          title="Remove from session (Del)"
        >
          <Icon name="trash" />
        </button>
      </div>

      <div class="tools" data-tauri-drag-region>
        <button
          class="icon-btn"
          onclick={() => session.closeViewer()}
          aria-label="Close viewer"
          title="Close (Esc)"
        >
          <Icon name="close" />
        </button>

        <WindowControls />
      </div>
    </div>
  </header>

  <div class="body">
    <button
      class="nav left"
      onclick={() => session.stepViewer(-1)}
      disabled={total < 2}
      aria-label="Previous image"
      title="Previous (←)"
    >
      <Icon name="chevronLeft" size={22} />
    </button>

    <div
      class="stage"
      class:grabbing={dragging}
      class:wiping={mode === "wipe"}
      bind:this={stage}
      {onpointerdown}
      {onpointermove}
      {onpointerup}
      role="presentation"
    >
      {#if pair.developed && pair.original}
        <!-- The developed frame sits underneath; the original is clipped on
             top, so the divider reveals the scan as it moves right. -->
        <img class="layer" src={pair.developed} alt={image.name} />

        {#if mode !== "after"}
          <div
            class="layer clip"
            style:clip-path={mode === "before"
              ? "none"
              : `inset(0 ${100 - wipe}% 0 0)`}
          >
            <img src={pair.original} alt="{image.name}, before developing" />
          </div>
        {/if}

        {#if mode === "wipe"}
          <div class="divider" style:left="{wipe}%">
            <span class="handle">
              <Icon name="compare" size={14} />
            </span>
          </div>
        {/if}

        <div class="labels">
          {#if mode !== "after"}<span class="tag left-tag">Before</span>{/if}
          {#if mode !== "before"}<span class="tag right-tag">After</span>{/if}
        </div>
      {:else if image.previewStatus === "error"}
        <div class="message">
          <Icon name="alert" size={26} />
          <p>This file could not be read.</p>
        </div>
      {:else}
        <div class="message">
          <span class="spinner"></span>
          <p>Reading the scan…</p>
        </div>
      {/if}
    </div>

    <button
      class="nav right"
      onclick={() => session.stepViewer(1)}
      disabled={total < 2}
      aria-label="Next image"
      title="Next (→)"
    >
      <Icon name="chevronRight" size={22} />
    </button>

    {#if details}
      <aside class="details" aria-label="Details">
        <div class="details-head">
          <h2>Details</h2>
          <button
            class="icon-btn small"
            onclick={() => (details = false)}
            aria-label="Hide details"
            title="Hide details (I)"
          >
            <Icon name="close" size={15} />
          </button>
        </div>

        <div class="details-body">
          <!-- The name is already in the titlebar above, so the panel opens
               on the status and goes straight into the file's properties. -->
          <StatusChip {image} />
          <ImageFacts {image} dense />
        </div>
      </aside>
    {/if}
  </div>

  <footer>
    <kbd>←</kbd><kbd>→</kbd> browse
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
  .viewer {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    animation: fade 0.13s ease;
  }

  header {
    display: flex;
    align-items: center;
    gap: 16px;
    flex: none;
    height: var(--header-height);
    /* The mode switch is a filled pill, so its own edge is the ink and it
       sits on the gutter exactly; the far end keeps the titlebar's 10px,
       since what ends it is the window's own buttons. */
    padding: 0 10px 0 var(--gutter);
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  /* Equal bases on the flanks put the name on the window's centre line
     rather than the centre of what is left over — the same arrangement the
     toolbar uses for Add. They are free to outgrow that share: at the
     smallest window the right end — the image's actions and the window's
     own buttons — needs more than half of it, and the name slides left
     rather than disappearing under it. */
  .left,
  .right {
    flex: 1 1 0;
  }

  .left {
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 16px;
    min-width: 0;
  }

  .title {
    /* Shrinks before the flanks do, so a long name ellipsises instead of
       pushing the buttons off the ends. */
    flex: 0 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    line-height: 1.25;
  }

  .name {
    max-width: 100%;
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .position {
    font-size: 11.5px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  .dim {
    color: var(--text-faint);
  }

  .modes {
    display: flex;
    gap: 2px;
    flex: none;
    /* Never squeezed: the name gives way first. */
    white-space: nowrap;
    padding: 2px;
    border-radius: var(--radius-sm);
    background: var(--surface-sunken);
  }

  .modes button {
    padding: 5px 13px;
    border-radius: 5px;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-muted);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .modes button:hover {
    color: var(--text);
  }

  .modes button.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow-sm);
  }

  .actions,
  .tools {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }

  /* Muted at rest like the toolbar's, and red only under the pointer: it is
     on screen for as long as the viewer is, next to nothing that undoes it. */
  .danger:hover {
    background: var(--danger-soft);
    color: var(--danger);
  }

  /* Pressed, the same way the toolbar marks its compare toggle. */
  .icon-btn.on {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 0;
    padding: 14px;
  }

  .nav {
    display: grid;
    place-items: center;
    width: 42px;
    height: 68px;
    flex: none;
    border-radius: var(--radius);
    color: var(--text-muted);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .nav:hover:not(:disabled) {
    background: var(--surface);
    color: var(--text);
  }

  .nav:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .stage {
    position: relative;
    flex: 1;
    height: 100%;
    min-width: 0;
    display: grid;
    place-items: center;
    overflow: hidden;
    border-radius: var(--radius);
    background: var(--surface-sunken);
    /* A checkerboard reads as "nothing here" behind a letterboxed photo. */
    background-image:
      linear-gradient(45deg, var(--border) 25%, transparent 25%),
      linear-gradient(-45deg, var(--border) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, var(--border) 75%),
      linear-gradient(-45deg, transparent 75%, var(--border) 75%);
    background-size: 18px 18px;
    background-position:
      0 0,
      0 9px,
      9px -9px,
      -9px 0;
  }

  .stage.wiping {
    cursor: ew-resize;
  }

  .stage.grabbing {
    cursor: grabbing;
  }

  .layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  /* The clipped layer has to land on exactly the same pixels as the layer
     underneath, so its image is positioned the same way rather than laid
     out as a child -- a grid context resolves `height: 100%` against the
     row instead of the stage, and the two frames drift apart. */
  .clip img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .divider {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    margin-left: -1px;
    background: #fff;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
    pointer-events: none;
  }

  .handle {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: #fff;
    color: #1a1a1a;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  }

  .labels {
    position: absolute;
    inset: auto 0 12px;
    display: flex;
    justify-content: space-between;
    padding: 0 12px;
    pointer-events: none;
  }

  .tag {
    padding: 3px 9px;
    border-radius: 99px;
    background: rgba(0, 0, 0, 0.62);
    color: #fff;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .right-tag {
    margin-left: auto;
  }

  .details {
    display: flex;
    flex-direction: column;
    width: 272px;
    /* The nav buttons hug the stage; the panel is its own thing. */
    margin-left: 10px;
    height: 100%;
    flex: none;
    overflow: hidden;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
  }

  .details-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex: none;
    padding: 8px 8px 8px 14px;
    border-bottom: 1px solid var(--border);
  }

  .details-head h2 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .icon-btn.small {
    width: 26px;
    height: 26px;
  }

  /* The panel scrolls on its own: a long output path must not push the
     picture around. */
  .details-body {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    padding: 14px;
  }

  .message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    color: var(--text-faint);
    font-size: 13px;
  }

  .message p {
    margin: 0;
  }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    animation: spin 0.7s linear infinite;
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

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  @media (max-width: 900px) {
    /* Narrow windows cannot spare 272px beside the picture. */
    .details {
      width: 232px;
    }
  }

  @media (max-width: 720px) {
    footer {
      display: none;
    }

    .nav {
      width: 34px;
    }
  }
</style>
