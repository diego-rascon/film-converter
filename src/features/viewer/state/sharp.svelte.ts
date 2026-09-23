import { untrack } from "svelte";
import { SvelteMap } from "svelte/reactivity";

import { buildPreview } from "$lib/api";
import { FULL_PREVIEW_EDGE } from "$data/preview";

/** A before/after pair at the viewer's resolution. */
export interface Pair {
  original: string;
  developed: string;
}

/**
 * The viewer's sharper pairs. Each is a full decode of the scan, so one is
 * only asked for once the viewer has settled on an image — holding an arrow
 * key down should not queue a decode per image passed — and the last few
 * are kept, so stepping back to an image just seen is instant.
 */
export class SharpPreviews {
  /** How long the viewer stays on an image before its pair is asked for. */
  static readonly SETTLE_MS = 150;
  /** Pairs kept, oldest dropped first. A 2000px pair is a few hundred KB. */
  static readonly KEEP = 6;

  #pairs = new SvelteMap<string, Pair>();
  #fetching = new Set<string>();

  /** The pair for `path`, once it has arrived. Reactive. */
  get(path: string): Pair | undefined {
    return this.#pairs.get(path);
  }

  /**
   * For an `$effect`: asks for `path`'s pair if the viewer is still on it
   * after a moment, and returns the teardown that stops waiting. A pair that
   * arrives after the viewer has moved on is kept all the same, since the
   * decode is done.
   */
  fetch(path: string): () => void {
    // Untracked, so the pair arriving does not re-run the effect that asked.
    const known = untrack(() => this.#pairs.has(path));
    if (known || this.#fetching.has(path)) return () => {};

    const timer = setTimeout(() => {
      this.#fetching.add(path);
      buildPreview(path, FULL_PREVIEW_EDGE)
        .then(({ original, developed }) => this.#keep(path, { original, developed }))
        .catch(() => {
          // The card preview is already on screen; nothing more to show.
        })
        .finally(() => this.#fetching.delete(path));
    }, SharpPreviews.SETTLE_MS);

    return () => clearTimeout(timer);
  }

  #keep(path: string, pair: Pair) {
    this.#pairs.set(path, pair);
    // A map iterates in insertion order, so the first key is the oldest.
    for (const oldest of this.#pairs.keys()) {
      if (this.#pairs.size <= SharpPreviews.KEEP) break;
      this.#pairs.delete(oldest);
    }
  }
}
