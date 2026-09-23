<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import Icon from "$components/Icon.svelte";
  import { listening } from "$hooks/listening";

  /** Stands in for the titlebar buttons the desktop would draw: the window is
      created with `decorations: false` so the header can host them instead. */
  const appWindow = getCurrentWindow();

  let maximized = $state(false);

  // Maximising and restoring both resize the window, whichever way they were
  // done — these buttons, a double click on the titlebar, the desktop's own
  // shortcut — so a resize is the one moment to look again.
  $effect(() => {
    const sync = async () => {
      maximized = await appWindow.isMaximized();
    };
    void sync();
    return listening(appWindow.onResized(sync));
  });
</script>

<div class="controls">
  <button
    class="icon-btn"
    onclick={() => appWindow.minimize()}
    aria-label="Minimise"
    title="Minimise"
  >
    <Icon name="windowMinimize" size={16} />
  </button>
  <button
    class="icon-btn"
    onclick={() => appWindow.toggleMaximize()}
    aria-label={maximized ? "Restore" : "Maximise"}
    title={maximized ? "Restore" : "Maximise"}
  >
    <Icon name={maximized ? "windowRestore" : "windowMaximize"} size={16} />
  </button>
  <button
    class="icon-btn danger"
    onclick={() => appWindow.close()}
    aria-label="Close window"
    title="Close window"
  >
    <Icon name="close" size={17} />
  </button>
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
    margin-left: 6px;
    padding-left: 7px;
    border-left: 1px solid var(--border);
  }
</style>
