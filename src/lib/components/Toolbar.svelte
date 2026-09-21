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

  /** The one way in from the toolbar: images or a folder, behind one button. */
  let addOpen = $state(false);

  function choose(action: () => void) {
    addOpen = false;
    action();
  }

  /** Same popover etiquette as the header's menus: a press outside closes. */
  function onwindowpointerdown(event: PointerEvent) {
    const target = event.target as HTMLElement | null;
    if (target?.closest("[data-popover]")) return;
    addOpen = false;
  }

  function onwindowkeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !addOpen) return;
    // Swallowed here so Escape does not also clear the selection underneath.
    event.stopPropagation();
    addOpen = false;
  }
</script>

<svelte:window
  on:pointerdown={onwindowpointerdown}
  on:keydown|capture={onwindowkeydown}
/>

<div class="toolbar">
  <div class="left">
    <span class="count">
      {session.total}
      {plural(session.total, "image")}
      {#if selectedCount > 0}
        <span class="dim">· {selectedCount} selected</span>
      {/if}
    </span>

    {#if selectedCount > 0}
      <div class="divider"></div>

      <button
        class="btn btn-ghost btn-danger"
        onclick={() => session.removeSelected()}
      >
        <Icon name="trash" size={15} />
        Remove
      </button>
    {/if}
  </div>

  <div class="center">
    <div class="popover-host" data-popover>
      <button
        class="btn btn-ghost add"
        class:open={addOpen}
        onclick={() => (addOpen = !addOpen)}
        aria-expanded={addOpen}
        aria-haspopup="menu"
        title="Add scans"
      >
        <Icon name="plus" size={15} />
        Add
        <Icon name="chevronDown" size={13} />
      </button>

      {#if addOpen}
        <div class="popover menu" role="menu">
          <button role="menuitem" onclick={() => choose(onpickFiles)}>
            <Icon name="image" size={15} />
            Add images…
          </button>
          <button role="menuitem" onclick={() => choose(onpickFolder)}>
            <Icon name="folder" size={15} />
            Add folder…
          </button>
        </div>
      {/if}
    </div>
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
    gap: 12px;
    flex: none;
    padding: 7px 12px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .left,
  .center,
  .right {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  /* Equal bases on the flanks are what keep Add on the window's centre line
     rather than the centre of whatever is left over. */
  .left,
  .right {
    flex: 1 1 0;
  }

  .center {
    flex: none;
  }

  .right {
    justify-content: flex-end;
  }

  .count {
    font-size: 12.5px;
    font-weight: 500;
    white-space: nowrap;
    padding-left: calc(var(--gutter) - 12px);
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

  .add :global(svg:last-child) {
    color: var(--text-faint);
    margin-left: -2px;
  }

  .add.open {
    background: var(--surface-sunken);
  }

  .popover-host {
    position: relative;
  }

  .popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    z-index: 40;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
    animation: drop 0.12s ease;
  }

  .popover.menu {
    min-width: 176px;
    padding: 5px;
  }

  .popover.menu button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    text-align: left;
    white-space: nowrap;
    color: var(--text);
  }

  .popover.menu button:hover {
    background: var(--surface-sunken);
  }

  .popover.menu button :global(svg) {
    color: var(--text-faint);
  }

  @keyframes drop {
    from {
      opacity: 0;
      transform: translate(-50%, -4px);
    }
  }

  /* Below this the clusters need their own lines, and Add stops being
     centred on anything — it just leads the row it lands on. */
  @media (max-width: 860px) {
    .toolbar {
      flex-wrap: wrap;
    }

    .left,
    .right {
      flex: 0 1 auto;
    }

    .size {
      display: none;
    }
  }
</style>
