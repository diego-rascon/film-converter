<script lang="ts">
  import Icon from "./Icon.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import WindowControls from "./WindowControls.svelte";

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
  <div class="brand">
    <Icon name="film" size={19} />
    <span>Film Converter</span>
  </div>

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
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    height: var(--header-height);
    flex: none;
    padding: 0 10px 0 18px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .brand {
    /* Lets a press on the title reach the header's drag region. */
    pointer-events: none;
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  .brand :global(svg) {
    color: var(--accent);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
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
