import {
  buildPreview,
  cancelBatch,
  developBatch,
  importPaths,
} from "$lib/api";
import { CARD_PREVIEW_EDGE, PREVIEW_CONCURRENCY } from "$data/preview";
import { counted, describeError } from "$utils/format";
import { settings } from "./settings.svelte";
import type { BatchProgress, BatchReport, ImageItem } from "$types";

/** Which column the list is ordered by; `added` is the import order. */
export type SortKey = "added" | "name" | "size";
export type SortDirection = "asc" | "desc";

/** A message shown in the floating notice panel over the images. */
export interface Notice {
  kind: "info" | "error" | "success";
  text: string;
}

class Session {
  images = $state<ImageItem[]>([]);
  /** True while a develop run is in flight. */
  developing = $state(false);
  /** True while imported paths are being expanded. */
  importing = $state(false);
  /** Images finished in the current or most recent run. */
  completed = $state(0);
  notice = $state<Notice | null>(null);
  /** Failures from the most recent run, for the notice panel to unfold. */
  failures = $state<BatchReport["failures"]>([]);

  /** Index of the image open in the fullscreen viewer, or null. */
  viewerIndex = $state<number | null>(null);

  /** Ordering of `images`, driven by the list header. */
  sortKey = $state<SortKey>("added");
  sortDirection = $state<SortDirection>("asc");

  #previewQueue: string[] = [];
  #previewActive = 0;
  #requested = new Set<string>();
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
  failed = $derived(this.images.filter((i) => i.status === "error").length);
  selected = $derived(this.images.filter((i) => i.selected));
  hasSelection = $derived(this.selected.length > 0);
  allSelected = $derived(
    this.images.length > 0 && this.selected.length === this.images.length,
  );
  /** Some but not all — the indeterminate state of the header checkbox. */
  partlySelected = $derived(this.hasSelection && !this.allSelected);

  viewerImage = $derived(
    this.viewerIndex === null
      ? null
      : (this.images[this.viewerIndex] ?? null),
  );

  /** Adds paths (files or folders) to the session, skipping duplicates. */
  async add(paths: string[]) {
    if (paths.length === 0) return;

    this.importing = true;
    try {
      const found = await importPaths(paths);
      const known = new Set(this.images.map((i) => i.path));
      const fresh = found
        .filter((f) => !known.has(f.path))
        .map(
          (f): ImageItem => ({
            path: f.path,
            name: f.name,
            bytes: f.bytes,
            sequence: this.#nextSequence++,
            status: "pending",
            selected: false,
            previewStatus: "idle",
          }),
        );

      if (fresh.length === 0) {
        this.notice =
          found.length > 0
            ? { kind: "info", text: "Those images are already loaded" }
            : { kind: "error", text: "No supported images found" };
        return;
      }

      this.images = [...this.images, ...fresh];
      this.#applySort();
      this.notice = {
        kind: "info",
        text: `Added ${counted(fresh.length, "image")}`,
      };
      this.queuePreviews(fresh.map((f) => f.path));
    } catch (error) {
      this.notice = { kind: "error", text: describeError(error) };
    } finally {
      this.importing = false;
    }
  }

