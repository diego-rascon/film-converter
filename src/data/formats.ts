import type { OutputFormat } from "$types";

/**
 * The containers the app reads. `import_paths` on the Rust side accepts
 * exactly these, and the file dialogs filter to the same set — one list, so
 * a dialog cannot offer a file the importer would then skip.
 */
export const IMAGE_EXTENSIONS = [
  "jpg",
  "jpeg",
  "png",
  "tif",
  "tiff",
  "bmp",
  "webp",
];

/**
 * What it writes: the output format setting's choices, in the order they are
 * offered. The settings validate a stored format against these values.
 */
export const OUTPUT_FORMATS: readonly {
  value: OutputFormat;
  label: string;
  note: string;
}[] = [
  { value: "jpg", label: "JPEG", note: "4:4:4, no chroma subsampling" },
  { value: "png", label: "PNG", note: "Lossless; quality sets compression" },
  { value: "tiff", label: "TIFF", note: "Lossless, uncompressed" },
];
