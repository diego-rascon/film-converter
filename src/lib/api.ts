import { invoke } from "@tauri-apps/api/core";
import type {
  BatchReport,
  ImportedImage,
  OutputSettings,
  Preview,
} from "./types";

/** Expands dropped or picked paths into a flat list of supported images. */
export function importPaths(paths: string[]): Promise<ImportedImage[]> {
  return invoke("import_paths", { paths });
}

/** Decodes one scan and returns before/after thumbnails from a single read. */
export function buildPreview(path: string, maxEdge: number): Promise<Preview> {
  return invoke("build_preview", { path, maxEdge });
}

/** Develops `paths` and writes the results; progress arrives as events. */
export function developBatch(
  paths: string[],
  settings: OutputSettings,
): Promise<BatchReport> {
  return invoke("develop_batch", { paths, settings });
}

/** Asks a running batch to stop after the images already in flight. */
export function cancelBatch(): Promise<void> {
  return invoke("cancel_batch");
}

/** The folder used until the user picks one. */
export function defaultOutputDir(): Promise<string> {
  return invoke("default_output_dir");
}

/** Files or folders named on the command line when the app was launched. */
export function startupPaths(): Promise<string[]> {
  return invoke("startup_paths");
}

/** Longest edge, in pixels, of the thumbnails shown in the grid and list. */
export const CARD_PREVIEW_EDGE = 720;

/** Longest edge of the larger preview the fullscreen viewer requests. */
export const FULL_PREVIEW_EDGE = 2000;
