<script lang="ts">
  import Icon from "./Icon.svelte";

  /**
   * The per-image actions, behind one three-dot button. It is the same menu
   * in both views; only the button changes — `overlay` floats over a card's
   * thumbnail, `inline` sits in a list row's action column.
   */
  interface Props {
    /** Named in the button's label, so screen readers say which image. */
    name: string;
    variant?: "overlay" | "inline";
    oninfo: () => void;
    onreveal: () => void;
    onremove: () => void;
  }

  let { name, variant = "inline", oninfo, onreveal, onremove }: Props =
    $props();

  let open = $state(false);
  let host = $state<HTMLDivElement | null>(null);
  /** Set on opening: a menu near the bottom of the window drops upwards. */
  let up = $state(false);

  /** Roughly the popover's height — three items and a separator. */
  const MENU_HEIGHT = 132;

  function toggle() {
    if (!open && host) {
      const box = host.getBoundingClientRect();
      up = box.bottom + MENU_HEIGHT > window.innerHeight;
    }
    open = !open;
  }

  function choose(action: () => void) {
    open = false;
    action();
  }

  /**
   * Same popover etiquette as the toolbar's, but bound only while the menu is
   * open: there is one of these per image, and a session holds hundreds, so
   * they cannot each keep a pair of window listeners alive. "Outside" is this
   * menu rather than any popover, which is what closes one card's menu when
   * the next card's button is pressed.
   */
  $effect(() => {
    if (!open) return;

    const onpointerdown = (event: PointerEvent) => {
      if (host?.contains(event.target as Node)) return;
      open = false;
    };

    const onkeydown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      // Swallowed so Escape does not also clear the selection underneath.
      event.stopPropagation();
      open = false;
    };

    window.addEventListener("pointerdown", onpointerdown);
    window.addEventListener("keydown", onkeydown, true);
    return () => {
      window.removeEventListener("pointerdown", onpointerdown);
      window.removeEventListener("keydown", onkeydown, true);
    };
  });
</script>

<!-- No `data-popover`: this menu closes itself (see above), and leaving the
     attribute off means opening it also dismisses the toolbar's own popover. -->
<div class="image-menu {variant}" class:open bind:this={host}>
  <button
    class="trigger"
    onclick={toggle}
    aria-expanded={open}
    aria-haspopup="menu"
    aria-label="Actions for {name}"
    title="Actions"
  >
    <Icon name="more" size={variant === "overlay" ? 15 : 16} />
  </button>

  {#if open}
    <div class="popover menu" class:up role="menu">
      <button role="menuitem" onclick={() => choose(oninfo)}>
        <Icon name="info" size={15} />
        Properties
      </button>
      <button role="menuitem" onclick={() => choose(onreveal)}>
        <Icon name="folder" size={15} />
        Show in folder
      </button>
      <div class="separator"></div>
      <button class="danger" role="menuitem" onclick={() => choose(onremove)}>
        <Icon name="trash" size={15} />
        Remove
      </button>
    </div>
  {/if}
</div>

<style>
  .image-menu {
    position: relative;
    flex: none;
  }

  /* Card view: the button is a chip over the thumbnail's top-right corner.
     The popover has to escape the frame's `overflow: hidden`, so the host is
     a sibling of the frame and positions itself against the figure. */
  .image-menu.overlay {
    position: absolute;
    top: 7px;
    right: 7px;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .image-menu.overlay.open,
  .image-menu.overlay:focus-within {
    opacity: 1;
  }

  .trigger {
    display: grid;
    place-items: center;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .overlay .trigger {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.55);
    color: #fff;
  }

  .overlay .trigger:hover,
  .overlay.open .trigger {
    background: rgba(0, 0, 0, 0.78);
  }

  /* List view: the same square as the remove button it replaced, on the
     `--col-action` slot the header keeps for it. */
  .inline .trigger {
    width: var(--col-action, 28px);
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .inline .trigger:hover,
  .inline.open .trigger {
    background: var(--surface-sunken);
    color: var(--text);
  }

  .popover {
    position: absolute;
    top: calc(100% + 6px);
    /* Right-aligned: the button is at the right edge of a card or a row, so
       the menu grows inwards rather than off the window. */
    right: 0;
    z-index: 40;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
    animation: drop 0.12s ease;
  }

  /* Rows near the bottom of a long list would otherwise open into nothing. */
  .popover.up {
    top: auto;
    bottom: calc(100% + 6px);
    animation-name: rise;
  }

  .popover.menu {
    min-width: 186px;
    padding: 5px;
  }

  .popover.menu button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    text-align: left;
    white-space: nowrap;
    color: var(--text);
  }

  .popover.menu button:hover {
    background: var(--surface-sunken);
  }

  .popover.menu button :global(svg) {
    color: var(--text-faint);
  }

  .popover.menu button.danger,
  .popover.menu button.danger :global(svg) {
    color: var(--danger);
  }

  .popover.menu button.danger:hover {
    background: var(--danger-soft);
  }

  .separator {
    height: 1px;
    margin: 5px 4px;
    background: var(--border);
  }

  @keyframes drop {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }
</style>
