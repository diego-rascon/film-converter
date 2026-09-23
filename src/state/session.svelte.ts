import { buildPreview, importPaths } from "$lib/api";
import { CARD_PREVIEW_EDGE, PREVIEW_CONCURRENCY } from "$data/preview";
import { counted, describeError } from "$utils/format";
import { batch } from "./batch.svelte";
import { notices } from "./notices.svelte";
import type { ImageItem, ImportedImage } from "$types";

/** Which column the list is ordered by; `added` is the import order. */
export type SortKey = "added" | "name" | "size";
export type SortDirection = "asc" | "desc";

/**
 * Which modifiers a click on an image carried. The views are file-explorer
 * shaped, so the two flags are the two a file manager reads: `toggle` is
 * ctrl/cmd — or a tick box, which is a toggle by its nature — and leaves the
 * rest of the selection alone, and `extend` is shift, which takes the whole
 * run from the anchor. Neither means the click *is* the selection.
 */
export interface PickModifiers {
  toggle?: boolean;
  extend?: boolean;
}

/**
 * The loaded roll: its images, their order, which are selected, and which
 * one the viewer has open. What a develop run does with them is
 * [batch](./batch.svelte.ts)'s business, and the message on screen is
 * [notices](./notices.svelte.ts)'.
 */
class Session {
  images = $state<ImageItem[]>([]);
  /** True while imported paths are being expanded. */
  importing = $state(false);

  /** Ordering of `images`, driven by the view header. */
  sortKey = $state<SortKey>("added");
  sortDirection = $state<SortDirection>("asc");

  /** Paths waiting for a thumbnail, decoded a few at a time. */
  #previewQueue: string[] = [];
  #previewActive = 0;
  #requested = new Set<string>();

  /**
   * The image open in the fullscreen viewer, by path rather than position:
   * re-sorting the roll moves it, and the viewer should stay on it.
   */
  #viewing = $state<string | null>(null);
  /** Hands out `ImageItem.sequence`, so the import order stays recoverable. */
  #nextSequence = 0;
  /** The last image picked, which a shift-click extends from. */
  #anchor: string | null = null;

  /**
   * These are `$derived`, not plain getters, because a getter recomputes on
   * every read: `hasSelection` is passed to every card and every row, so a
   * plain one would scan the whole roll once per image each time anything
   * was picked. Derived, each is computed once per change and shared by
   * everything that reads it.
   */
  total = $derived(this.images.length);
  developed = $derived(this.images.filter((i) => i.status === "done").length);
  selected = $derived(this.images.filter((i) => i.selected));
  hasSelection = $derived(this.selected.length > 0);
  allSelected = $derived(this.total > 0 && this.selected.length === this.total);
  /** Some but not all — the indeterminate state of the header checkbox. */
  partlySelected = $derived(this.hasSelection && !this.allSelected);
  /** What Save develops: the selection when there is one, otherwise the lot. */
  targets = $derived(this.hasSelection ? this.selected : this.images);

  /** The image open in the viewer and where it sits in the roll, or null. */
  viewer = $derived.by(() => {
    const path = this.#viewing;
    const index = path === null ? -1 : this.images.findIndex((i) => i.path === path);
    return index < 0 ? null : { image: this.images[index], index };
  });

  /**
   * Every image by path. Rebuilt only when the roll itself changes — an
   * image's status or selection is not part of it — so the lookups previews
   * and picks make per image cost nothing to repeat.
   */
  #byPath = $derived(new Map(this.images.map((image) => [image.path, image])));

  find(path: string): ImageItem | undefined {
    return this.#byPath.get(path);
  }

