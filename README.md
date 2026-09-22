# Film Converter

A desktop app for batch-developing scanned colour negatives. Drop a roll of
scans in, get positives out.

It is a Rust + Svelte rewrite of `film_converter_v2.py`, and the image
pipeline is a faithful port: for the same input file and settings it writes
the same pixels the Python script did.

## How it develops an image

1. **Invert** the scan — `255 - value`.
2. **Measure** black and white points per channel: the 0.5th and 99.5th
   percentiles of the inverted image.
3. **Stretch** each channel between its own two points, clipped to 0–255.

Because red, green and blue each get their own black and white point, the
orange mask of a colour negative is removed as a side effect.

A channel whose two points are less than one level apart is left alone, the
same guard the Python has.

## Running it

```sh
pnpm install
pnpm tauri dev          # development, with hot reload
pnpm tauri build        # a release bundle in src-tauri/target/release/bundle
```

Scans can also be named on the command line, so the app can be wired up as the
"open with" handler for image files:

```sh
film-converter roll-01/*.tif
```

## Using it

Drop images or a folder onto the main panel, or use **Add** / **Folder**.
Sub-folders are searched too.

- **Grid or list**, with a size slider for the grid.
- **Before / after** for the whole set from the toolbar, or per image in the
  viewer. Click any thumbnail to open it full screen; drag the divider to wipe
  between the scan and the result.
- **← →** move between images, **B** flips before/after, **Del** removes,
  **Esc** closes.
- **Select** images with their checkboxes (shift-click extends a range) to
  develop or remove only those. **Ctrl/Cmd + A** selects everything.
- The **gear** holds the output folder, format and quality; the **hamburger**
  holds Credits and About.

Previews are generated from the full-resolution file but the clipping points
are measured before downscaling, so what you see is what gets written.

## Output

Files are written as `<name>_positive.<ext>`. If that name is taken, a
numbered copy is written instead — turn on **Overwrite existing files** to
replace it.

| Format | Quality setting                                              |
| ------ | ------------------------------------------------------------ |
| JPEG   | 50–100, encoded 4:4:4 so colour detail survives              |
| PNG    | Lossless; the slider picks a deflate level (`(100 - q) / 10`) |
| TIFF   | Lossless and uncompressed; the slider does nothing            |

Reads JPEG, PNG, TIFF, BMP and WebP. EXIF orientation is applied on load,
which the Python did not do — negatives shot with a camera are often tagged
sideways, and rotating cannot affect the colour pipeline.

## Layout

```
src/                       Svelte 5 front end
  app/                     the one route
  components/              UI more than one feature draws
  features/                the roll, the shell and the viewer, a folder each
  lib/api.ts               typed wrappers over the Rust commands
  lib/files.ts             the desktop dialogs and the file manager
  state/session.svelte.ts  loaded images, selection, batch progress
  state/settings.svelte.ts preferences, persisted to local storage
  hooks/ types/ utils/     shared behaviour, shapes and helpers
  styles/                  tokens, reset, motion, shared controls
src-tauri/src/
  processing.rs            invert, percentiles, per-channel stretch
  image_io.rs              decode, downscale, encode, output naming
  commands.rs              the commands the front end calls
```

## Tests

```sh
cd src-tauri && cargo test    # 21 tests
pnpm check                    # svelte-check
```

The processing tests are the ones that matter. `percentile` has to match
`np.percentile` exactly, down to the last bit: numpy computes its virtual
index as `(q / 100) * (n - 1)` and interpolates from the upper sample once the
fraction reaches 0.5. Writing either of those the other way round moves a
clipping point by a fraction of a level, which is enough to change the
developed pixels after `astype(uint8)` truncates. Two regression tests pin
both down.

This port was checked against the Python on 708 generated images: every one
produced byte-identical output, and the written PNG and TIFF files matched
PIL's byte for byte.

## Credits

Original idea by Adrián Rascón. Developed by Diego Rascón and Adrián Rascón.
