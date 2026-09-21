/** Output container, matching the Rust `OutputFormat`. */
export type OutputFormat = "jpg" | "png" | "tiff";

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

export interface ImportedImage {
  path: string;
  name: string;
  bytes: number;
}

export interface Preview {
  path: string;
  original: string;
  developed: string;
  width: number;
  height: number;
}

export interface OutputSettings {
  directory: string;
  format: OutputFormat;
  quality: number;
  overwrite: boolean;
}

export interface BatchProgress {
  path: string;
  name: string;
  completed: number;
  total: number;
  output: string | null;
  error: string | null;
}

export interface BatchFailure {
  path: string;
  name: string;
  error: string;
}

export interface BatchReport {
  succeeded: number;
  failed: number;
  cancelled: boolean;
  failures: BatchFailure[];
}
