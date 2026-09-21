<script lang="ts">
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
</script>

<div class="panel">
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
