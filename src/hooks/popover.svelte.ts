import { on } from "svelte/events";

/**
 * The one etiquette every popover in the app follows: a pointerdown
 * outside it closes it, and Escape closes it without also reaching the
 * page's own shortcut handler underneath.
 */

interface Options {
  /** Nothing is bound while this is false, so a closed popover costs nothing. */
  open: boolean;
  /** True for a target that should *not* close the popover. */
  isInside: (target: HTMLElement | null) => boolean;
  close: () => void;
}

/**
 * Call from an `$effect` and return its result, so the listeners live only
 * as long as the popover is open:
 *
 * ```ts
 * $effect(() => dismissOnOutside({ open, isInside, close }));
 * ```
 *
 * Binding per opening rather than for the component's life is what lets a
 * session hold hundreds of `ImageMenu`s without hundreds of live listeners.
 * `on` rather than `addEventListener` keeps these in order with the
 * handlers Svelte delegates from the markup.
 */
export function dismissOnOutside({ open, isInside, close }: Options): (() => void) | void {
  if (!open) return;

  const stopPointer = on(window, "pointerdown", (event) => {
    if (!isInside(event.target as HTMLElement | null)) close();
  });

  const stopKey = on(
    window,
    "keydown",
    (event) => {
      if (event.key !== "Escape") return;
      // Capture phase, and swallowed: the same key would otherwise also clear
      // the selection or close the viewer behind the popover.
      event.stopPropagation();
      close();
    },
    { capture: true },
  );

  return () => {
    stopPointer();
    stopKey();
  };
}

/**
 * A set of popovers on one bar, of which at most one is open. Holding the
 * open one by name rather than a boolean each is what makes them mutually
 * exclusive by construction — opening one cannot forget to close another.
 */
export class PopoverGroup<Name extends string> {
  current = $state<Name | null>(null);

  isOpen(name: Name): boolean {
    return this.current === name;
  }

  toggle(name: Name) {
    this.current = this.current === name ? null : name;
  }

  close() {
    this.current = null;
  }
}
