<script lang="ts">
  import Modal from "./Modal.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();
</script>

<Modal title="About" {onclose}>
  <div class="about">
    <div class="mark">
      <Icon name="film" size={26} />
    </div>

    <div>
      <h3>Film Converter</h3>
      <p class="version">Version 0.1.0</p>
    </div>

    <p>
      Turns scanned colour negatives into positives. Each image is inverted,
      then every colour channel is stretched between its own black and white
      points — which is what lifts the orange mask off the frame.
    </p>

    <dl>
      <div>
        <dt>Clipping</dt>
        <dd>0.5% shadows, 0.5% highlights, per channel</dd>
      </div>
      <div>
        <dt>Reads</dt>
        <dd>JPEG, PNG, TIFF, BMP, WebP</dd>
      </div>
      <div>
        <dt>Writes</dt>
        <dd>JPEG, PNG, TIFF</dd>
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

  dt {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-faint);
    padding-top: 1px;
  }

  dd {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-muted);
  }
</style>
