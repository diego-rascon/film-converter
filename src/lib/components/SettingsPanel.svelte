<script lang="ts">
  import { settings } from "$lib/settings.svelte";
  import type { Theme } from "$lib/settings.svelte";
  import type { OutputFormat } from "$lib/types";

  const formats: { value: OutputFormat; label: string; note: string }[] = [
    { value: "jpg", label: "JPEG", note: "4:4:4, no chroma subsampling" },
    { value: "png", label: "PNG", note: "Lossless; quality sets compression" },
    { value: "tiff", label: "TIFF", note: "Lossless, uncompressed" },
  ];

  const themes: { value: Theme; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];

  let activeFormat = $derived(
    formats.find((f) => f.value === settings.format) ?? formats[0],
  );

  function chooseTheme(theme: Theme) {
    settings.setTheme(theme);
    settings.save();
  }
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

  <div class="field appearance">
    <span class="field-label" id="theme-label">Theme</span>
    <div class="segmented" role="radiogroup" aria-labelledby="theme-label">
      {#each themes as option (option.value)}
        <button
          role="radio"
          aria-checked={settings.theme === option.value}
          class:active={settings.theme === option.value}
          onclick={() => chooseTheme(option.value)}
        >
          {option.label}
        </button>
      {/each}
    </div>
    {#if settings.theme === "auto"}
      <p class="note">
        Following the system — {settings.resolvedTheme} right now
      </p>
    {/if}
  </div>
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

  /* Appearance is not an output setting, so it sits below a hairline. */
  .appearance {
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  /* The three share the panel's width evenly; the track and the pressed
     state are `.segmented` in app.css. */
  .segmented button {
    flex: 1;
    padding: 6px 0;
    font-size: 12.5px;
    font-weight: 500;
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
