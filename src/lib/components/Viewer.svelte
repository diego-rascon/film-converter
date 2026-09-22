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
    /** The grid's before/after toggle, which is the mode the viewer opens in. */
    showOriginal: boolean;
    /** Shows the image in the file manager; the page owns the failure notice. */
    onreveal: () => void;
  }

  let { image, index, total, showOriginal, onreveal }: Props = $props();

  type Mode = "before" | "wipe" | "after";
  // Opens on whichever side the grid or list was showing, so clicking an
  // image does not change what is on screen. The viewer is mounted per
  // opening, so this is read once and the switch is free to move from there.
  let mode = $state<Mode>(showOriginal ? "before" : "after");
  /**
   * The details panel is a sidebar, not a dialog: it stays open while the
   * arrows move through the roll, and `ImageFacts` re-reads each file as it
   * arrives, so the metadata can be browsed alongside the pictures.
   */
  let details = $state(false);
  /** Position of the wipe divider, 0-100. */
  let wipe = $state(50);
  let stage = $state<HTMLDivElement | null>(null);

  /**
   * Zoom is a multiple of the fitted picture, so 1 is "fit to the stage" and
   * there is nothing below it — the viewer never shows the scan smaller than
   * the room it has. The pair is fetched at `FULL_PREVIEW_EDGE`, which runs
   * out of detail somewhere around 4x on a large stage; 6 leaves a little
   * headroom for pixel-peeping without pretending there is more.
   */
  const MAX_ZOOM = 6;
  const ZOOM_STEP = 1.4;

  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let zoomed = $derived(zoom > 1);

  /**
   * A drag is either the wipe divider or a pan, decided once on pointerdown:
   * zoomed in, the stage is a thing you move; fitted, it is a thing you wipe.
   * A pan remembers where it started so the picture tracks the pointer
   * exactly rather than accumulating rounding per move.
   */
  type Drag =
    | { kind: "wipe" }
    | { kind: "pan"; fromX: number; fromY: number; panX: number; panY: number };
  let drag = $state<Drag | null>(null);

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
  let ready = $derived(Boolean(pair.developed && pair.original));

  // A new scan arrives fitted, not at wherever the last one was left.
  $effect(() => {
    void image.path;
    resetZoom();
  });

  function resetZoom() {
    zoom = 1;
    panX = 0;
    panY = 0;
  }

  /**
   * How far the picture may be pushed before its own edge crosses the
   * stage's. `object-fit: contain` means the picture is the largest box of
   * its aspect that fits, so the overflow to pan across is measured on
   * *that*, not on the stage — otherwise a letterboxed scan pans into its
   * own empty margins.
   */
  function clampPan() {
    if (!stage) return;
    const box = stage.getBoundingClientRect();
    const aspect =
      image.width && image.height
        ? image.width / image.height
        : box.width / box.height;
    const fittedWidth = Math.min(box.width, box.height * aspect);
    const fittedHeight = Math.min(box.height, box.width / aspect);
    const limitX = Math.max(0, (fittedWidth * zoom - box.width) / 2);
    const limitY = Math.max(0, (fittedHeight * zoom - box.height) / 2);
    panX = Math.min(limitX, Math.max(-limitX, panX));
    panY = Math.min(limitY, Math.max(-limitY, panY));
  }

  // The stage changes size when the details panel opens or the window is
  // resized, which can leave a zoomed picture outside the bounds it was
  // clamped to. Re-clamping on resize keeps it from being stuck there.
  $effect(() => {
    if (!stage) return;
    const observer = new ResizeObserver(() => clampPan());
    observer.observe(stage);
    return () => observer.disconnect();
  });

  /**
   * Scales by `factor` about (`atX`, `atY`), given from the stage's centre —
   * the origin the transform itself scales about. Keeping whatever sits
   * under that point where it is means the pan has to absorb the difference,
   * which is what lets the wheel zoom into the corner it is pointing at.
   */
  function zoomBy(factor: number, atX = 0, atY = 0) {
    const next = Math.min(MAX_ZOOM, Math.max(1, zoom * factor));
    if (next === zoom) return;
    const ratio = next / zoom;
    panX = atX - ratio * (atX - panX);
    panY = atY - ratio * (atY - panY);
    zoom = next;
    clampPan();
  }

  function setWipeFrom(clientX: number) {
    if (!stage) return;
    const box = stage.getBoundingClientRect();
    const ratio = (clientX - box.left) / box.width;
    wipe = Math.min(100, Math.max(0, ratio * 100));
  }

  function onpointerdown(event: PointerEvent) {
    const target = event.target as Element;
    // A press that lands in a floating pod is that button's, not the
    // stage's: without this it would also start a wipe or a pan underneath.
    if (target.closest(".pod")) return;
    // Zoomed in, the stage is a thing you move rather than a thing you wipe
    // — except on the divider itself, which keeps its grip so a comparison
    // can still be moved without first zooming back out.
    if (mode === "wipe" && (!zoomed || target.closest(".divider"))) {
      drag = { kind: "wipe" };
      setWipeFrom(event.clientX);
    } else if (zoomed) {
      drag = {
        kind: "pan",
        fromX: event.clientX,
        fromY: event.clientY,
        panX,
        panY,
      };
    } else {
      return;
    }
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onpointermove(event: PointerEvent) {
    if (!drag) return;
    if (drag.kind === "wipe") {
      setWipeFrom(event.clientX);
    } else {
      panX = drag.panX + (event.clientX - drag.fromX);
      panY = drag.panY + (event.clientY - drag.fromY);
      clampPan();
    }
  }

  function onpointerup(event: PointerEvent) {
    drag = null;
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
  }

  function onwheel(event: WheelEvent) {
    if (!stage || !ready) return;
    // The stage never scrolls, so the wheel is free to mean zoom outright
    // rather than waiting on a modifier.
    event.preventDefault();
    const box = stage.getBoundingClientRect();
    zoomBy(
      event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP,
      event.clientX - box.left - box.width / 2,
      event.clientY - box.top - box.height / 2,
    );
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
      case "+":
      case "=":
        event.preventDefault();
        zoomBy(ZOOM_STEP);
        break;
      case "-":
      case "_":
        event.preventDefault();
        zoomBy(1 / ZOOM_STEP);
        break;
      case "0":
        event.preventDefault();
        resetZoom();
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
    <div
      class="stage"
      class:wiping={mode === "wipe" && !zoomed}
      class:pannable={zoomed}
      class:panning={drag?.kind === "pan"}
      style:--zoom={zoom}
      style:--pan-x="{panX}px"
      style:--pan-y="{panY}px"
      bind:this={stage}
      {onpointerdown}
      {onpointermove}
      {onpointerup}
      {onwheel}
      role="presentation"
    >
      {#if ready}
        <!-- The developed frame sits underneath; the original is clipped on
             top, so the divider reveals the scan as it moves right. The clip
             is applied outside the zoom rather than in it, so that the
             divider stays a 2px line on the stage however far in the picture
             is pushed. -->
        <div class="canvas">
          <img class="layer" src={pair.developed} alt={image.name} />
        </div>

        {#if mode !== "after"}
          <div
            class="clip"
            style:clip-path={mode === "before"
              ? "none"
              : `inset(0 ${100 - wipe}% 0 0)`}
          >
            <div class="canvas">
              <img src={pair.original} alt="{image.name}, before developing" />
            </div>
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

      <!-- Both pods float in the stage's bottom corners rather than flanking
           it, so the picture gets the body's whole width and the controls
           are where the eye already is. They sit outside `.canvas`, so
           zooming moves the picture under them and not them with it. The
           stage's own pointerdown ignores anything that lands in one. -->
      {#if total > 1}
        <div class="pod pager">
          <button
            onclick={() => session.stepViewer(-1)}
            aria-label="Previous image"
            title="Previous (←)"
          >
            <Icon name="chevronLeft" size={20} />
          </button>
          <button
            onclick={() => session.stepViewer(1)}
            aria-label="Next image"
            title="Next (→)"
          >
            <Icon name="chevronRight" size={20} />
          </button>
        </div>
      {/if}

      {#if ready}
        <div class="pod zoomer">
          <button
            onclick={() => zoomBy(1 / ZOOM_STEP)}
            disabled={!zoomed}
            aria-label="Zoom out"
            title="Zoom out (−)"
          >
            <Icon name="minus" size={18} />
          </button>
          <button
            onclick={() => zoomBy(ZOOM_STEP)}
            disabled={zoom >= MAX_ZOOM}
            aria-label="Zoom in"
            title="Zoom in (+)"
          >
            <Icon name="plus" size={18} />
          </button>
          <button
            onclick={resetZoom}
            disabled={!zoomed}
            aria-label="Fit to window"
            title="Fit to window (0)"
          >
            <Icon name="fit" size={17} />
          </button>
        </div>
      {/if}
    </div>

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
    animation: fade 0.13s ease;
  }

  @supports (backdrop-filter: blur(1px)) or
    (-webkit-backdrop-filter: blur(1px)) {
    .viewer {
      background: var(--viewer-backdrop);
      -webkit-backdrop-filter: blur(34px) saturate(1.3);
      backdrop-filter: blur(34px) saturate(1.3);
    }
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

  /* The one inset in the viewer's body, and the gutter every floating
     control in the stage is already on: the picture clears the window by
     the same margin its own pods do, so the two line up on one frame.
     Padding here rather than on the stage keeps the stage's box and the
     picture's box the same rectangle, which is what `clampPan`, the wipe
     and the wheel's zoom origin all measure against. */
  .body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--gutter);
    min-height: 0;
    padding: var(--gutter);
  }

  .stage {
    position: relative;
    flex: 1;
    height: 100%;
    min-width: 0;
    display: grid;
    place-items: center;
    /* Still clipped, so a zoomed picture stops at the header and at the
       details panel rather than spilling over them. */
    overflow: hidden;
    /* No background and no radius. The picture floats on the viewer's own
       blurred backdrop, so the letterboxing around it is the blur itself
       and there is no box for the eye to find an edge on. */
  }

  .stage.wiping {
    cursor: ew-resize;
  }

  .stage.pannable {
    cursor: grab;
  }

  .stage.panning {
    cursor: grabbing;
  }

  /* The one part of a zoomed stage that is still the wipe's, so the handle
     stays a grip rather than becoming somewhere the picture pans from. */
  .stage.pannable .divider {
    pointer-events: auto;
    cursor: ew-resize;
  }

  /* Zoom and pan live here rather than on the pictures themselves, so the
     wipe's clip and the divider keep working in the stage's own
     coordinates. Both copies of the picture carry the same transform, which
     is why it reads the three custom properties off the stage instead of
     being written twice. */
  .canvas {
    position: absolute;
    inset: 0;
    transform: translate(var(--pan-x), var(--pan-y)) scale(var(--zoom));
    transform-origin: center;
  }

  .clip {
    position: absolute;
    inset: 0;
  }

  /* The clipped layer has to land on exactly the same pixels as the layer
     underneath, so its image is positioned the same way rather than laid
     out as a child -- a grid context resolves `height: 100%` against the
     row instead of the stage, and the two frames drift apart. */
  .layer,
  .clip img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  /* The two floating clusters. Frosted, because what is behind them is the
     photograph and no surface token can be read against that. */
  .pod {
    position: absolute;
    /* Flush to the stage's corners, which the body's padding has already
       put on `--gutter`: a pod is its own ink, so its edge lands on the
       window's margin exactly and on the picture's, and the pager lines up
       under the header's mode switch. */
    bottom: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 2px;
    /* The geometry of every other cluster of controls in the app — the mode
       switch, the toolbar's — rather than a pill: 2px of padding around
       children a step smaller than the box that holds them. */
    padding: 2px;
    border-radius: var(--radius);
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    /* A letterboxed scan leaves a pod sitting on the backdrop instead of on
       the picture, where the glass alone has nothing to darken. The hairline
       is what gives it an edge on either. */
    box-shadow:
      inset 0 0 0 1px rgba(255, 255, 255, 0.1),
      0 2px 10px rgba(0, 0, 0, 0.3);
  }

  .pager {
    left: 0;
  }

  .zoomer {
    right: 0;
  }

  /* An `.icon-btn` in all but its colours, which have to come off the glass
     rather than off the surface tokens. */
  .pod button {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    color: rgba(255, 255, 255, 0.82);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .pod button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.16);
    color: #fff;
  }

  .pod button:disabled {
    opacity: 0.35;
    cursor: default;
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

  /* Along the top: the pods have the bottom corners. */
  .labels {
    position: absolute;
    /* The pods' corners, so a tag's edge lines up with the pod below it. */
    inset: 0 0 auto;
    display: flex;
    justify-content: space-between;
    pointer-events: none;
  }

  .tag {
    padding: 3px 9px;
    border-radius: 99px;
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
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
    color: rgba(255, 255, 255, 0.55);
    font-size: 13px;
  }

  .message p {
    margin: 0;
  }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.22);
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
  }
</style>
