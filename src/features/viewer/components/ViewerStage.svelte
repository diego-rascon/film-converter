<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import ViewerPod from "./ViewerPod.svelte";
  import { session } from "$state/session.svelte";
  import { ZoomPan } from "../state/zoom.svelte";
  import type { ImageItem } from "$types";
  import type { Mode } from "../types";

  /**
   * The picture itself: the pair, the wipe between them, and the two pods
   * floating in the bottom corners. Everything here works in the stage's own
   * coordinates, which is why the body's padding and not the stage's is what
   * holds the picture off the window — padding here would leave the picture's
   * box and the box `clamp`, the wipe and the wheel all measure two different
   * rectangles, and the divider would drift from the pointer.
   */
  interface Props {
    image: ImageItem;
    /** The pair to show, at whatever resolution has arrived so far. */
    pair: { original?: string; developed?: string };
    ready: boolean;
    mode: Mode;
    total: number;
    zoom: ZoomPan;
  }

  let { image, pair, ready, mode, total, zoom }: Props = $props();

  /** Position of the wipe divider, 0-100. */
  let wipe = $state(50);

  /**
   * A drag is either the wipe or a pan, decided once on pointerdown: zoomed
   * in, the stage is a thing you move; fitted, it is a thing you wipe.
   */
  let drag = $state<"wipe" | "pan" | null>(null);

  // The stage changes size when the details panel opens or the window is
  // resized, which can leave a zoomed picture outside the bounds it was
  // clamped to. Re-clamping keeps it from being stuck there.
  $effect(() => {
    const stage = zoom.stage;
    if (!stage) return;
    const observer = new ResizeObserver(() => zoom.clamp());
    observer.observe(stage);
    return () => observer.disconnect();
  });

  function setWipeFrom(clientX: number) {
    const box = zoom.stage?.getBoundingClientRect();
    if (!box) return;
    const ratio = (clientX - box.left) / box.width;
    wipe = Math.min(100, Math.max(0, ratio * 100));
  }

  function onpointerdown(event: PointerEvent) {
    const target = event.target as Element;
    // A press that lands in a floating pod is that button's, not the
    // stage's: without this it would also start a wipe or a pan underneath.
    // Testing the ancestor covers the gaps between the buttons as well as
    // the buttons, which five `stopPropagation`s would not.
    if (target.closest(".pod")) return;

    // Zoomed in the stage is a thing you move rather than a thing you wipe
    // — except on the divider itself, which keeps its grip so a comparison
    // can still be moved without first zooming back out.
    if (mode === "wipe" && (!zoom.zoomed || target.closest(".divider"))) {
      drag = "wipe";
      setWipeFrom(event.clientX);
    } else if (zoom.zoomed) {
      drag = "pan";
      zoom.startPan(event.clientX, event.clientY);
    } else {
      return;
    }

    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onpointermove(event: PointerEvent) {
    if (drag === "wipe") setWipeFrom(event.clientX);
    else if (drag === "pan") zoom.panTo(event.clientX, event.clientY);
  }

  function onpointerup(event: PointerEvent) {
    drag = null;
    zoom.endPan();
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
  }

  function onwheel(event: WheelEvent) {
    if (!ready) return;
    // The stage never scrolls, so the wheel is free to mean zoom outright
    // rather than waiting on a modifier.
    event.preventDefault();
    zoom.wheel(event.deltaY, event.clientX, event.clientY);
  }
</script>

<div
  class="stage"
  class:wiping={mode === "wipe" && !zoom.zoomed}
  class:pannable={zoom.zoomed}
  class:panning={drag === "pan"}
  style:--zoom={zoom.zoom}
  style:--pan-x="{zoom.x}px"
  style:--pan-y="{zoom.y}px"
  bind:this={zoom.stage}
  {onpointerdown}
  {onpointermove}
  {onpointerup}
  {onwheel}
  role="presentation"
>
  {#if ready}
    <!-- The developed frame sits underneath; the original is clipped on top,
         so the divider reveals the scan as it moves right. The clip is
         applied outside the zoom rather than in it, so the divider stays a
         2px line on the stage however far in the picture is pushed. -->
    <div class="canvas">
      <img class="layer" src={pair.developed} alt={image.name} />
    </div>

    {#if mode !== "after"}
      <div
        class="clip"
        style:clip-path={mode === "before" ? "none" : `inset(0 ${100 - wipe}% 0 0)`}
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

    <!-- Along the top, because the pods have the bottom corners. -->
    <div class="labels">
      {#if mode !== "after"}<span class="tag">Before</span>{/if}
      {#if mode !== "before"}<span class="tag after">After</span>{/if}
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

  <!-- Both pods float in the stage's bottom corners rather than flanking it,
       so the picture gets the body's whole width and the controls are where
       the eye already is. They sit outside `.canvas`, so zooming moves the
       picture under them and not them with it. -->
  {#if total > 1}
    <ViewerPod side="left" label="Browse">
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
    </ViewerPod>
  {/if}

  {#if ready}
    <ViewerPod side="right" label="Zoom">
      <button
        onclick={() => zoom.by(1 / ZoomPan.STEP)}
        disabled={!zoom.zoomed}
        aria-label="Zoom out"
        title="Zoom out (−)"
      >
        <Icon name="minus" size={18} />
      </button>
      <button
        onclick={() => zoom.by(ZoomPan.STEP)}
        disabled={zoom.zoom >= ZoomPan.MAX}
        aria-label="Zoom in"
        title="Zoom in (+)"
      >
        <Icon name="plus" size={18} />
      </button>
      <button
        onclick={() => zoom.reset()}
        disabled={!zoom.zoomed}
        aria-label="Fit to window"
        title="Fit to window (0)"
      >
        <Icon name="fit" size={17} />
      </button>
    </ViewerPod>
  {/if}
</div>

<style>
  .stage {
    position: relative;
    flex: 1;
    height: 100%;
    min-width: 0;
    display: grid;
    place-items: center;
    /* Clipped, so a zoomed picture stops at the header and at the details
       panel rather than spilling over them. No background and no radius:
       the picture floats on the viewer's own blurred backdrop, so the
       letterboxing around it is the blur itself and there is no box for the
       eye to find an edge on. */
    overflow: hidden;
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
     stays a grip rather than somewhere the picture pans from. */
  .stage.pannable .divider {
    pointer-events: auto;
    cursor: ew-resize;
  }

  /* Zoom and pan live here rather than on the pictures themselves, so the
     wipe's clip and the divider keep working in the stage's own
     coordinates. Both copies carry the same transform, which is why it
     reads the three custom properties off the stage rather than being
     written twice. */
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
     underneath, so its image is positioned the same way rather than laid out
     as a child — a grid context resolves `height: 100%` against the row
     instead of the stage, and the two frames drift apart. */
  .layer,
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

  /* Holds the right corner when it is the only tag on the bar. */
  .tag.after {
    margin-left: auto;
  }

  /* The stage's controls sit on a dark field in both themes, so they are
     light-on-dark rather than themed greys. */
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
</style>