  /** Asks for previews of `paths`, a few at a time. */
  queuePreviews(paths: string[]) {
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
      if (!path) return;

      const image = this.find(path);
      // The image may have been removed while it sat in the queue.
      if (!image) continue;

      this.#previewActive += 1;
      image.previewStatus = "loading";

      buildPreview(path, CARD_PREVIEW_EDGE)
        .then((preview) => {
          const target = this.find(path);
          if (!target) return;
          target.original = preview.original;
          target.developed = preview.developed;
          target.width = preview.width;
          target.height = preview.height;
          target.previewStatus = "ready";
        })
        .catch(() => {
          const target = this.find(path);
          if (target) target.previewStatus = "error";
        })
        .finally(() => {
          this.#previewActive -= 1;
          this.#pumpPreviews();
        });
    }
  }

  find(path: string): ImageItem | undefined {
    return this.images.find((i) => i.path === path);
  }

  remove(paths: string[]) {
    const removing = new Set(paths);
    if (removing.size === 0) return;

    const survivor = this.images.filter((i) => !removing.has(i.path));
    const removedCount = this.images.length - survivor.length;
    this.images = survivor;
    for (const path of removing) this.#requested.delete(path);
    if (this.#anchor !== null && removing.has(this.#anchor)) this.#anchor = null;

    // Keep the viewer pointing at something sensible, or close it.
    if (this.viewerIndex !== null) {
      if (survivor.length === 0) this.viewerIndex = null;
      else this.viewerIndex = Math.min(this.viewerIndex, survivor.length - 1);
    }

    if (removedCount > 0) {
      this.notice = {
        kind: "info",
        text: `Removed ${counted(removedCount, "image")}`,
      };
    }
  }

  removeSelected() {
    this.remove(this.selected.map((i) => i.path));
  }

  /**
   * Empties the session back to the drop zone. The view preferences — the
   * sort column, the view mode — are the user's and survive; only the roll
   * goes. Previews already in flight resolve against an image that is no
   * longer there and drop what they decoded.
   */
  clear() {
    const cleared = this.images.length;

    this.images = [];
    this.viewerIndex = null;
    this.completed = 0;
    this.failures = [];
    this.#requested.clear();
    this.#previewQueue.length = 0;
    this.#nextSequence = 0;
    this.#anchor = null;
    // Whatever the last notice was about went with the images, so the clear
    // speaks for itself rather than leaving the previous run's summary up.
    this.notice = cleared
      ? { kind: "info", text: `Cleared ${counted(cleared, "image")}` }
      : null;
  }

  toggleSelected(path: string) {
    const image = this.find(path);
    if (image) image.selected = !image.selected;
  }

  /**
   * A click on an image's checkbox. Plain, it picks that one; with shift it
   * extends from the last one picked, which is why the anchor is kept here
   * rather than in the page — it belongs to the selection, and the range is
   * read off `images`, the one order the app has.
   */
  pick(path: string, extend: boolean) {
    const anchor = this.#anchor;

    if (extend && anchor !== null) {
      const from = this.images.findIndex((i) => i.path === anchor);
      const to = this.images.findIndex((i) => i.path === path);

      if (from >= 0 && to >= 0) {
        const [start, end] = from < to ? [from, to] : [to, from];
        // The range takes the value the clicked image is heading for, so a
        // shift-click can clear a run as well as extend one.
        const value = !this.images[to].selected;
        for (let i = start; i <= end; i += 1) this.images[i].selected = value;
        this.#anchor = path;
        return;
      }
    }

    this.toggleSelected(path);
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
    this.#applySort();
  }

  #applySort() {
    const key = this.sortKey;
    const direction = this.sortDirection === "desc" ? -1 : 1;
    // The viewer addresses an image by index, so it has to be carried across.
    const viewing = this.viewerImage?.path ?? null;

    this.images = [...this.images].sort((a, b) => {
      let order = 0;
      if (key === "name") {
        // Numeric collation so DSC_9 sorts before DSC_10.
        order = a.name.localeCompare(b.name, undefined, {
          numeric: true,
          sensitivity: "base",
        });
      } else if (key === "size") {
        order = a.bytes - b.bytes;
      }
      // Ties, and `added`, fall back to the import order.
      return order !== 0 ? direction * order : a.sequence - b.sequence;
    });

    if (viewing !== null) {
      const index = this.images.findIndex((i) => i.path === viewing);
      if (index >= 0) this.viewerIndex = index;
    }
  }

  /** Opens the fullscreen viewer on `path`. */
  openViewer(path: string) {
    const index = this.images.findIndex((i) => i.path === path);
    if (index >= 0) this.viewerIndex = index;
  }

  closeViewer() {
    this.viewerIndex = null;
  }

  /** Moves the viewer by `step`, wrapping around the ends. */
  stepViewer(step: number) {
    if (this.viewerIndex === null || this.images.length === 0) return;
    const count = this.images.length;
    this.viewerIndex = (this.viewerIndex + step + count) % count;
  }

  /** Folds a per-image progress event into the session. */
  applyProgress(progress: BatchProgress) {
    const image = this.find(progress.path);
    if (image) {
      image.status = progress.error ? "error" : "done";
      image.error = progress.error ?? undefined;
      image.output = progress.output ?? undefined;
    }
    this.completed = progress.completed;
  }

  /**
   * Develops every loaded image, or only the selected ones when a selection
   * is active.
   */
  async develop() {
    if (this.developing) return;

    const targets = this.hasSelection ? this.selected : this.images;
    if (targets.length === 0) {
      this.notice = { kind: "error", text: "No images to develop" };
      return;
    }
    if (!settings.directory.trim()) {
      this.notice = { kind: "error", text: "Choose an output folder first" };
      return;
    }

    for (const image of targets) {
      image.status = "developing";
      image.error = undefined;
      image.output = undefined;
    }

    this.developing = true;
    this.completed = 0;
    this.failures = [];
    this.notice = {
      kind: "info",
      text: `Developing ${counted(targets.length, "image")}…`,
    };

    try {
      const report = await developBatch(
        targets.map((i) => i.path),
        settings.output,
      );
      this.failures = report.failures;

      // A cancelled run leaves untouched images still marked as developing.
      for (const image of targets) {
        if (image.status === "developing") image.status = "pending";
      }

      this.notice = summarise(report, targets.length);
    } catch (error) {
      for (const image of targets) {
        if (image.status === "developing") image.status = "pending";
      }
      this.notice = { kind: "error", text: describeError(error) };
    } finally {
      this.developing = false;
    }
  }

  async cancel() {
    if (!this.developing) return;
    this.notice = { kind: "info", text: "Finishing the images in flight…" };
    await cancelBatch();
  }
}

function summarise(report: BatchReport, requested: number): Notice {
  if (report.cancelled) {
    return {
      kind: "info",
      text: `Cancelled — developed ${report.succeeded} of ${requested}`,
    };
  }
  if (report.failed > 0) {
    return {
      kind: "error",
      text: `Developed ${report.succeeded} of ${requested} — ${report.failed} failed`,
    };
  }
  return {
    kind: "success",
    text: `Developed ${counted(report.succeeded, "image")}`,
  };
}

export const session = new Session();
