<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import Icon from "./Icon.svelte";

  /** Stands in for the titlebar buttons the desktop would draw: the window is
      created with `decorations: false` so the header can host them instead. */
  const appWindow = getCurrentWindow();

  let maximized = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    let done = false;

    (async () => {
      maximized = await appWindow.isMaximized();
      const stop = await appWindow.onResized(async () => {
        maximized = await appWindow.isMaximized();
      });
      // The component may already be gone by the time the listener lands.
      if (done) stop();
      else unlisten = stop;
    })();

    return () => {
      done = true;
      unlisten?.();
    };
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
    class="icon-btn close"
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

  .close:hover {
    background: var(--danger-soft);
    color: var(--danger);
  }
</style>
