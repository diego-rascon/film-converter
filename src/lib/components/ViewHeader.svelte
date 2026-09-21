<script lang="ts">
  import Icon from "./Icon.svelte";
  import { session } from "$lib/session.svelte";
  import { settings } from "$lib/settings.svelte";

  /**
   * The bar above the images: select-all on the left, and the sort controls.
   * In list view those controls double as the column headings, so they take
   * the column widths; in grid view there are no columns to head and they sit
   * next to each other as plain buttons.
   */
  let list = $derived(settings.viewMode === "list");
</script>

{#snippet sort(key: "name" | "size", label: string, variant: string)}
  <button
    class="cell {variant}"
    class:active={session.sortKey === key}
    onclick={() => session.sortBy(key)}
    title="Sort by {key === 'name' ? 'name' : 'file size'}"
  >
    <span>{label}</span>
    <!-- Fixed slot, so a column does not shift when it becomes the sorted
         one; an unsorted column shows the chevron faintly on hover. -->
    <span class="indicator" class:on={session.sortKey === key}>
      <Icon
        name={session.sortKey === key && session.sortDirection === "desc"
          ? "chevronDown"
          : "chevronUp"}
        size={12}
      />
    </span>
  </button>
{/snippet}

<div class="head">
  <input
    type="checkbox"
    checked={session.allSelected}
    indeterminate={session.partlySelected}
    onclick={() => session.setAllSelected(!session.allSelected)}
    aria-label={session.allSelected ? "Deselect all" : "Select all"}
    title={session.allSelected ? "Deselect all" : "Select all"}
  />

  {#if list}
    <div class="cols">
      <span class="thumb-slot"></span>
      {@render sort("name", "Name", "grow")}
      <span class="cell dims">Dimensions</span>
      {@render sort("size", "Size", "meta")}
    </div>

    <span class="cell status">Status</span>
    <span class="action-slot"></span>
  {:else}
    <span class="cell">Sort</span>
    {@render sort("name", "Name", "loose")}
    {@render sort("size", "Size", "loose")}
  {/if}
</div>

<style>
  .head {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    /* The same gutter in both views, so the checkbox does not move when the
       view is switched; a row carries it too, which is what puts the header
       and the rows on one column grid. The background is opaque so the
       stripes slide under it cleanly. */
    padding: 7px var(--gutter);
    background: var(--bg);
    border-bottom: 1px solid var(--border);
  }

  .cols {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .thumb-slot {
    width: var(--col-thumb, 52px);
    flex: none;
  }

  .action-slot {
    width: var(--col-action, 28px);
    flex: none;
  }

  /* Same shape as `.field-label`, which is how this app labels a group. */
  .cell {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-faint);
    white-space: nowrap;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .dims,
  .meta {
    flex: none;
    width: var(--col-meta, 92px);
  }

  /* Right-aligned over the numbers, which puts the chevron before the word. */
  .meta {
    flex-direction: row-reverse;
    justify-content: flex-start;
  }

  .dims {
    justify-content: flex-end;
  }

  .status {
    flex: none;
    width: var(--col-status, 104px);
  }

  /* Grid view: nothing below to line up with, so the buttons take a hover
     background to read as controls rather than as headings. */
  .loose {
    margin: 0 -6px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .loose:hover {
    background: var(--surface-sunken);
  }

  button.cell {
    transition: color 0.12s ease;
  }

  button.cell:hover {
    color: var(--text-muted);
  }

  button.cell.active {
    color: var(--text);
  }

  .indicator {
    display: flex;
    flex: none;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  button.cell:hover .indicator {
    opacity: 0.4;
  }

  .indicator.on {
    opacity: 1;
  }

  /* Mirrors the column the rows drop as space gets tight. */
  @media (max-width: 720px) {
    .dims {
      display: none;
    }
  }
</style>
