<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";

  import Modal from "$components/overlay/Modal.svelte";
  import Icon from "$components/Icon.svelte";
  import { INPUT_FORMAT_NAMES, OUTPUT_FORMATS } from "$data/formats";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  /** The version the app was built as, rather than a copy of it here. */
  const version = getVersion();
</script>

<Modal title="About" {onclose}>
  <div class="about">
    <div class="mark">
      <Icon name="film" size={26} />
    </div>

    <div>
      <h3>Film Converter</h3>
      <p class="version">
        {#await version then number}Version {number}{/await}
      </p>
    </div>

    <p>
      Turns scanned colour negatives into positives. Each image is inverted,
      then every colour channel is stretched between its own black and white
      points — which is what lifts the orange mask off the frame.
    </p>

    <dl>
      <div>
        <dt class="caps">Clipping</dt>
        <dd>0.5% shadows, 0.5% highlights, per channel</dd>
      </div>
      <div>
        <dt class="caps">Reads</dt>
        <dd>{INPUT_FORMAT_NAMES.join(", ")}</dd>
      </div>
      <div>
        <dt class="caps">Writes</dt>
        <dd>{OUTPUT_FORMATS.map((format) => format.label).join(", ")}</dd>
      </div>
    </dl>
  </div>
</Modal>

<style>
  .about {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .mark {
    display: grid;
    place-items: center;
    width: 50px;
    height: 50px;
    border-radius: var(--radius);
    background: var(--accent-soft);
    color: var(--accent);
  }

  h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .version {
    margin: 2px 0 0;
    font-size: 12.5px;
    color: var(--text-faint);
  }

  p {
    margin: 0;
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-muted);
  }

  dl {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 4px 0 0;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }

  dl div {
    display: grid;
    grid-template-columns: 78px 1fr;
    gap: 12px;
  }

  /* `.caps` in the markup carries the type; this is the optical nudge
     that lands it on the value's baseline. */
  dt {
    padding-top: 1px;
  }

  dd {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-muted);
  }
</style>
