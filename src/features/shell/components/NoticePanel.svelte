<script lang="ts" module>
  import type { IconName } from "$components/Icon.svelte";
  import type { NoticeKind } from "$state/notices.svelte";

  /** How long a notice that asks nothing of the user stays up. */
  const DISMISS_MS = 4500;

  const GLYPHS: Record<NoticeKind, IconName> = {
    error: "alert",
    success: "check",
    info: "info",
  };
</script>

<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import { counted } from "$utils/format";
  import { batch } from "$state/batch.svelte";
  import { notices } from "$state/notices.svelte";

  let notice = $derived(notices.current);
  let failures = $derived(batch.failures);
  let shown = $derived(notice !== null || failures.length > 0);

  /**
   * Whether the failure list is unfolded. The button sets it; a new list —
   * or none — starts folded up again, which is why it is derived from the
   * failures rather than state an effect has to reset.
   */
  let detailsOpen = $derived.by(() => {
    void failures;
    return false;
  });

  /**
   * Errors and runs with failures wait to be dismissed — they are the ones
   * with something to read. Everything else clears itself so the panel does
   * not sit on top of the images, and a run in flight keeps its notice for
   * as long as it lasts.
   */
  $effect(() => {
    const current = notice;
    if (!current || current.kind === "error") return;
    if (failures.length > 0 || batch.running) return;

    const timer = setTimeout(() => notices.clear(current), DISMISS_MS);
    return () => clearTimeout(timer);
  });

  function dismiss() {
    notices.clear();
    batch.dismissFailures();
  }
</script>

{#if shown}
  <!-- A dock the panel is centred in, rather than the panel centring itself
       with a transform: that transform would otherwise have to be restated
       inside its entry animation, which is the shared one. -->
  <div class="dock">
    <div class="panel" role="status" aria-live="polite">
      <div class="message">
        {#if notice}
          <span class="glyph {notice.kind}">
            <Icon name={GLYPHS[notice.kind]} size={15} />
          </span>
          <span class="text">{notice.text}</span>
        {:else}
          <span class="glyph error"><Icon name="alert" size={15} /></span>
          <span class="text">{counted(failures.length, "image")} failed</span>
        {/if}

        {#if failures.length > 0}
          <button
            class="btn btn-ghost small"
            aria-expanded={detailsOpen}
            onclick={() => (detailsOpen = !detailsOpen)}
          >
            Details
            <Icon name={detailsOpen ? "chevronDown" : "chevronUp"} size={13} />
          </button>
        {/if}

        <button class="icon-btn small" onclick={dismiss} aria-label="Dismiss">
          <Icon name="close" size={14} />
        </button>
      </div>

      {#if detailsOpen && failures.length > 0}
        <ul class="failures">
          {#each failures as failure (failure.path)}
            <li>
              <span class="failure-name">{failure.name}</span>
              <span class="failure-error">{failure.error}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dock {
    position: absolute;
    left: 0;
    right: 0;
    /* Floats on the same line everything else is held off the window by; it
       is the last thing above the status bar, so a stray inset shows. */
    bottom: var(--gutter);
    padding: 0 var(--gutter);
    z-index: 30;
    display: flex;
    justify-content: center;
    /* The dock spans the window; only the panel in it is a target. */
    pointer-events: none;
  }

  .panel {
    pointer-events: auto;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    animation: rise-in 0.16s ease;
  }

  .message {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    padding: 8px 8px 8px 14px;
  }

  .glyph {
    display: flex;
    flex: none;
    color: var(--text-muted);
  }

  .glyph.error {
    color: var(--danger);
  }

  .glyph.success {
    color: var(--success);
  }

  .text {
    min-width: 0;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .failures {
    margin: 0;
    padding: 6px;
    list-style: none;
    max-height: 220px;
    overflow-y: auto;
    overscroll-behavior: contain;
    border-top: 1px solid var(--border);
  }

  .failures li {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
  }

  .failures li:hover {
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
</style>
