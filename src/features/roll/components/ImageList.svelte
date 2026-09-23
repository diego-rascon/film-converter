<script lang="ts">
  import ImageRow from "./ImageRow.svelte";
  import ViewHeader from "./ViewHeader.svelte";
  import { session } from "$state/session.svelte";
  import type { RollViewProps } from "../types";
  import { deselectOnBackdrop, tileCallbacks } from "../utils/tile";

  /** List view: the header and the rows, in one scroller. */
  let { showOriginal, ...handlers }: RollViewProps = $props();
</script>

<!-- The rows are full-bleed and butt up against each other, so the backdrop
     is what is left below the last of them. -->
<div class="scroll" data-backdrop onclick={deselectOnBackdrop} role="presentation">
  <div class="list" data-backdrop>
    <ViewHeader />
    {#each session.images as image, index (image.path)}
      <ImageRow
        {image}
        striped={index % 2 === 1}
        anySelected={session.hasSelection}
        {showOriginal}
        {...tileCallbacks(image.path, handlers)}
      />
    {/each}
  </div>
</div>

<style>
  .list {
    /* Column widths, read by both `ViewHeader` and `ImageRow` with a
       fallback of the same value, so the two stay on one grid. A column
       added to one needs the same slot in the other. */
    --col-thumb: 52px;
    --col-meta: 92px;
    --col-status: 104px;
    --col-action: 28px;

    display: flex;
    flex-direction: column;
    /* No gutter and no gap: the rows run edge to edge and butt up against
       each other so the zebra stripes read as continuous bands. Each row
       carries the gutter as its own padding instead. The bottom is the one
       exception, and it is the grid's, so the last row ends the same distance
       above the status bar in either view. */
    padding: 0 0 var(--gutter);
  }
</style>