  /** Adds paths (files or folders) to the session, skipping duplicates. */
  async add(paths: string[]) {
    if (paths.length === 0) return;

    this.importing = true;
    try {
      const found = await importPaths(paths);
      const fresh = found
        .filter((f) => !this.#byPath.has(f.path))
        .map((f) => this.#item(f));

      if (fresh.length === 0) {
        if (found.length > 0) notices.show("info", "Those images are already loaded");
        else notices.show("error", "No supported images found");
        return;
      }

      this.images = sorted([...this.images, ...fresh], this.sortKey, this.sortDirection);
      notices.show("info", `Added ${counted(fresh.length, "image")}`);
      this.#queuePreviews(fresh.map((f) => f.path));
    } catch (error) {
      notices.show("error", describeError(error));
    } finally {
      this.importing = false;
    }
  }

  #item(found: ImportedImage): ImageItem {
    return {
      ...found,
      sequence: this.#nextSequence++,
      status: "pending",
      selected: false,
      previewStatus: "idle",
    };
  }

  /** Asks for previews of `paths`, skipping any already asked for. */
  #queuePreviews(paths: string[]) {
    for (const path of paths) {
      if (this.#requested.has(path)) continue;
      this.#requested.add(path);
      this.#previewQueue.push(path);
    }
    this.#pumpPreviews();
  }

  #pumpPreviews() {
    while (this.#previewActive < PREVIEW_CONCURRENCY) {
      const path = this.#previewQueue.shift();
      if (path === undefined) return;

      this.#previewActive += 1;
      this.#loadPreview(path).finally(() => {
        this.#previewActive -= 1;
        this.#pumpPreviews();
      });
    }
  }

  async #loadPreview(path: string) {
    const image = this.find(path);
    // Removed while it waited its turn.
    if (!image) return;
    image.previewStatus = "loading";

    try {
      const preview = await buildPreview(path, CARD_PREVIEW_EDGE);
      // Looked up again: the image may have gone while it decoded.
      const target = this.find(path);
      if (!target) return;
      Object.assign(target, preview);
      target.previewStatus = "ready";
    } catch {
      const target = this.find(path);
      if (target) target.previewStatus = "error";
    }
  }

  remove(paths: Iterable<string>) {
    const removing = new Set(paths);
    const survivors = this.images.filter((i) => !removing.has(i.path));
    const removed = this.images.length - survivors.length;
    if (removed === 0) return;

    // The viewer moves on to whichever image takes the removed one's place,
    // or closes with the last of them.
    const viewer = this.viewer;
    if (viewer && removing.has(viewer.image.path)) {
      this.#viewing = survivors[Math.min(viewer.index, survivors.length - 1)]?.path ?? null;
    }

    this.images = survivors;
    for (const path of removing) this.#requested.delete(path);
    if (this.#anchor !== null && removing.has(this.#anchor)) this.#anchor = null;
    notices.show("info", `Removed ${counted(removed, "image")}`);
  }

  removeSelected() {
    this.remove(this.selected.map((i) => i.path));
  }

  /**
   * Empties the session back to the drop zone. The view preferences — the
   * sort column, the view mode — are the user's and survive; only the roll
   * goes, and with it the last run's summary. Previews already in flight
   * resolve against an image that is no longer there and drop what they
   * decoded.
   */
  clear() {
    const cleared = this.images.length;

    this.images = [];
    this.#viewing = null;
    this.#nextSequence = 0;
    this.#anchor = null;
    this.#requested.clear();
    this.#previewQueue.length = 0;
    batch.reset();

    // Whatever the last notice was about went with the images, so the clear
    // speaks for itself rather than leaving the previous run's summary up.
    if (cleared > 0) notices.show("info", `Cleared ${counted(cleared, "image")}`);
    else notices.clear();
  }

  /**
   * A click on an image. Plain, it is the whole selection — everything else
   * lets go, the way clicking a file in a file manager does; with `toggle` it
   * joins or leaves the selection without disturbing it; with `extend` it
   * takes the run from the last image picked. The anchor is kept here rather
   * than in the page because it belongs to the selection, and the range is
   * read off `images`, the one order the app has.
   */
  pick(path: string, { toggle = false, extend = false }: PickModifiers = {}) {
    const anchor = this.#anchor;

    if (extend && anchor !== null) {
      const from = this.images.findIndex((i) => i.path === anchor);
      const to = this.images.findIndex((i) => i.path === path);

      if (from >= 0 && to >= 0) {
        const [start, end] = from < to ? [from, to] : [to, from];
        // Toggling, the run takes the value the clicked image is heading for,
        // so a shift-click can clear a run as well as extend one. Replacing,
        // the run *is* the selection. Either way the anchor stays where it
        // was: a second shift-click grows or shrinks the same run rather than
        // starting a new one from the end of the last.
        const value = toggle ? !this.images[to].selected : true;
        if (!toggle) this.setAllSelected(false);
        for (let i = start; i <= end; i += 1) this.images[i].selected = value;
        return;
      }
    }

    const image = this.find(path);
    if (!image) return;
    if (toggle) {
      image.selected = !image.selected;
    } else {
      // Not a toggle: a plain click on the one already picked keeps it picked
      // rather than letting go, since it is a selection of one being made.
      this.setAllSelected(false);
      image.selected = true;
    }
    this.#anchor = path;
  }

  setAllSelected(value: boolean) {
    for (const image of this.images) image.selected = value;
  }

  /**
   * Cycles a column: ascending, descending, then back to the import order.
   * The images array is the one order the app has — the grid, the viewer and
   * shift-click ranges all read it — so sorting reorders it rather than
   * building a second, list-only view of the same images.
   */
  sortBy(key: Exclude<SortKey, "added">) {
    if (this.sortKey !== key) {
      this.sortKey = key;
      this.sortDirection = "asc";
    } else if (this.sortDirection === "asc") {
      this.sortDirection = "desc";
    } else {
      this.sortKey = "added";
      this.sortDirection = "asc";
    }
    this.images = sorted(this.images, this.sortKey, this.sortDirection);
  }

  /** Opens the fullscreen viewer on `path`. */
  openViewer(path: string) {
    if (this.#byPath.has(path)) this.#viewing = path;
  }

  closeViewer() {
    this.#viewing = null;
  }

  /** Moves the viewer by `step`, wrapping around the ends. */
  stepViewer(step: number) {
    const viewer = this.viewer;
    if (!viewer) return;
    const count = this.images.length;
    this.#viewing = this.images[(viewer.index + step + count) % count].path;
  }
}

/** Numeric, so DSC_9 sorts before DSC_10; blind to case and accents. */
const names = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/** A copy of `images` in the order the header asks for. */
function sorted(images: ImageItem[], key: SortKey, direction: SortDirection): ImageItem[] {
  const sign = direction === "desc" ? -1 : 1;
  return [...images].sort((a, b) => {
    const order =
      key === "name" ? names.compare(a.name, b.name) : key === "size" ? a.bytes - b.bytes : 0;
    // Ties, and `added`, fall back to the import order.
    return order !== 0 ? sign * order : a.sequence - b.sequence;
  });
}

export const session = new Session();
