<script lang="ts">
  import ImageCard from "./ImageCard.svelte";
  import ViewHeader from "./ViewHeader.svelte";
  import { session } from "$state/session.svelte";
  import { settings } from "$state/settings.svelte";
  import type { RollViewProps } from "../types";
  import { deselectOnBackdrop, tileCallbacks } from "../utils/tile";

  /** Grid view: the header and the cards, in one scroller. */
  let { showOriginal, ...handlers }: RollViewProps = $props();
</script>

<!-- The scroller and the grid are the backdrop: a click in the gaps between
     the cards or below the last row lets the selection go. -->
<div class="scroll" data-backdrop onclick={deselectOnBackdrop} role="presentation">
  <ViewHeader />

  <div class="grid" data-backdrop style:--card-size="{settings.cardSize}px">
    {#each session.images as image (image.path)}
      <ImageCard
        {image}
        anySelected={session.hasSelection}
        {showOriginal}
        {...tileCallbacks(image.path, handlers)}
      />
    {/each}
  </div>
</div>

<style>
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
