<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";

  import { startupPaths } from "$lib/api";

  import AboutModal from "$lib/components/AboutModal.svelte";
  import AppHeader from "$lib/components/AppHeader.svelte";
  import CreditsModal from "$lib/components/CreditsModal.svelte";
  import DropZone from "$lib/components/DropZone.svelte";
  import ImageCard from "$lib/components/ImageCard.svelte";
  import ImageRow from "$lib/components/ImageRow.svelte";
  import InfoModal from "$lib/components/InfoModal.svelte";
  import ViewHeader from "$lib/components/ViewHeader.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import Viewer from "$lib/components/Viewer.svelte";
  import WindowResizeEdges from "$lib/components/WindowResizeEdges.svelte";

  import { session } from "$lib/session.svelte";
  import { settings } from "$lib/settings.svelte";
  import type { BatchProgress } from "$lib/types";

  let ready = $state(false);
  let dragging = $state(false);
  let showOriginal = $state(false);
  let creditsOpen = $state(false);
  let aboutOpen = $state(false);
  /** The image whose properties are on screen, by path. */
  let infoPath = $state<string | null>(null);

  let infoImage = $derived(infoPath === null ? null : session.find(infoPath));

  /** Anchor for shift-click range selection. */
  let lastPicked: string | null = null;

  onMount(() => {
    const disposers: (() => void)[] = [];

    (async () => {
      await settings.load();
      ready = true;

      // Tauri delivers real file paths here; the browser's own drag events
      // never see them, which is why the HTML5 API is not used.
      disposers.push(
        await getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === "over") {
            dragging = true;
          } else if (event.payload.type === "drop") {
            dragging = false;
            session.add(event.payload.paths);
          } else {
            dragging = false;
          }
        }),
      );

      disposers.push(
        await listen<BatchProgress>("develop://progress", (event) => {
          session.applyProgress(event.payload);
        }),
      );

      // Scans named on the command line, e.g. opened from a file manager.
      const initial = await startupPaths();
      if (initial.length > 0) await session.add(initial);
    })();

    return () => {
      for (const dispose of disposers) dispose();
    };
  });

  async function pickFiles() {
    const chosen = await open({
      multiple: true,
      title: "Choose film scans",
      filters: [
        {
          name: "Images",
          extensions: ["jpg", "jpeg", "png", "tif", "tiff", "bmp", "webp"],
        },
      ],
    });
    if (Array.isArray(chosen)) await session.add(chosen);
    else if (typeof chosen === "string") await session.add([chosen]);
  }

  async function pickFolder() {
    const chosen = await open({
      directory: true,
      multiple: false,
      title: "Choose a folder of scans",
    });
    if (typeof chosen === "string") await session.add([chosen]);
  }

  /**
   * Opens the enclosing folder with the files picked out. A whole selection
   * goes in one call: the plugin shows them together rather than opening a
   * window each.
   */
  async function reveal(paths: string | string[]) {
    if (paths.length === 0) return;
    try {
      await revealItemInDir(paths);
    } catch {
      session.notice = {
        kind: "error",
        text: "Could not open the file manager",
      };
    }
  }

  /** Plain click picks one; shift extends from the last pick. */
  function pick(path: string, event: MouseEvent) {
    event.stopPropagation();

    if (event.shiftKey && lastPicked) {
      const from = session.images.findIndex((i) => i.path === lastPicked);
      const to = session.images.findIndex((i) => i.path === path);
      if (from >= 0 && to >= 0) {
        const [start, end] = from < to ? [from, to] : [to, from];
        const value = !session.find(path)!.selected;
        for (let i = start; i <= end; i += 1) {
          session.images[i].selected = value;
        }
        lastPicked = path;
        return;
      }
    }

    session.toggleSelected(path);
    lastPicked = path;
  }

  function onkeydown(event: KeyboardEvent) {
    // The viewer and the dialogs run their own keyboard handling.
    if (session.viewerIndex !== null) return;
    if (creditsOpen || aboutOpen || infoImage) return;

    const target = event.target as HTMLElement | null;
    if (target && /^(INPUT|SELECT|TEXTAREA)$/.test(target.tagName)) return;

    if ((event.ctrlKey || event.metaKey) && event.key === "a") {
      event.preventDefault();
      session.setAllSelected(true);
    } else if (
      (event.key === "Delete" || event.key === "Backspace") &&
      session.hasSelection
    ) {
      event.preventDefault();
      session.removeSelected();
    } else if (event.key === "Escape" && session.hasSelection) {
      session.setAllSelected(false);
    }
  }
