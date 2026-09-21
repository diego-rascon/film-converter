<script lang="ts">
  import Icon from "./Icon.svelte";
  import WindowControls from "./WindowControls.svelte";
  import { FULL_PREVIEW_EDGE, buildPreview } from "$lib/api";
  import { session } from "$lib/session.svelte";
  import type { ImageItem } from "$lib/types";

  interface Props {
    image: ImageItem;
    index: number;
    total: number;
  }

  let { image, index, total }: Props = $props();

  type Mode = "before" | "wipe" | "after";
  let mode = $state<Mode>("wipe");
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
    <div class="title" data-tauri-drag-region>
      <span class="name" title={image.path} data-tauri-drag-region>{image.name}</span>
      <span class="position" data-tauri-drag-region>
        {index + 1} of {total}
        {#if image.width && image.height}
          <span class="dim" data-tauri-drag-region>· {image.width} × {image.height}</span>
        {/if}
      </span>
    </div>

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

    <div class="tools">
      <button
        class="icon-btn"
        onclick={() => session.remove([image.path])}
        aria-label="Remove from session"
        title="Remove from session"
      >
        <Icon name="trash" />
      </button>
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
  </div>

  <footer>
    <kbd>←</kbd><kbd>→</kbd> browse
    <span class="sep">·</span>
    <kbd>B</kbd> before / after
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
    padding: 0 0 0 18px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    line-height: 1.25;
  }

  .name {
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

  .tools {
    display: flex;
    gap: 2px;
    flex: none;
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

  @media (max-width: 720px) {
    footer {
      display: none;
    }

    .nav {
      width: 34px;
    }
  }
</style>
