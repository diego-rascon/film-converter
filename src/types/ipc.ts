/**
 * What the Rust side sends and takes, one interface per serde struct in
 * `src-tauri/src/` — all `rename_all = "camelCase"`, so a field here is its
 * Rust name in camel case. A change to either side is a change to both.
 */

/** Output container, matching the Rust `OutputFormat`. */
export type OutputFormat = "jpg" | "png" | "tiff";

/** One scan found by `import_paths`. */
export interface ImportedImage {
  path: string;
  name: string;
  bytes: number;
}

/** Before/after thumbnails for one scan, from a single decode. */
export interface Preview {
  /** Untouched scan, as a JPEG data URL. */
  original: string;
  /** Developed result, as a JPEG data URL. */
  developed: string;
  /** Full-resolution dimensions of the source file, not of the preview. */
  width: number;
  height: number;
}

/** One file's properties, read from its header and the filesystem. */
export interface ImageMetadata {
  /** The enclosing folder. */
  directory: string;
  bytes: number;
  /** Container, e.g. "JPEG". */
  format: string;
  /** As the app sees them: a file tagged sideways is already rotated. */
  width: number;
  height: number;
  /** Colour model and depth, e.g. "RGB, 24-bit". */
  color: string;
  /** The EXIF rotation applied on load, or null when the file is upright. */
  orientation: string | null;
  /** Milliseconds since the epoch, or null where the filesystem has none. */
  modified: number | null;
  created: number | null;
}

export interface OutputSettings {
  directory: string;
  format: OutputFormat;
  quality: number;
  overwrite: boolean;
}

/** Sent over the run's channel after each image, written or failed. */
export interface BatchProgress {
  path: string;
  /** Images attempted so far in this run, this one included. */
  completed: number;
  output: string | null;
  error: string | null;
}

export interface BatchFailure {
  path: string;
  name: string;
  error: string;
}

/** How a run ended. */
export interface BatchReport {
  succeeded: number;
  cancelled: boolean;
  /** Ordered by name. */
  failures: BatchFailure[];
}
