import type { Attachment } from "svelte/attachments";

/**
 * For an overlay's root — a dialog, the viewer: takes focus in as it opens
 * and hands it back as it closes, to whatever had it before — the tile that
 * was opened, the button that raised the dialog — so the keyboard carries on
 * from where it was rather than from the top of the page.
 *
 * ```svelte
 * <div class="dialog" tabindex="-1" {@attach holdFocus}>
 * ```
 */
export const holdFocus: Attachment<HTMLElement> = (element) => {
  const previous = document.activeElement;
  element.focus();

  return () => {
    // After the update that removes the overlay, by which time the shell
    // behind it is no longer inert and can take focus again. No scrolling:
    // the viewer may have moved far from the tile it was opened on.
    queueMicrotask(() => {
      if (previous instanceof HTMLElement && previous.isConnected) {
        previous.focus({ preventScroll: true });
      }
    });
  };
};
