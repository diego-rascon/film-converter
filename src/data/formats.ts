import type { OutputFormat } from "$types";

/**
 * The containers the app reads, as a person would name them — for the empty
 * window and the About dialog. Which files the importer actually takes is
 * the Rust side's own list of extensions, and the file dialog asks it for
 * that list rather than keeping a copy here that could drift.
 */
export const INPUT_FORMAT_NAMES = ["JPEG", "PNG", "TIFF", "BMP", "WebP"] as const;

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
