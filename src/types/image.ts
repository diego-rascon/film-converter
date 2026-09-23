/**
 * The app's own model of a scan: what the importer found, plus how the
 * session is treating it. Every view reads it, so it lives here rather than
 * with any one of them.
 */

/** How an image is faring in the current session. */
export type ImageStatus = "pending" | "developing" | "done" | "error";

/** Whether a preview has been generated yet. */
export type PreviewStatus = "idle" | "loading" | "ready" | "error";

/** One scan imported into the session. */
export interface ImageItem {
  /** The absolute path, which also serves as the identity of the image. */
  path: string;
  name: string;
  bytes: number;
  /** Position in the import order, which sorting can always fall back to. */
  sequence: number;
  status: ImageStatus;
  selected: boolean;
  /** Set when developing failed. */
  error?: string;
  /** Where the developed file was written. */
  output?: string;
  previewStatus: PreviewStatus;
  /** Untouched scan, as a data URL. */
  original?: string;
  /** Developed result, as a data URL. */
  developed?: string;
  /** Full-resolution dimensions of the source file. */
  width?: number;
  height?: number;
}
