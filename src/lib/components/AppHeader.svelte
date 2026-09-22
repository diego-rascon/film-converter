<script lang="ts">
  import Icon from "./Icon.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import WindowControls from "./WindowControls.svelte";
  import { PopoverGroup, dismissOnOutside } from "$lib/popover.svelte";
  import { session } from "$lib/session.svelte";

  interface Props {
    oncredits: () => void;
    onabout: () => void;
    onpickFiles: () => void;
    onpickFolder: () => void;
  }

  let { oncredits, onabout, onpickFiles, onpickFolder }: Props = $props();

  /**
   * One name rather than three booleans: the bar's popovers are mutually
   * exclusive, and holding the open one by name makes that true by
   * construction instead of by remembering to close the other two at every
   * call site.
   */
  const popovers = new PopoverGroup<"add" | "settings" | "menu">();

  $effect(() =>
    dismissOnOutside({
      open: popovers.current !== null,
      // Any popover on the bar counts as inside: pressing one's button is
      // what swaps to it, and that press must not also be read as a
      // dismissal of the one it is replacing.
      isInside: (target) => Boolean(target?.closest("[data-popover]")),
      close: () => popovers.close(),
    }),
  );

  function choose(action: () => void) {
    popovers.close();
    action();
  }
</script>

<header data-tauri-drag-region>
  <!-- Add and Clear act on the session itself, not on a selection or a view,
       so they sit in the titlebar, and only when there is a session — nothing
       shifts when they come and go, since the title is centred on the window
       and the right cluster is pushed over by `margin-left: auto`. -->
  {#if session.total > 0}
    <div class="lead">
      <div class="popover-host" data-popover>
        <button
          class="btn btn-ghost add"
          class:open={popovers.isOpen("add")}
          onclick={() => popovers.toggle("add")}
          aria-expanded={popovers.isOpen("add")}
          aria-haspopup="menu"
          title="Add scans"
        >
          <Icon name="plus" size={15} />
          Add
          <Icon name="chevronDown" size={13} />
        </button>

        {#if popovers.isOpen("add")}
          <div class="popover menu left" role="menu">
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
    </div>
  {/if}

  <div class="brand">Film Converter</div>

  <div class="actions">
    <div class="popover-host" data-popover>
      <button
        class="icon-btn"
        class:active={popovers.isOpen("settings")}
        onclick={() => popovers.toggle("settings")}
        aria-label="Settings"
        aria-expanded={popovers.isOpen("settings")}
        title="Settings"
      >
        <Icon name="settings" />
      </button>
      {#if popovers.isOpen("settings")}
        <div class="popover wide">
          <SettingsPanel />
        </div>
      {/if}
    </div>

    <div class="popover-host" data-popover>
      <button
        class="icon-btn"
        class:active={popovers.isOpen("menu")}
        onclick={() => popovers.toggle("menu")}
        aria-label="Menu"
        aria-expanded={popovers.isOpen("menu")}
      >
        <Icon name="menu" />
      </button>
      {#if popovers.isOpen("menu")}
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
       reason: Add and Clear are absent before any image is loaded, and the window's
       buttons must not move when it appears. */
    margin-left: auto;
  }

  /* The toolbar's 4px between neighbouring buttons, not the header's 16px
     between its clusters. */
  .lead {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .add {
    /* The box keeps the header's 10px, which is what holds it clear of the
       resize grip, and pays the rest of the gutter as its own padding so
       the glyph still lands on the line the toolbar and the cards sit on. */
    padding: 0 10px 0 calc(var(--gutter) - 10px);
  }

  .clear :global(svg) {
    color: var(--text-faint);
  }

  .icon-btn.active,
  .add.open {
    background: var(--surface-sunken);
  }

  .icon-btn.active {
    color: var(--text);
  }

  .add :global(svg:last-child) {
    color: var(--text-faint);
    margin-left: -2px;
  }

  /* The shape and the drop come from `.popover` in app.css; these two are
     the only panels on the bar that need a size of their own. */
  .popover.wide {
    width: 320px;
    padding: 16px;
  }

  .popover.menu {
    min-width: 168px;
  }
</style>
