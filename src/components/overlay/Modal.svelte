<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "$components/Icon.svelte";

  interface Props {
    title: string;
    onclose: () => void;
    children: Snippet;
  }

  let { title, onclose, children }: Props = $props();

  let dialog = $state<HTMLDivElement | null>(null);

  // Move focus into the dialog so Escape and Tab behave as expected.
  $effect(() => {
    dialog?.focus();
  });

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window on:keydown={onkeydown} />

<div class="backdrop">
  <!-- Clicking away closes the dialog. Escape does the same from the
       keyboard, so this button is hidden from assistive tech. -->
  <button class="scrim" onclick={onclose} tabindex="-1" aria-hidden="true"
  ></button>

  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    bind:this={dialog}
  >
    <header>
      <h2>{title}</h2>
      <button class="icon-btn" onclick={onclose} aria-label="Close">
        <Icon name="close" size={17} />
      </button>
    </header>
    <div class="body">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: grid;
    place-items: center;
    padding: 24px;
    background: var(--overlay);
    backdrop-filter: blur(3px);
    animation: fade-in 0.14s ease;
  }

  .scrim {
    position: absolute;
    inset: 0;
    cursor: default;
  }

  .dialog {
    position: relative;
    width: min(460px, 100%);
    max-height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    animation: dialog-in 0.16s ease;
  }

  .dialog:focus {
    outline: none;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 12px 14px 20px;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }

  .body {
    padding: 20px;
    overflow-y: auto;
  }

  /* The shared `rise-in` with a hair of scale on it, which reads as a
     dialog coming forward rather than a toast sliding up. */
  @keyframes dialog-in {
    from {
      opacity: 0;
      transform: translateY(6px) scale(0.99);
    }
  }
</style>
