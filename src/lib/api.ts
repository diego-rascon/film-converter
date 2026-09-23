import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  BatchProgress,
  BatchReport,
  ImageMetadata,
  ImportedImage,
  OutputSettings,
  Preview,
} from "$types";

/** Expands dropped or picked paths into a flat list of supported images. */
export function importPaths(paths: string[]): Promise<ImportedImage[]> {
  return invoke("import_paths", { paths });
}

/** The extensions `importPaths` accepts, for a file dialog's filter. */
export function supportedExtensions(): Promise<string[]> {
  return invoke("supported_extensions");
}

/** Decodes one scan and returns before/after thumbnails from a single read. */
export function buildPreview(path: string, maxEdge: number): Promise<Preview> {
  return invoke("build_preview", { path, maxEdge });
}

/** One file's properties, read from its header without decoding it. */
export function imageMetadata(path: string): Promise<ImageMetadata> {
  return invoke("image_metadata", { path });
}

/**
 * Develops `paths` and writes the results. `onprogress` hears about each
 * image as it lands, over a channel that belongs to this one run; the
 * promise settles with the report once the run is over.
 */
export function developBatch(
  paths: string[],
  settings: OutputSettings,
  onprogress: (progress: BatchProgress) => void,
): Promise<BatchReport> {
  return invoke("develop_batch", {
    paths,
    settings,
    progress: new Channel<BatchProgress>(onprogress),
  });
}

/** Asks a running batch to stop after the images already in flight. */
export function cancelBatch(): Promise<void> {
  return invoke("cancel_batch");
}

/** The folder the output dialog opens in until the user has picked one. */
export function defaultOutputDir(): Promise<string> {
  return invoke("default_output_dir");
}

/** Files or folders named on the command line when the app was launched. */
export function startupPaths(): Promise<string[]> {
  return invoke("startup_paths");
}
