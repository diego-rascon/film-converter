<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";

  import { startupPaths } from "$lib/api";
  import { pickFolder, pickImages, revealItems } from "$lib/files";
  import { listening } from "$hooks/listening";

  import AboutModal from "$features/shell/components/AboutModal.svelte";
  import AppHeader from "$features/shell/components/AppHeader.svelte";
  import CreditsModal from "$features/shell/components/CreditsModal.svelte";
  import DropZone from "$features/roll/components/DropZone.svelte";
  import ImageGrid from "$features/roll/components/ImageGrid.svelte";
  import ImageList from "$features/roll/components/ImageList.svelte";
  import InfoModal from "$components/overlay/InfoModal.svelte";
  import NoticePanel from "$features/shell/components/NoticePanel.svelte";
  import StatusBar from "$features/shell/components/StatusBar.svelte";
  import Toolbar from "$features/shell/components/Toolbar.svelte";
  import Viewer from "$features/viewer/components/Viewer.svelte";
  import WindowResizeEdges from "$components/window/WindowResizeEdges.svelte";

  import { session } from "$state/session.svelte";
  import { settings } from "$state/settings.svelte";

  /**
   * The app shell: the bars, whichever view is showing, and the overlays.
   * The two views own their own layout, the session owns the images and
   * [files.ts](src/lib/files.ts) owns the dialogs, so what is left here is
   * the wiring between them.
   */
  let dragging = $state(false);
  let showOriginal = $state(false);
  /** The About or Credits dialog, whichever is up. */
  let dialog = $state<"about" | "credits" | null>(null);
  /** The image whose properties are on screen, by path. */
  let infoPath = $state<string | null>(null);

  let infoImage = $derived(infoPath === null ? undefined : session.find(infoPath));

  /**
   * The viewer or a dialog has the window. The shell behind it goes inert —
   * out of the tab order, out of reach of the pointer — and its shortcuts
   * stand down, since whatever is on top runs its own keyboard.
   */
  let covered = $derived(session.viewer !== null || dialog !== null || infoImage !== undefined);

  /** What a card or a row reports back, the same in either view. */
  const viewCallbacks = {
    onopen: (path: string) => session.openViewer(path),
    oninfo: (path: string) => (infoPath = path),
    onreveal: (path: string) => revealItems(path),
  };

  // Tauri delivers real file paths here; the browser's own drag events
  // never see them, which is why the HTML5 API is not used.
  $effect(() =>
    listening(
      getCurrentWebview().onDragDropEvent(({ payload }) => {
        if (payload.type === "drop") session.add(payload.paths);
        dragging = payload.type === "enter" || payload.type === "over";
      }),
    ),
  );

  // Scans named on the command line, e.g. opened from a file manager.
  onMount(() => {
    startupPaths().then((paths) => session.add(paths));
  });

  /** A field typed into, where the letter and delete keys are the field's. */
  function typing(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLElement &&
      (target.isContentEditable ||
        target.matches("textarea, select, input:not([type='checkbox'], [type='range'])"))
    );
  }

  function onkeydown(event: KeyboardEvent) {
    if (covered || typing(event.target)) return;

    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
      event.preventDefault();
      session.setAllSelected(true);
    } else if ((event.key === "Delete" || event.key === "Backspace") && session.hasSelection) {
      event.preventDefault();
      session.removeSelected();
    } else if (event.key === "Escape" && session.hasSelection) {
      session.setAllSelected(false);
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="app" class:dragging inert={covered}>
  <AppHeader
    oncredits={() => (dialog = "credits")}
    onabout={() => (dialog = "about")}
    onpickFiles={pickImages}
    onpickFolder={pickFolder}
  />

  {#if session.total > 0}
    <Toolbar
      {showOriginal}
      ontoggleCompare={() => (showOriginal = !showOriginal)}
      oninfo={() => (infoPath = session.selected[0]?.path ?? null)}
      onreveal={() => revealItems(session.selected.map((i) => i.path))}
    />
  {/if}

  <main>
    {#if session.total === 0}
      <DropZone
        {dragging}
        importing={session.importing}
        onpickFiles={pickImages}
        onpickFolder={pickFolder}
      />
    {:else if settings.viewMode === "grid"}
      <ImageGrid {showOriginal} {...viewCallbacks} />
    {:else}
      <ImageList {showOriginal} {...viewCallbacks} />
    {/if}

    {#if dragging && session.total > 0}
      <div class="drop-overlay">
        <span>Drop to add</span>
      </div>
    {/if}

    <!-- Floats over the images rather than living in the status bar, so a
         message can be as long as it needs to be and a failed run can unfold
         its list under it. -->
    <NoticePanel />
  </main>

  <StatusBar />
</div>

{#if session.viewer}
  <Viewer
    image={session.viewer.image}
    index={session.viewer.index}
    total={session.total}
    {showOriginal}
    onreveal={revealItems}
  />
{/if}

{#if infoImage}
  <InfoModal image={infoImage} onclose={() => (infoPath = null)} />
{/if}

{#if dialog === "credits"}
  <CreditsModal onclose={() => (dialog = null)} />
{:else if dialog === "about"}
  <AboutModal onclose={() => (dialog = null)} />
{/if}

<WindowResizeEdges />

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  main {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .drop-overlay {
    position: absolute;
    /* The same frame the empty session's drop zone draws, on the same line:
       dragging onto a loaded window and dragging onto an empty one are the
       same offer and should not be outlined in two different places. */
    inset: var(--gutter);
    display: grid;
    place-items: center;
    border: 2px dashed var(--accent);
    border-radius: var(--radius-lg);
    background: var(--accent-soft);
    backdrop-filter: blur(2px);
    pointer-events: none;
    z-index: 20;
  }

  .drop-overlay span {
    padding: 8px 18px;
    border-radius: 99px;
    background: var(--accent);
    color: var(--accent-text);
    font-size: 13px;
    font-weight: 600;
  }
</style>
