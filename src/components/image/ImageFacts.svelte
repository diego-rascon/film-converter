<script lang="ts">
  import { imageMetadata } from "$lib/api";
  import { describeError, formatBytes } from "$utils/format";
  import type { ImageItem, ImageMetadata } from "$types";

  /**
   * One file's properties as a definition list. It is the body of the
   * properties dialog and of the viewer's details panel, so the two always
   * say the same things about a file.
   */
  interface Props {
    image: ImageItem;
    /** Labels above their values, for the viewer's narrow side panel. */
    dense?: boolean;
  }

  let { image, dense = false }: Props = $props();

  /**
   * Headers already read, by path. The viewer's panel asks again on every
   * arrow press and a file's header does not change under an open session,
   * so stepping back to an image answers from here instead of blinking
   * through the waiting line again. Per instance, so it is discarded with
   * the panel rather than growing for the life of the app.
   */
  const known = new Map<string, ImageMetadata>();

  let facts = $state<ImageMetadata | null>(null);
  let error = $state<string | null>(null);

  /**
   * Read from the file rather than from the session: the header carries the
   * format, colour depth and orientation the preview never reported, and it
   * answers even for an image whose preview failed.
   */
  $effect(() => {
    const path = image.path;
    const cached = known.get(path);

    facts = cached ?? null;
    error = null;
    if (cached) return;

    let current = true;

    imageMetadata(path)
      .then((found) => {
        known.set(path, found);
        if (current) facts = found;
      })
      .catch((reason) => {
        if (current) error = describeError(reason, "Could not read this file");
      });

    return () => {
      current = false;
    };
  });

  let megapixels = $derived(
    facts ? (facts.width * facts.height) / 1_000_000 : 0,
  );

  function formatDate(millis: number): string {
    return new Date(millis).toLocaleString(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    });
  }
</script>

{#if error}
  <p class="error">{error}</p>
{:else if !facts}
  <p class="waiting">Reading the file…</p>
{:else}
  <dl class:dense>
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

<style>
  dl {
    display: grid;
    /* One column for the labels, one for the values; a long path wraps
       inside its own cell instead of widening the dialog. */
    grid-template-columns: 96px minmax(0, 1fr);
    gap: 9px 14px;
    margin: 0;
    font-size: 13px;
  }

  /* The side panel is too narrow for two columns: a label eats half the
     width and every value wraps. Stacked, the values get the whole width. */
  dl.dense {
    /* Fills its column even where the panel lays it out as a flex item. */
    width: 100%;
    grid-template-columns: minmax(0, 1fr);
    gap: 0;
    font-size: 12.5px;
  }

  dt {
    color: var(--text-faint);
  }

  dl.dense dt {
    font-size: 11px;
    margin-top: 13px;
  }

  dl.dense dt:first-child {
    margin-top: 0;
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
    font-variant-numeric: tabular-nums;
  }

  dl.dense dd {
    margin-top: 2px;
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
