<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  /** Dropping the decorations also drops the compositor's resize border — GTK
      stops handling it once the window is undecorated — so the frame draws its
      own grips around the window edge. */
  const appWindow = getCurrentWindow();

  const EDGES = [
    ["n", "North"],
    ["s", "South"],
    ["e", "East"],
    ["w", "West"],
    ["nw", "NorthWest"],
    ["ne", "NorthEast"],
    ["sw", "SouthWest"],
    ["se", "SouthEast"],
  ] as const;

  function grab(event: PointerEvent, direction: (typeof EDGES)[number][1]) {
    if (event.button !== 0) return;
    event.preventDefault();
    appWindow.startResizeDragging(direction);
  }
</script>

{#each EDGES as [edge, direction] (edge)}
  <div
    class="grip {edge}"
    role="presentation"
    onpointerdown={(event) => grab(event, direction)}
  ></div>
{/each}

<style>
  /* Above the viewer (50) and the modals (60): the window stays resizable
     whatever is on screen. */
  .grip {
    position: fixed;
    z-index: 70;
  }

  .n,
  .s {
    left: 0;
    right: 0;
    height: 4px;
    cursor: ns-resize;
  }

  .e,
  .w {
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: ew-resize;
  }

  .n {
    top: 0;
  }
  .s {
    bottom: 0;
  }
  .e {
    right: 0;
  }
  .w {
    left: 0;
  }

  .nw,
  .ne,
  .sw,
  .se {
    width: 12px;
    height: 12px;
  }

  .nw {
    top: 0;
    left: 0;
    cursor: nwse-resize;
  }
  .ne {
    top: 0;
    right: 0;
    cursor: nesw-resize;
  }
  .sw {
    bottom: 0;
    left: 0;
    cursor: nesw-resize;
  }
  .se {
    bottom: 0;
    right: 0;
    cursor: nwse-resize;
  }
</style>
