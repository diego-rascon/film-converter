<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { openPath } from "@tauri-apps/plugin-opener";

  import Icon from "./Icon.svelte";
  import { session, plural } from "$lib/session.svelte";
  import { settings } from "$lib/settings.svelte";

  let failuresOpen = $state(false);

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

  <div class="left">
    <span class="tally">
      {session.developed} out of {session.total}
      {plural(session.total, "image")} developed
    </span>

    {#if session.notice}
      <span class="notice {session.notice.kind}">
        {#if session.notice.kind === "error"}
          <Icon name="alert" size={13} />
        {:else if session.notice.kind === "success"}
          <Icon name="check" size={13} />
        {/if}
        {session.notice.text}
      </span>
    {/if}

    {#if session.failures.length > 0}
      <div class="failures">
        <button
          class="btn btn-ghost btn-danger small"
          onclick={() => (failuresOpen = !failuresOpen)}
        >
          Details
        </button>
        {#if failuresOpen}
          <div class="failure-list">
            <header>
              <span>
                {session.failures.length}
                {plural(session.failures.length, "failure")}
              </span>
              <button
                class="icon-btn small"
                onclick={() => (failuresOpen = false)}
                aria-label="Close"
              >
                <Icon name="close" size={14} />
              </button>
            </header>
            <ul>
              {#each session.failures as failure (failure.path)}
                <li>
                  <span class="failure-name">{failure.name}</span>
                  <span class="failure-error">{failure.error}</span>
                </li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>
    {/if}
  </div>

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
    justify-content: space-between;
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

  .left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .tally {
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .notice {
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .notice.error {
    color: var(--danger);
  }

  .notice.success {
    color: var(--success);
  }

  .right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
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

  .btn.small {
    height: 26px;
    padding: 0 9px;
    font-size: 12px;
  }

  .icon-btn.small {
    width: 24px;
    height: 24px;
  }

  .failures {
    position: relative;
  }

  .failure-list {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    z-index: 30;
    width: min(420px, 60vw);
    max-height: 260px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
  }

  .failure-list header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 8px 14px;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .failure-list ul {
    margin: 0;
    padding: 6px;
    list-style: none;
    overflow-y: auto;
  }

  .failure-list li {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
  }

  .failure-list li:hover {
    background: var(--surface-2);
  }

  .failure-name {
    font-size: 12.5px;
    font-weight: 500;
  }

  .failure-error {
    font-size: 11.5px;
    color: var(--danger);
  }

  @media (max-width: 720px) {
    .notice {
      display: none;
    }
  }
</style>
