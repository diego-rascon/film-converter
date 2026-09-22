<script lang="ts">
  import ImageRow from "./ImageRow.svelte";
  import ViewHeader from "./ViewHeader.svelte";
  import { session } from "$lib/session.svelte";
  import type { ImageTileProps } from "./tile";

  /** List view: the header and the rows, in one scroller. */
  interface Props {
    showOriginal: boolean;
    onopen: (path: string) => void;
    oninfo: (path: string) => void;
    onreveal: (path: string) => void;
  }

  let { showOriginal, onopen, oninfo, onreveal }: Props = $props();

  function row(path: string): Omit<ImageTileProps, "image" | "anySelected" | "showOriginal"> {
    return {
      onopen: () => onopen(path),
      ontoggleSelect: (event) => {
        event.stopPropagation();
        session.pick(path, event.shiftKey);
      },
      oninfo: () => oninfo(path),
      onreveal: () => onreveal(path),
      onremove: () => session.remove([path]),
    };
  }
</script>

<div class="scroll">
  <div class="list">
    <ViewHeader />
    {#each session.images as image, index (image.path)}
      <ImageRow
        {image}
        striped={index % 2 === 1}
        anySelected={session.hasSelection}
        {showOriginal}
        {...row(image.path)}
      />
    {/each}
  </div>
</div>

<style>
  .scroll {
    flex: 1;
    overflow-y: auto;
    overscroll-behavior: contain;
  }

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
