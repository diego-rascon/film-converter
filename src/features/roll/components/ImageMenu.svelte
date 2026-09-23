<script lang="ts" module>
  /** Roughly the popover's height — three items and a separator. */
  const MENU_HEIGHT = 132;
</script>

<script lang="ts">
  import Icon from "$components/Icon.svelte";
  import { dismissOnOutside } from "$hooks/popover.svelte";

  /**
   * The per-image actions, behind one three-dot button. It is the same menu
   * in both views; only the button changes — `caption` sits at the right end
   * of a card's caption, on the name's line, and `inline` in a list row's
   * action column.
   */
  interface Props {
    /** Named in the button's label, so screen readers say which image. */
    name: string;
    variant?: "caption" | "inline";
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
   * The shared etiquette, with one difference from the toolbar's: "inside"
   * is this menu rather than any popover, which is what closes one image's
   * menu when the next image's button is pressed.
   */
  $effect(() =>
    dismissOnOutside({
      open,
      isInside: (target) => Boolean(host?.contains(target)),
      close: () => (open = false),
    }),
  );
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
    <Icon name="more" size={variant === "inline" ? 16 : 15} />
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

  /* Card view: the button ends the caption, level with the name rather than
     floating over the picture — so it is a plain muted glyph like a row's,
     not a glass chip. It keeps out of the way until the card is hovered,
     which the card itself decides; open or focused it stays up, so the
     pointer can leave the card while the popover is on screen. The 2px the
     caption is inset by is paid back here, because a 15px glyph in a 24px
     box already hangs past its own ink. */
  .image-menu.caption {
    margin-right: -2px;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .image-menu.caption.open,
  .image-menu.caption:focus-within {
    opacity: 1;
  }

  /* One button in two sizes: the variants differ only in the square they
     take, so everything else is stated once. */
  .trigger {
    display: grid;
    place-items: center;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .trigger:hover,
  .open .trigger {
    background: var(--surface-sunken);
    color: var(--text);
  }

  /* A touch smaller than a row's, so the caption stays close to the height
     of the name it sits on. */
  .caption .trigger {
    width: 24px;
    height: 24px;
  }

  /* List view: the `--col-action` slot the header keeps for it, as a
     square. */
  .inline .trigger {
    width: var(--col-action, 28px);
    height: 28px;
  }

  /* Everything else about the panel — where it hangs, how it animates, the
     shape of an item — is `.popover` and `.menu` in controls.css. Only the width
     is this menu's, and it is set by its longest label. */
  .popover {
    min-width: 186px;
  }
</style>
