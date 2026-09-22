import type { ImageItem } from "$lib/types";

/**
 * What a grid card and a list row are both given. They are two renderings
 * of one thing, so they take one set of props — a control added to an
 * image in either view is added to both, or the interface stops compiling.
 */
export interface ImageTileProps {
  image: ImageItem;
  /** Something in the roll is picked, so every tile shows its box. */
  anySelected: boolean;
  /** Show the untouched scan instead of the developed result. */
  showOriginal: boolean;
  onopen: () => void;
  ontoggleSelect: (event: MouseEvent) => void;
  oninfo: () => void;
  onreveal: () => void;
  onremove: () => void;
}