</script>

<svelte:window on:keydown={onkeydown} />

<div class="app" class:dragging>
  <AppHeader
    oncredits={() => (creditsOpen = true)}
    onabout={() => (aboutOpen = true)}
  />

  {#if session.total > 0}
    <Toolbar
      {showOriginal}
      ontoggleCompare={() => (showOriginal = !showOriginal)}
      onpickFiles={pickFiles}
      onpickFolder={pickFolder}
      oninfo={() => (infoPath = session.selected[0]?.path ?? null)}
      onreveal={() => reveal(session.selected.map((i) => i.path))}
    />
  {/if}

  <main>
    {#if !ready}
      <div class="booting"></div>
    {:else if session.total === 0}
      <DropZone
        {dragging}
        importing={session.importing}
        onpickFiles={pickFiles}
        onpickFolder={pickFolder}
      />
    {:else if settings.viewMode === "grid"}
      <!-- Clicking the backdrop itself — not a card, not the header —
           clears the selection. -->
      <div
        class="scroll"
        onclick={(event) => {
          if (event.target === event.currentTarget) session.setAllSelected(false);
        }}
        role="presentation"
      >
        <ViewHeader />

        <div class="grid" style:--card-size="{settings.cardSize}px">
          {#each session.images as image (image.path)}
            <ImageCard
              {image}
              {showOriginal}
              onopen={() => session.openViewer(image.path)}
              ontoggleSelect={(event) => pick(image.path, event)}
              oninfo={() => (infoPath = image.path)}
              onreveal={() => reveal(image.path)}
              onremove={() => session.remove([image.path])}
            />
          {/each}
        </div>
      </div>
    {:else}
      <div class="scroll">
        <div class="list">
          <ViewHeader />
          {#each session.images as image, index (image.path)}
            <ImageRow
              {image}
              striped={index % 2 === 1}
              {showOriginal}
              onopen={() => session.openViewer(image.path)}
              ontoggleSelect={(event) => pick(image.path, event)}
              oninfo={() => (infoPath = image.path)}
              onreveal={() => reveal(image.path)}
              onremove={() => session.remove([image.path])}
            />
          {/each}
        </div>
      </div>
    {/if}

    {#if dragging && session.total > 0}
      <div class="drop-overlay">
        <span>Drop to add</span>
      </div>
    {/if}
  </main>

  <StatusBar />
</div>

{#if session.viewerImage && session.viewerIndex !== null}
  <Viewer
    image={session.viewerImage}
    index={session.viewerIndex}
    total={session.total}
  />
{/if}

{#if infoImage}
  <InfoModal image={infoImage} onclose={() => (infoPath = null)} />
{/if}

{#if creditsOpen}
  <CreditsModal onclose={() => (creditsOpen = false)} />
{/if}

{#if aboutOpen}
  <AboutModal onclose={() => (aboutOpen = false)} />
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

  .scroll {
    flex: 1;
    overflow-y: auto;
    overscroll-behavior: contain;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--card-size), 1fr));
    gap: 16px;
    padding: 16px var(--gutter);
    align-content: start;
  }

  .list {
    /* Column widths, shared by the header and the rows so the two stay on
       one grid; both read them with a fallback of the same value. */
    --col-thumb: 52px;
    --col-meta: 92px;
    --col-status: 104px;
    --col-action: 28px;

    display: flex;
    flex-direction: column;
    /* No gutter and no gap: the rows run edge to edge and butt up against
       each other so the zebra stripes read as continuous bands. Each row
       carries the gutter as its own padding instead. */
    padding: 0 0 8px;
  }

  .booting {
    flex: 1;
  }

  .drop-overlay {
    position: absolute;
    inset: 10px;
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
