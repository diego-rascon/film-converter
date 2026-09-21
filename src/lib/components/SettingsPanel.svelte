<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import { settings } from "$lib/settings.svelte";
  import type { OutputFormat } from "$lib/types";

  const formats: { value: OutputFormat; label: string; note: string }[] = [
    { value: "jpg", label: "JPEG", note: "4:4:4, no chroma subsampling" },
    { value: "png", label: "PNG", note: "Lossless; quality sets compression" },
    { value: "tiff", label: "TIFF", note: "Lossless, uncompressed" },
  ];

  let activeFormat = $derived(
    formats.find((f) => f.value === settings.format) ?? formats[0],
  );

  // The path box below is right-to-left so that a long path keeps its tail
  // visible. Left-to-right embedding marks hold the path itself the right
  // way round, which stops the leading slash being moved to the end.
  const LRE = String.fromCharCode(0x202a);
  const PDF = String.fromCharCode(0x202c);

  let displayPath = $derived(
    settings.directory ? LRE + settings.directory + PDF : "Not set",
  );

  async function pickFolder() {
    const chosen = await open({
      directory: true,
      multiple: false,
      title: "Choose an output folder",
      defaultPath: settings.directory || undefined,
    });
    if (typeof chosen === "string") {
      settings.directory = chosen;
      settings.save();
    }
  }
</script>

<div class="panel">
  <div class="field">
    <span class="field-label">Output folder</span>
    <div class="folder">
      <span class="path" title={settings.directory}>{displayPath}</span>
      <button class="btn" onclick={pickFolder}>
        <Icon name="folder" size={15} />
        Browse
      </button>
    </div>
  </div>

  <div class="field">
    <label class="field-label" for="format">Format</label>
    <select
      id="format"
      bind:value={settings.format}
      onchange={() => settings.save()}
    >
      {#each formats as format (format.value)}
        <option value={format.value}>{format.label}</option>
      {/each}
    </select>
    <p class="note">{activeFormat.note}</p>
  </div>

  {#if settings.qualityApplies}
    <div class="field">
      <label class="field-label" for="quality">
        {settings.format === "png" ? "Compression" : "Quality"}
        <span class="value">{settings.quality}</span>
      </label>
      <input
        id="quality"
        type="range"
        min="50"
        max="100"
        step="1"
        bind:value={settings.quality}
        onchange={() => settings.save()}
      />
    </div>
  {/if}

  <label class="toggle">
    <input
      type="checkbox"
      bind:checked={settings.overwrite}
      onchange={() => settings.save()}
    />
    <span>
      Overwrite existing files
      <em>Off: a numbered copy is written instead</em>
    </span>
  </label>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .field-label {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .value {
    font-variant-numeric: tabular-nums;
    letter-spacing: 0;
    color: var(--text);
  }

  .folder {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .path {
    flex: 1;
    min-width: 0;
    height: 34px;
    display: flex;
    align-items: center;
    padding: 0 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface-2);
    font-size: 12px;
    color: var(--text-muted);
    /* Clip the head, not the tail: the folder name matters more than the
       root. In a right-to-left box, overflow falls off the left edge. */
    direction: rtl;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .note {
    margin: 6px 0 0;
    font-size: 11.5px;
    color: var(--text-faint);
  }

  .toggle {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    font-size: 13px;
    cursor: pointer;
  }

  .toggle input {
    margin-top: 2px;
    flex: none;
  }

  .toggle em {
    display: block;
    margin-top: 2px;
    font-size: 11.5px;
    font-style: normal;
    color: var(--text-faint);
  }
</style>
