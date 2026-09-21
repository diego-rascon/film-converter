<script lang="ts">
  import Icon from "./Icon.svelte";
  import { session, plural } from "$lib/session.svelte";
  import { settings } from "$lib/settings.svelte";

  interface Props {
    showOriginal: boolean;
    ontoggleCompare: () => void;
    onpickFiles: () => void;
    onpickFolder: () => void;
  }

  let { showOriginal, ontoggleCompare, onpickFiles, onpickFolder }: Props =
    $props();

  let selectedCount = $derived(session.selected.length);
</script>

<div class="toolbar">
  <div class="left">
    <span class="count">
      {session.total}
      {plural(session.total, "image")}
      {#if selectedCount > 0}
        <span class="dim">· {selectedCount} selected</span>
      {/if}
    </span>

    <div class="divider"></div>

    <button class="btn btn-ghost" onclick={onpickFiles}>
      <Icon name="plus" size={15} />
      Add
    </button>
    <button class="btn btn-ghost" onclick={onpickFolder}>
      <Icon name="folder" size={15} />
      Folder
    </button>

    {#if session.total > 0}
      <button
        class="btn btn-ghost"
        onclick={() => session.setAllSelected(!session.allSelected)}
      >
        <Icon name="check" size={15} />
        {session.allSelected ? "Deselect all" : "Select all"}
      </button>
    {/if}

    {#if selectedCount > 0}
      <button
        class="btn btn-ghost btn-danger"
        onclick={() => session.removeSelected()}
      >
        <Icon name="trash" size={15} />
        Remove
      </button>
    {/if}
  </div>

  <div class="right">
    <button
      class="btn btn-ghost"
      class:on={showOriginal}
      onclick={ontoggleCompare}
      title="Toggle between the scans and the developed results"
    >
      <Icon name="compare" size={15} />
      {showOriginal ? "Before" : "After"}
    </button>

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
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex: none;
    padding: 7px 12px;
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

  .count {
    font-size: 12.5px;
    font-weight: 500;
    white-space: nowrap;
    padding-left: 4px;
  }

  .dim {
    color: var(--text-faint);
    font-weight: 400;
  }

  .divider {
    width: 1px;
    height: 20px;
    margin: 0 6px;
    background: var(--border);
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

  /* Below this the two clusters need their own lines. */
  @media (max-width: 860px) {
    .toolbar {
      flex-wrap: wrap;
    }

    .size {
      display: none;
    }
  }
</style>
