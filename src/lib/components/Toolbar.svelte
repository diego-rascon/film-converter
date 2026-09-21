<script lang="ts">
  import Icon from "./Icon.svelte";
  import { session, plural } from "$lib/session.svelte";
  import { settings } from "$lib/settings.svelte";

  interface Props {
    showOriginal: boolean;
    ontoggleCompare: () => void;
    /** Properties of the one selected image. */
    oninfo: () => void;
    /** Shows every selected image in the file manager. */
    onreveal: () => void;
  }

  let {
    showOriginal,
    ontoggleCompare,
    oninfo,
    onreveal,
  }: Props = $props();

  let selectedCount = $derived(session.selected.length);
  /** The properties dialog is about one file, so it needs exactly one. */
  let single = $derived(selectedCount === 1);

  let subject = $derived(
    `${selectedCount} ${plural(selectedCount, "image")}`,
  );
</script>

<div class="toolbar">
  <!-- How the images are shown, mirrored from the right end it used to sit
       at: the view switch is on the gutter and stays there, so the thumbnail
       slider coming and going with grid view moves only itself. -->
  <div class="left">
    <div class="segmented">
      <button
        class:active={settings.viewMode === "grid"}
        onclick={() => {
          settings.viewMode = "grid";
          settings.save();
        }}
        aria-label="Grid view"
        title="Grid view"
      >
        <Icon name="grid" size={15} />
      </button>
      <button
        class:active={settings.viewMode === "list"}
        onclick={() => {
          settings.viewMode = "list";
          settings.save();
        }}
        aria-label="List view"
        title="List view"
      >
        <Icon name="list" size={15} />
      </button>
    </div>

    {#if settings.viewMode === "grid"}
      <label class="size">
        <span class="sr-only">Thumbnail size</span>
        <Icon name="image" size={13} />
        <input
          type="range"
          min="140"
          max="420"
          step="10"
          bind:value={settings.cardSize}
          onchange={() => settings.save()}
        />
      </label>
    {/if}

    <button
      class="btn btn-ghost"
      class:on={showOriginal}
      onclick={ontoggleCompare}
      title="Toggle between the scans and the developed results"
    >
      <Icon name="compare" size={15} />
      {showOriginal ? "Before" : "After"}
    </button>
  </div>

  <!-- The same three actions an image's own menu carries, aimed at the
       selection. They stay put and go dim rather than appearing with a
       selection, so nothing in the toolbar shifts under the pointer. -->
  <div class="right">
    <button
      class="icon-btn"
      disabled={!single}
      onclick={oninfo}
      aria-label="Properties"
      title={single
        ? "Properties"
        : "Select one image to see its properties"}
    >
      <Icon name="info" size={16} />
    </button>

    <button
      class="icon-btn"
      disabled={selectedCount === 0}
      onclick={onreveal}
      aria-label="Show in folder"
      title={selectedCount === 0
        ? "Select images to show them in the file manager"
        : `Show ${subject} in the file manager`}
    >
      <Icon name="folder" size={16} />
    </button>

    <button
      class="icon-btn danger"
      disabled={selectedCount === 0}
      onclick={() => session.removeSelected()}
      aria-label="Remove"
      title={selectedCount === 0
        ? "Select images to remove them"
        : `Remove ${subject} from the session`}
    >
      <Icon name="trash" size={16} />
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: none;
    /* The gutter lives on the bar, the way the titlebar's and the status
       bar's do, rather than on the cluster inside it: a cluster padding
       stacks with the bar's own and the controls end up a full bar padding
       inside the line everything else is on. The view switch is a filled
       pill, so its own edge is the ink and it sits on the gutter exactly.
       The far end is a 16px glyph centred in a 34px box, which hangs 9px
       past its ink, so that end pays 9px to leave the same margin. */
    padding: 7px calc(var(--gutter) - 9px) 7px var(--gutter);
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .left,
  .right {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .left,
  .right {
    flex: 1 1 0;
  }

  .right {
    justify-content: flex-end;
  }

  /* Muted at rest like its neighbours — a toolbar that is always on screen
     should not carry a red button — and red only under the pointer. */
  .danger:hover:not(:disabled) {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .btn-ghost.on {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .size {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 0 10px 0 8px;
    color: var(--text-faint);
  }

  .size input {
    width: 84px;
  }

  .segmented {
    display: flex;
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius-sm);
    background: var(--surface-sunken);
  }

  .segmented button {
    display: grid;
    place-items: center;
    width: 30px;
    height: 28px;
    border-radius: 5px;
    color: var(--text-faint);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .segmented button:hover {
    color: var(--text);
  }

  .segmented button.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow-sm);
  }

  /* Below this the clusters need their own lines. */
  @media (max-width: 860px) {
    .toolbar {
      flex-wrap: wrap;
    }

    .left,
    .right {
      flex: 0 1 auto;
    }

    /* Without the equal bases nothing pushes the selection cluster over, so
       it would trail the view controls or start the wrapped line on the left. */
    .right {
      margin-left: auto;
    }

    .size {
      display: none;
    }
  }
</style>
