<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { openPath } from "@tauri-apps/plugin-opener";

  import Icon from "./Icon.svelte";
  import { session, plural } from "$lib/session.svelte";
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
    const chosen = await open({
      directory: true,
      multiple: false,
      title: "Save developed images to",
      defaultPath: settings.directory || undefined,
    });
    if (typeof chosen !== "string") return;

    settings.directory = chosen;
    settings.save();
    await session.develop();
  }

  async function revealOutput() {
    try {
      await openPath(settings.directory);
    } catch {
      session.notice = {
        kind: "error",
        text: "Could not open the output folder",
      };
    }
  }
</script>

<footer>
  {#if session.developing}
    <div class="track" aria-hidden="true">
      <div class="fill" style:width="{progress}%"></div>
    </div>
  {/if}

  {#if session.total > 0}
    <span class="count">{session.total} {plural(session.total, "image")}</span>
  {/if}

  <div class="right">
    {#if session.developed > 0 && !session.developing}
      <button class="btn btn-ghost" onclick={revealOutput}>
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
    padding: 0 12px 0 var(--gutter);
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

  .save {
    height: 36px;
    padding: 0 20px;
    font-weight: 600;
  }

  .badge {
    display: inline-grid;
    place-items: center;
    min-width: 19px;
    height: 19px;
    padding: 0 5px;
    border-radius: 99px;
    background: rgba(0, 0, 0, 0.18);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }
</style>
