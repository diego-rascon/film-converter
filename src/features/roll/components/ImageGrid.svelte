<script lang="ts">
  import ImageCard from "./ImageCard.svelte";
  import ViewHeader from "./ViewHeader.svelte";
  import { session } from "$state/session.svelte";
  import { settings } from "$state/settings.svelte";
  import { tileCallbacks } from "../utils/tile";

  /** Grid view: the header and the cards, in one scroller. */
  interface Props {
    showOriginal: boolean;
    /** Per-image callbacks, passed straight through to each card. */
    onopen: (path: string) => void;
    oninfo: (path: string) => void;
    onreveal: (path: string) => void;
  }

  let { showOriginal, onopen, oninfo, onreveal }: Props = $props();
</script>

<!-- Clicking the backdrop itself — not a card, not the header — clears the
     selection. The header is inside the scroller so it can be sticky, which
     is why this tests the target rather than having children stop the
     event on its way up. -->
<div
  class="scroll"
  onclick={(event) => {
    if (event.target === event.currentTarget) session.setAllSelected(false);
  }}
  role="presentation"
>
  <ViewHeader />

  <div class="grid" style:--card-size="{settings.cardSize}px">
    {#each session.images as image (image.path)}
      <ImageCard
        {image}
        anySelected={session.hasSelection}
        {showOriginal}
        {...tileCallbacks(image.path, { onopen, oninfo, onreveal })}
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

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--card-size), 1fr));
    gap: 16px;
    /* The gutter on every side, not just the two the cards line up on: the
       last row ends the same distance above the status bar as the first card
       sits in from the window's left. The 16px gap between cards is a
       separate measure and stays tighter than the margin around them. */
    padding: var(--gutter);
    align-content: start;
  }
</style>
