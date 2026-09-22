<script lang="ts">
  import Icon from "./Icon.svelte";
  import { openFolder, pickOutputFolder } from "$lib/files";
  import { counted } from "$lib/format";
  import { session } from "$lib/session.svelte";
  import { settings } from "$lib/settings.svelte";

  /** Saving acts on the selection when there is one, otherwise everything. */
  let targetCount = $derived(
    session.hasSelection ? session.selected.length : session.total,
  );

  let progress = $derived(
    session.developing && targetCount > 0
      ? (session.completed / targetCount) * 100
      : 0,
  );

  /**
   * Saving always asks where to put the files, so the destination is a
   * decision made per run rather than a setting to remember to check. The
   * last folder used seeds the dialog and is what "Show output" opens.
   */
  async function save() {
    const chosen = await pickOutputFolder(settings.directory);
    if (chosen === null) return;

    settings.directory = chosen;
    settings.save();
    await session.develop();
  }
</script>

<footer>
  {#if session.developing}
    <div class="track" aria-hidden="true">
      <div class="fill" style:width="{progress}%"></div>
    </div>
  {/if}

  {#if session.total > 0}
    <span class="count">{counted(session.total, "image")}</span>
  {/if}

  <div class="right">
    {#if session.developed > 0 && !session.developing}
      <button class="btn btn-ghost" onclick={() => openFolder(settings.directory)}>
        <Icon name="external" size={15} />
        Show output
      </button>
    {/if}

    {#if session.developing}
      <span class="counter">{session.completed} / {targetCount}</span>
      <button class="btn" onclick={() => session.cancel()}>
        <Icon name="stop" size={14} />
        Cancel
      </button>
    {:else}
      <button
        class="btn btn-primary save"
        onclick={save}
        disabled={session.total === 0}
        title="Choose a folder and write the developed images"
      >
        Save…
        {#if session.hasSelection}
          <span class="badge">{targetCount}</span>
        {/if}
      </button>
    {/if}
  </div>
</footer>

<style>
  footer {
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;
    flex: none;
    height: var(--status-height);
    /* Both ends on the gutter, and the height leaves the same above and below,
       so Save clears the window by one number on all four sides and lands
       under the drop frame's corner rather than beside it. */
    padding: 0 var(--gutter);
    background: var(--surface);
    border-top: 1px solid var(--border);
  }

  .track {
    position: absolute;
    top: -1px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--surface-sunken);
  }

  .fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.25s ease;
  }

  .count {
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    /* Not `space-between` on the footer: the count is gone before any image
       is loaded, and Save must not slide left when it is. */
    margin-left: auto;
  }

  .counter {
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
    color: var(--text-muted);
  }

  /* One height for the whole cluster, so whichever button is rightmost at the
     time — Save, or Cancel mid-run — clears the bottom of the window by the
     same gutter it clears the right by. It is also what the bar's height is
     derived from. */
  .right .btn {
    height: var(--action-height);
  }

  .save {
    position: relative;
    /* The corner bubble hangs outside the button's box. */
    overflow: visible;
    padding: 0 20px;
    font-weight: 600;
  }

  /* A bubble on the button's top-right corner rather than a child in its row,
     so the count comes and goes without changing the button's width. It hangs
     7px outside the corner, well inside the gutter, and the ring in the bar's
     own colour cuts it cleanly away from the fill beneath. */
  .badge {
    position: absolute;
    top: -7px;
    right: -7px;
    display: grid;
    place-items: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    box-sizing: border-box;
    border-radius: 99px;
    background: var(--text);
    color: var(--surface);
    box-shadow: 0 0 0 2px var(--surface);
    font-size: 11px;
    font-weight: 600;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }
</style>
