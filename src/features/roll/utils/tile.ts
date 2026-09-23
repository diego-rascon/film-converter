import type { Attachment } from "svelte/attachments";

import { onVisibility } from "$hooks/visibility";
import { session } from "$state/session.svelte";
import type { ImageItem } from "$types";
import type { ImageTileProps } from "../types";

/** The per-image callbacks a view hands down, before they are bound to a path. */
export type TileHandlers = {
  onopen: (path: string) => void;
  oninfo: (path: string) => void;
  onreveal: (path: string) => void;
};

/** Everything in `ImageTileProps` except what the view supplies per image. */
type BoundCallbacks = Omit<
  ImageTileProps,
  "image" | "anySelected" | "showOriginal"
>;

/**
 * Binds one image's path into the callbacks a card or a row takes. The grid
 * and the list are two renderings of one thing and answer a tile the same
 * way — picking, removing and opening do not differ between them — so the
 * wiring is stated here once rather than in each view.
 *
 * The gestures are a file manager's: a click picks, a double click opens,
 * ctrl/cmd picks alongside what is already picked and shift takes the run.
 * Which modifier means what is decided here for the same reason — a card and
 * a row read one click identically.
 */
export function tileCallbacks(
  path: string,
  { onopen, oninfo, onreveal }: TileHandlers,
): BoundCallbacks {
  return {
    onopen: () => onopen(path),
    onpick: (event) => {
      session.pick(path, {
        // Ctrl where the platform uses it, cmd on a Mac.
        toggle: event.ctrlKey || event.metaKey,
        extend: event.shiftKey,
      });
    },
    ontoggleSelect: (event) => {
      // The box is a toggle whatever else is picked — that is what it is for,
      // and it is how a selection is built without holding a key down. Its
      // click must not also reach the image behind it, which would replace
      // the selection the box is being used to add to.
      event.stopPropagation();
      session.pick(path, { toggle: true, extend: event.shiftKey });
    },
    oninfo: () => oninfo(path),
    onreveal: () => onreveal(path),
    onremove: () => session.remove([path]),
  };
}

/**
 * Enter opens an image, the way a double click does. A `<button>` raises a
 * click of its own from Enter, which would pick the image rather than open
 * it, so the default is held back here. Space is deliberately left alone: it
 * still reaches `onpick`, which is the key a file manager picks with.
 */
export function openOnEnter(onopen: () => void) {
  return (event: KeyboardEvent) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    onopen();
  };
}

/**
 * Which of an image's two previews the tile shows. The before/after toggle
 * reaches a card and a row alike, so they read it the same way.
 */
export function previewSource(
  image: ImageItem,
  showOriginal: boolean,
): string | undefined {
  return showOriginal ? image.original : image.developed;
}

/**
 * For a tile's root: moves its preview up the decode queue while the tile is
 * on screen, so a large import fills in where the user is looking.
 */
export function previewPriority(path: string): Attachment {
  return onVisibility((visible) => session.previews.want(path, visible));
}
