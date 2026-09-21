<script lang="ts">
  import { imageMetadata } from "$lib/api";
  import Modal from "./Modal.svelte";
  import StatusChip from "./StatusChip.svelte";
  import { formatBytes } from "$lib/session.svelte";
  import type { ImageItem, ImageMetadata } from "$lib/types";

  interface Props {
    image: ImageItem;
    onclose: () => void;
  }

  let { image, onclose }: Props = $props();

  let facts = $state<ImageMetadata | null>(null);
  let error = $state<string | null>(null);

  /**
   * Read from the file rather than from the session: the header carries the
   * format, colour depth and orientation the preview never reported, and it
   * answers even for an image whose preview failed.
   */
  $effect(() => {
    const path = image.path;
    let current = true;
    facts = null;
    error = null;

    imageMetadata(path)
      .then((found) => {
        if (current) facts = found;
      })
      .catch((reason) => {
        if (current) error = describe(reason);
      });

    return () => {
      current = false;
    };
  });

  let megapixels = $derived(
    facts ? (facts.width * facts.height) / 1_000_000 : 0,
  );

  function describe(reason: unknown): string {
    if (typeof reason === "string") return reason;
    if (reason instanceof Error) return reason.message;
    return "Could not read this file";
  }

  function formatDate(millis: number): string {
    return new Date(millis).toLocaleString(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    });
  }
</script>

<Modal title="Properties" {onclose}>
  <div class="head">
    <span class="thumb">
      {#if image.previewStatus === "ready" && image.developed}
        <img src={image.developed} alt="" />
      {/if}
    </span>
    <div class="title">
      <span class="name">{image.name}</span>
      <StatusChip {image} />
    </div>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if !facts}
    <p class="waiting">Reading the file…</p>
  {:else}
    <dl>
      <dt>Kind</dt>
      <dd>{facts.format} image</dd>

      <dt>Dimensions</dt>
      <dd>
        {facts.width} × {facts.height}
        <span class="aside">({megapixels.toFixed(1)} MP)</span>
      </dd>

      <dt>Colour</dt>
      <dd>{facts.color}</dd>

      {#if facts.orientation}
        <!-- Only worth a line when it is not upright; the scan is rotated on
             load, so this says why the preview is turned. -->
        <dt>Orientation</dt>
        <dd>{facts.orientation} <span class="aside">(applied)</span></dd>
      {/if}

      <dt>Size</dt>
      <dd>
        {formatBytes(facts.bytes)}
        <span class="aside">({facts.bytes.toLocaleString()} bytes)</span>
      </dd>

      {#if facts.modified !== null}
        <dt>Modified</dt>
        <dd>{formatDate(facts.modified)}</dd>
      {/if}

      {#if facts.created !== null}
        <dt>Created</dt>
        <dd>{formatDate(facts.created)}</dd>
      {/if}

      <dt>Where</dt>
      <dd class="path">{facts.directory}</dd>

      {#if image.output}
        <dt>Developed to</dt>
        <dd class="path">{image.output}</dd>
      {/if}

      {#if image.error}
        <dt>Error</dt>
        <dd class="failure">{image.error}</dd>
      {/if}
    </dl>
  {/if}
</Modal>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 16px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }

  .thumb {
    display: block;
    width: 56px;
    height: 56px;
    flex: none;
    border-radius: var(--radius-sm);
    overflow: hidden;
    border: 1px solid var(--border);
    background: var(--surface-sunken);
  }

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .title {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    min-width: 0;
  }

  /* Names are long and the interesting end is usually the last one, so this
     wraps rather than truncating. */
  .name {
    font-size: 14px;
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  dl {
    display: grid;
    /* One column for the labels, one for the values; a long path wraps
       inside its own cell instead of widening the dialog. */
    grid-template-columns: 96px minmax(0, 1fr);
    gap: 9px 14px;
    margin: 0;
    font-size: 13px;
  }

  dt {
    color: var(--text-faint);
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
    font-variant-numeric: tabular-nums;
  }

  .aside {
    color: var(--text-faint);
  }

  .path {
    font-size: 12px;
    color: var(--text-muted);
  }

  .failure {
    color: var(--danger);
  }

  .waiting,
  .error {
    margin: 0;
    font-size: 13px;
    color: var(--text-faint);
  }

  .error {
    color: var(--danger);
  }
</style>
