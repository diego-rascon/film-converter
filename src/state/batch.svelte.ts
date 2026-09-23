import { cancelBatch, developBatch } from "$lib/api";
import { counted, describeError } from "$utils/format";
import { notices } from "./notices.svelte";
import type { BatchFailure, BatchReport, ImageItem, OutputSettings } from "$types";

/**
 * A develop run: how far it has got and what failed. The session decides
 * which images to develop and the settings where to put them; this only
 * runs them, marking each image as its result arrives.
 */
class Batch {
  /** True while a run is in flight. */
  running = $state(false);
  /** Images the current or most recent run was asked to develop. */
  total = $state(0);
  /** How many of them have been attempted so far. */
  completed = $state(0);
  /** The most recent run's failures, for the notice panel to unfold. */
  failures = $state.raw<BatchFailure[]>([]);

  /** 0 to 1, for the status bar's progress line. */
  progress = $derived(this.total > 0 ? this.completed / this.total : 0);

  async develop(targets: readonly ImageItem[], output: OutputSettings) {
    if (this.running) return;
    if (targets.length === 0) {
      notices.show("error", "No images to develop");
      return;
    }
    if (!output.directory.trim()) {
      notices.show("error", "Choose an output folder first");
      return;
    }

    // Progress names an image by path; the run's own index answers it in one
    // step, and still does if the image is removed while it develops.
    const byPath = new Map(targets.map((image) => [image.path, image]));
    for (const image of targets) {
      image.status = "developing";
      image.error = undefined;
      image.output = undefined;
    }

    this.running = true;
    this.total = targets.length;
    this.completed = 0;
    this.failures = [];
    notices.show("info", `Developing ${counted(targets.length, "image")}…`);

    try {
      const report = await developBatch([...byPath.keys()], output, (progress) => {
        const image = byPath.get(progress.path);
        if (image) {
          image.status = progress.error ? "error" : "done";
          image.error = progress.error ?? undefined;
          image.output = progress.output ?? undefined;
        }
        this.completed = progress.completed;
      });
      this.failures = report.failures;
      notices.show(...summarise(report, targets.length));
    } catch (error) {
      notices.show("error", describeError(error));
    } finally {
      // A cancelled or failed run leaves the images it never reached still
      // marked as developing.
      for (const image of targets) {
        if (image.status === "developing") image.status = "pending";
      }
      this.running = false;
    }
  }

  async cancel() {
    if (!this.running) return;
    notices.show("info", "Finishing the images in flight…");
    await cancelBatch();
  }

  /** Lets go of the last run's failure list once it has been read. */
  dismissFailures() {
    this.failures = [];
  }

  /** Forgets the last run: what it developed went with the images. */
  reset() {
    this.total = 0;
    this.completed = 0;
    this.failures = [];
  }
}

function summarise(
  report: BatchReport,
  requested: number,
): Parameters<typeof notices.show> {
  const failed = report.failures.length;
  if (report.cancelled) {
    return ["info", `Cancelled — developed ${report.succeeded} of ${requested}`];
  }
  if (failed > 0) {
    return ["error", `Developed ${report.succeeded} of ${requested} — ${failed} failed`];
  }
  return ["success", `Developed ${counted(report.succeeded, "image")}`];
}

export const batch = new Batch();
