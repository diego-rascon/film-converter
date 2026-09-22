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
 */
export function tileCallbacks(
  path: string,
  { onopen, oninfo, onreveal }: TileHandlers,
): BoundCallbacks {
  return {
    onopen: () => onopen(path),
    ontoggleSelect: (event) => {
      event.stopPropagation();
      session.pick(path, event.shiftKey);
    },
    oninfo: () => oninfo(path),
    onreveal: () => onreveal(path),
    onremove: () => session.remove([path]),
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
