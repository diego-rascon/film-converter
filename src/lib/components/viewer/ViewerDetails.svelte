<script lang="ts">
  import Icon from "../Icon.svelte";
  import ImageFacts from "../ImageFacts.svelte";
  import StatusChip from "../StatusChip.svelte";
  import type { ImageItem } from "$lib/types";

  /**
   * A sidebar rather than a dialog: it stays open while the arrows move
   * through the roll, `Escape` still closes the viewer, and the stage
   * narrows rather than the panel floating over it, so the wipe divider
   * stays reachable.
   */
  interface Props {
    image: ImageItem;
    onclose: () => void;
  }

  let { image, onclose }: Props = $props();
</script>

<aside aria-label="Details">
  <div class="head">
    <h2 class="caps">Details</h2>
    <button
      class="icon-btn small"
      onclick={onclose}
      aria-label="Hide details"
      title="Hide details (I)"
    >
      <Icon name="close" size={15} />
    </button>
  </div>

  <div class="body">
    <!-- The name is already in the titlebar above, so the panel opens on the
         status and goes straight into the file's properties. -->
    <StatusChip {image} />
    <ImageFacts {image} dense />
  </div>
</aside>

<style>
  aside {
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

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex: none;
    padding: 8px 8px 8px 14px;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0;
  }

  /* Scrolls on its own: a long output path must not push the picture around. */
  .body {
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

  /* Narrow windows cannot spare 272px beside the picture. */
  @media (max-width: 900px) {
    aside {
      width: 232px;
    }
  }
</style>
