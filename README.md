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

Drop images or a folder onto the window, or use **Add** (images or a folder).
Sub-folders are searched too.

- **Grid or list**, with a size slider for the grid. Thumbnails on screen are
  decoded first, so a large import fills in where you are looking.
- **Pick** images the way a file manager does: a click picks one, Ctrl/Cmd-click
  adds to the selection, Shift-click takes the run, and a click on the empty
  background lets go. The checkboxes build a selection without a key held.
  **Ctrl/Cmd + A** picks everything, **Del** removes the selection, **Esc**
  clears it.
- **Double-click** an image (or press Enter on it) to open it full screen.
  **Before**, **Compare** and **After** switch the view; drag the divider to
  wipe between the scan and the result, and scroll or press **+ −** to zoom.
  **← →** move between images, **B** flips before/after, **I** shows the
  details, **Del** removes, **Esc** closes.
- **Save…** asks where to write, then develops the selection — or everything,
  when nothing is picked.
- The **gear** holds the format, quality, overwrite and theme; the
  **hamburger** holds Credits and About.

Previews are generated from the full-resolution file but the clipping points
are measured before downscaling, so what you see is what gets written.

## Output

Files are written as `<name>_positive.<ext>`. If that name is taken, a
numbered copy is written instead — turn on **Overwrite existing files** to
replace it. Two scans that share a name in one run, like frame 01 of two
rolls, always get a file each.

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
src/                        Svelte 5 front end
  app/                      the one route
  components/               UI more than one feature draws
  features/                 the roll, the shell and the viewer, a folder each
  lib/api.ts                typed wrappers over the Rust commands
  lib/files.ts              the desktop dialogs and the file manager
  state/session.svelte.ts   the roll: images, order, selection, viewer
  state/batch.svelte.ts     a develop run and its progress
  state/notices.svelte.ts   the message on screen
  state/settings.svelte.ts  preferences, persisted to local storage
  hooks/ types/ utils/      shared behaviour, shapes and helpers
  styles/                   tokens, reset, motion, shared controls
src-tauri/src/
  commands.rs               the commands the front end calls
  batch.rs                  a develop run: output names, parallel develop
  preview.rs                before/after previews from one decode
  discovery.rs              supported formats and the folder walk
  image_io.rs               decode, downscale, encode, save
  processing.rs             measure (percentiles) and develop (one table pass)
  error.rs                  the one error type
```

## Tests

```sh
pnpm test                     # cargo test — 38 tests
pnpm lint                     # clippy, pedantic, warnings as errors
pnpm check                    # svelte-check
```

The processing tests are the ones that matter. `percentile` has to match
`np.percentile` exactly, down to the last bit: numpy computes its virtual
index as `(q / 100) * (n - 1)` and interpolates from the upper sample once the
fraction reaches 0.5. Writing either of those the other way round moves a
clipping point by a fraction of a level, which is enough to change the
developed pixels after `astype(uint8)` truncates. Two regression tests pin
both down.

The port never builds the inverted image — it reads the inversion's
percentiles off the scan's own histogram, reversed, and folds the inversion
into the stretch's lookup table — and a third test holds that to a literal
invert, sort, stretch reference, byte for byte.

This port was checked against the Python on 708 generated images: every one
produced byte-identical output, and the written PNG and TIFF files matched
PIL's byte for byte.

## Credits

Original idea by Adrián Rascón. Developed by Diego Rascón and Adrián Rascón.
