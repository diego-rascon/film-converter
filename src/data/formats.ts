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
