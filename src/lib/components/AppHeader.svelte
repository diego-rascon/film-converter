<script lang="ts">
  import Icon from "./Icon.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import WindowControls from "./WindowControls.svelte";
  import { session } from "$lib/session.svelte";

  interface Props {
    oncredits: () => void;
    onabout: () => void;
  }

  let { oncredits, onabout }: Props = $props();

  let menuOpen = $state(false);
  let settingsOpen = $state(false);

  function choose(action: () => void) {
    menuOpen = false;
    action();
  }

  /** Closes whichever popover is open when focus or the pointer leaves it. */
  function onwindowpointerdown(event: PointerEvent) {
    const target = event.target as HTMLElement | null;
    if (target?.closest("[data-popover]")) return;
    menuOpen = false;
    settingsOpen = false;
  }

  function onwindowkeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    if (menuOpen || settingsOpen) event.stopPropagation();
    menuOpen = false;
    settingsOpen = false;
  }
</script>

<svelte:window
  on:pointerdown={onwindowpointerdown}
  on:keydown|capture={onwindowkeydown}
/>

<header data-tauri-drag-region>
  <!-- The session's own reset, at the far end from the window's buttons.
       It sits in the titlebar rather than the toolbar because it acts on
       the session itself, not on a selection or a view, and it is only
       there when there is something to clear — nothing shifts when it
       comes and goes, since the title is centred on the window and the
       right cluster is pushed over by `margin-left: auto`. -->
  {#if session.total > 0}
    <button
      class="btn btn-ghost clear"
      onclick={() => session.clear()}
      disabled={session.developing}
      title={session.developing
        ? "Cancel the run before clearing the session"
        : "Remove every image and start again"}
    >
      <Icon name="reload" size={15} />
      Clear
    </button>
  {/if}

  <div class="brand">Film Converter</div>

  <div class="actions">
    <div class="popover-host" data-popover>
      <button
        class="icon-btn"
        class:active={settingsOpen}
        onclick={() => {
          settingsOpen = !settingsOpen;
          menuOpen = false;
        }}
        aria-label="Settings"
        aria-expanded={settingsOpen}
        title="Settings"
      >
        <Icon name="settings" />
      </button>
      {#if settingsOpen}
        <div class="popover wide">
          <SettingsPanel />
        </div>
      {/if}
    </div>

    <div class="popover-host" data-popover>
      <button
        class="icon-btn"
        class:active={menuOpen}
        onclick={() => {
          menuOpen = !menuOpen;
          settingsOpen = false;
        }}
        aria-label="Menu"
        aria-expanded={menuOpen}
      >
        <Icon name="menu" />
      </button>
      {#if menuOpen}
        <div class="popover menu" role="menu">
          <button role="menuitem" onclick={() => choose(oncredits)}>
            <Icon name="users" size={15} />
            Credits
          </button>
          <button role="menuitem" onclick={() => choose(onabout)}>
            <Icon name="info" size={15} />
            About
          </button>
        </div>
      {/if}
    </div>

    <WindowControls />
  </div>
</header>

<style>
  header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;
    height: var(--header-height);
    flex: none;
    padding: 0 10px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .brand {
    /* Centred on the window rather than on the space the actions leave, so
       it does not shift when a button is added to the right cluster.
       pointer-events: none lets a press on the title reach the drag region. */
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    pointer-events: none;
    max-width: 50%;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
    /* Not `justify-content: flex-end` on the header, for the status bar's
       reason: Clear is absent before any image is loaded, and the window's
       buttons must not move when it appears. */
    margin-left: auto;
  }

  .clear {
    /* The box keeps the header's 10px, which is what holds it clear of the
       resize grip, and pays the rest of the gutter as its own padding so
       the glyph still lands on the line the toolbar and the cards sit on. */
    padding: 0 10px 0 calc(var(--gutter) - 10px);
  }

  .clear :global(svg) {
    color: var(--text-faint);
  }

  .icon-btn.active {
    background: var(--surface-sunken);
    color: var(--text);
  }

  .popover-host {
    position: relative;
  }

  .popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 40;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
    animation: drop 0.12s ease;
  }

  .popover.wide {
    width: 320px;
    padding: 16px;
  }

  .popover.menu {
    min-width: 168px;
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
      transform: translateY(-4px);
    }
  }
</style>
