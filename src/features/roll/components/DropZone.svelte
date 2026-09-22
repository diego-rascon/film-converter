<script lang="ts">
  import Icon from "$components/Icon.svelte";

  interface Props {
    /** True while a drag is hovering the window. */
    dragging: boolean;
    importing: boolean;
    onpickFiles: () => void;
    onpickFolder: () => void;
  }

  let { dragging, importing, onpickFiles, onpickFolder }: Props = $props();
</script>

<div class="empty" class:dragging>
  <div class="art">
    <Icon name="film" size={52} />
  </div>

  <h2>Drop your scans here</h2>
  <p>Drag images or a folder onto this panel, or pick them yourself.</p>

  <div class="buttons">
    <button class="btn btn-primary" onclick={onpickFiles} disabled={importing}>
      <Icon name="image" size={15} />
      Choose images
    </button>
    <button class="btn" onclick={onpickFolder} disabled={importing}>
      <Icon name="folder" size={15} />
      Choose folder
    </button>
  </div>

  <p class="formats">JPEG · PNG · TIFF · BMP · WebP</p>
</div>

<style>
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    /* The gutter, because this panel is the body's content while the session
       is empty: the dashed frame stands where the first card will, so nothing
       slides inwards the moment images arrive. */
    margin: var(--gutter);
    padding: 40px 24px;
    border: 1.5px dashed var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--surface-sunken);
    text-align: center;
    transition:
      border-color 0.15s ease,
      background 0.15s ease;
  }

  .empty.dragging {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .art {
    color: var(--text-faint);
    margin-bottom: 14px;
    transition: color 0.15s ease;
  }

  .dragging .art {
    color: var(--accent);
  }

  h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  p {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
  }

  .buttons {
    display: flex;
    gap: 8px;
    margin-top: 20px;
  }

  .formats {
    margin-top: 22px;
    font-size: 11px;
    letter-spacing: 0.05em;
    color: var(--text-faint);
  }
</style>
