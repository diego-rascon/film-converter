<script lang="ts" module>
  import type { Mode } from "../types";

  /** Before on the left and after on the right is the order the wipe
      reveals them in, so the switch keeps it rather than the bar's. */
  const MODES: { value: Mode; label: string }[] = [
    { value: "before", label: "Before" },
    { value: "wipe", label: "Compare" },
    { value: "after", label: "After" },
  ];
</script>

<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import WindowControls from "$components/window/WindowControls.svelte";
  import { session } from "$state/session.svelte";
  import type { ImageItem } from "$types";

  /**
   * The viewer's titlebar. It splits the way the toolbar splits its bar, so
   * an image's actions are where they were before it was opened: the
   * before/after switch on the left and details, show in folder and remove
   * on the right, in the toolbar's order.
   */
  interface Props {
    image: ImageItem;
    index: number;
    total: number;
    mode: Mode;
    details: boolean;
    onmode: (mode: Mode) => void;
    ontoggleDetails: () => void;
    onreveal: () => void;
  }

  let {
    image,
    index,
    total,
    mode,
    details,
    onmode,
    ontoggleDetails,
    onreveal,
  }: Props = $props();
</script>

<header data-tauri-drag-region>
  <div class="left" data-tauri-drag-region>
    <div class="segmented" role="group" aria-label="Comparison mode">
      {#each MODES as option (option.value)}
        <button
          class:active={mode === option.value}
          onclick={() => onmode(option.value)}
        >
          {option.label}
        </button>
      {/each}
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
         same end of the bar. Details is a panel here rather than a dialog,
         but it is still the image's properties, so it keeps its place. The
         gap to the window's own buttons is the bar's 16px, not the
         cluster's 2px: remove must not end up flush against close. -->
    <div class="actions" data-tauri-drag-region>
      <button
        class="icon-btn"
        class:on={details}
        onclick={ontoggleDetails}
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

    <div class="actions" data-tauri-drag-region>
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

<style>
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
     rather than the centre of what is left over. They are free to outgrow
     that share: at the smallest window the right end needs more than half
     of it, and the name slides left rather than under it. */
  .left,
  .right {
    flex: 1 1 0;
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .right {
    justify-content: flex-end;
    gap: 16px;
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

  .position,
  .dim {
    font-size: 11.5px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  /* Words rather than glyphs, so these are padded rather than square. */
  .segmented button {
    padding: 5px 13px;
    font-size: 12.5px;
    font-weight: 500;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }

  /* Pressed, the same way the toolbar marks its compare toggle. */
  .icon-btn.on {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
