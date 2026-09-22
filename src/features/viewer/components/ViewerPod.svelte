<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * A cluster of controls floating in a bottom corner of the stage, the way
   * GNOME's image viewer arranges them. What is behind it is a photograph,
   * so it takes its contrast from the glass tokens rather than from any
   * surface colour.
   *
   * The stage's own `pointerdown` bails on anything inside a `.pod`, which
   * covers the gaps between the buttons as well as the buttons themselves —
   * a `stopPropagation` per button would not.
   */
  interface Props {
    side: "left" | "right";
    label: string;
    children: Snippet;
  }

  let { side, label, children }: Props = $props();
</script>

<div class="pod {side}" role="group" aria-label={label}>
  {@render children()}
</div>

<style>
  .pod {
    position: absolute;
    /* Flush to the stage's corners, which the body's padding has already put
       on `--gutter`: a pod is its own ink, so its edge lands on the window's
       margin exactly and on the picture's, and the pager lines up under the
       header's mode switch. */
    bottom: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 2px;
    /* The geometry of every other cluster of controls in the app — the mode
       switch, the toolbar's — rather than a pill: 2px of padding around
       children a step smaller than the box that holds them. */
    padding: 2px;
    border-radius: var(--radius);
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    /* A letterboxed scan leaves a pod sitting on the backdrop instead of on
       the picture, where the glass alone has nothing to darken. The hairline
       is what gives it an edge on either. */
    box-shadow:
      inset 0 0 0 1px rgba(255, 255, 255, 0.1),
      0 2px 10px rgba(0, 0, 0, 0.3);
  }

  .left {
    left: 0;
  }

  .right {
    right: 0;
  }

  /* An `.icon-btn` in all but its colours, which have to come off the glass
     rather than off the surface tokens. */
  .pod :global(button) {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    color: rgba(255, 255, 255, 0.82);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .pod :global(button:hover:not(:disabled)) {
    background: rgba(255, 255, 255, 0.16);
    color: #fff;
  }

  .pod :global(button:disabled) {
    opacity: 0.35;
    cursor: default;
  }
</style>
