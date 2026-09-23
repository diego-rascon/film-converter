<script lang="ts" module>
  import type { Theme } from "$state/settings.svelte";

  const THEMES: { value: Theme; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];
</script>

<script lang="ts">
  import { OUTPUT_FORMATS } from "$data/formats";
  import { QUALITY_RANGE, settings } from "$state/settings.svelte";

  /**
   * The output and appearance preferences. The fields only assign: the
   * settings save themselves the moment anything changes.
   */

  /** Unique to this panel, so each label's `for` finds its own field. */
  const id = $props.id();

  let activeFormat = $derived(
    OUTPUT_FORMATS.find((f) => f.value === settings.format) ?? OUTPUT_FORMATS[0],
  );
</script>

<div class="panel">
  <div class="field">
    <label class="field-label" for="{id}-format">Format</label>
    <select id="{id}-format" bind:value={settings.format}>
      {#each OUTPUT_FORMATS as format (format.value)}
        <option value={format.value}>{format.label}</option>
      {/each}
    </select>
    <p class="note">{activeFormat.note}</p>
  </div>

  {#if settings.qualityApplies}
    <div class="field">
      <label class="field-label" for="{id}-quality">
        {settings.format === "png" ? "Compression" : "Quality"}
        <span class="value">{settings.quality}</span>
      </label>
      <input
        id="{id}-quality"
        type="range"
        min={QUALITY_RANGE.min}
        max={QUALITY_RANGE.max}
        step="1"
        bind:value={settings.quality}
      />
    </div>
  {/if}

  <label class="toggle">
    <input type="checkbox" bind:checked={settings.overwrite} />
    <span>
      Overwrite existing files
      <em>Off: a numbered copy is written instead</em>
    </span>
  </label>

  <div class="field appearance">
    <span class="field-label" id="{id}-theme">Theme</span>
    <div class="segmented" role="radiogroup" aria-labelledby="{id}-theme">
      {#each THEMES as option (option.value)}
        <button
          role="radio"
          aria-checked={settings.theme === option.value}
          class:active={settings.theme === option.value}
          onclick={() => (settings.theme = option.value)}
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
     state are `.segmented` in controls.css. */
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
