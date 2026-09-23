<script lang="ts">
  import ImageFacts from "$components/image/ImageFacts.svelte";
  import Modal from "./Modal.svelte";
  import StatusChip from "$components/image/StatusChip.svelte";
  import type { ImageItem } from "$types";

  interface Props {
    image: ImageItem;
    onclose: () => void;
  }

  let { image, onclose }: Props = $props();
</script>

<Modal title="Properties" {onclose}>
  <div class="head">
    <span class="thumb">
      {#if image.previewStatus === "ready" && image.developed}
        <img src={image.developed} alt="" decoding="async" />
      {/if}
    </span>
    <div class="title">
      <span class="name">{image.name}</span>
      <StatusChip {image} />
    </div>
  </div>

  <ImageFacts {image} />
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
</style>
