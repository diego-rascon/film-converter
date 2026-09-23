<div align="center">

# Film Converter

**Batch-develop scanned colour negatives into positives.**

Drop in a roll of scans, compare every frame before and after, and save the whole roll
in one go.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)
![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)

</div>

---

## About

A scanned colour negative doesn't look like a photo: the tones are inverted and the
whole frame sits under the film's orange mask. Film Converter removes both. It
balances every frame on its own and writes the finished positives to a folder you
choose.

The app started as a small Python script, [`film_converter_v2.py`](film_converter_v2.py),
which is still in the repository. Film Converter is a native desktop rewrite of it in
Rust and Svelte, and its colour pipeline is a faithful port: for the same scan and
settings, it writes exactly the same pixels the script did.

## Features

- **Whole rolls at once.** Drop files or folders on the window (folders are searched
  recursively), use the **Add** menu, or pass files on the command line.
- **Accurate previews.** Every thumbnail shows the developed result, balanced using
  the full-resolution scan, so the preview matches the file that gets saved.
- **Before and after.** Flip the whole grid between scans and results, or open any
  frame full screen with a draggable comparison wipe, zoom and pan.
- **Grid and list views** with sorting and file-manager-style selection.
- **Parallel developing** across all your CPU cores, with live progress and a cancel
  button.
- **JPEG, PNG or TIFF output.** JPEG is written without chroma subsampling (4:4:4), so
  fine colour detail survives.
- **Safe by default.** Existing files are never overwritten unless you turn that on.
- **Light and dark themes** that follow your system by default. Both use neutral grey
  surfaces, so the interface doesn't tint how you judge colour.
- **Reads** JPEG, PNG, TIFF, BMP and WebP, and applies EXIF orientation on load.

## How it works

Each scan goes through three steps:

1. **Invert** the image: every value `v` becomes `255 - v`.
2. **Measure** a black point and a white point for each of the red, green and blue
   channels: the 0.5th and 99.5th percentiles of the inverted image.
3. **Stretch** each channel between its own two points, clipped to 0–255.

Because each channel gets its own black and white point, the orange mask disappears
as a side effect, and every frame is balanced from its own content. If a channel's two
points are less than one level apart, that channel is left as it is.

## Installation

### Build from source

You'll need:

- [Rust](https://rustup.rs/) 1.88 or newer
- [Node.js](https://nodejs.org/) 20.19+ or 22.12+
- [pnpm](https://pnpm.io/installation)
- The system dependencies Tauri needs on your platform (WebKitGTK on Linux, for
  example). Follow [Tauri's prerequisites guide](https://v2.tauri.app/start/prerequisites/).

```sh
git clone https://github.com/diego-rascon/film-converter.git
cd film-converter
pnpm install
pnpm tauri build
```

The installers are written to `src-tauri/target/release/bundle/`: `.deb`, `.rpm` and
`.AppImage` on Linux, `.dmg` on macOS, and `.msi` on Windows. Film Converter is
developed on Linux; Tauri can also build it for macOS and Windows.

## Usage

1. **Add your scans.** Drop images or a whole folder onto the window, or use
   **Add → Add images…** or **Add folder…**.
2. **Review them.** The thumbnails fill in with developed previews, starting with the
   ones on screen. Use the before/after toggle in the toolbar to compare, or
   double-click a frame to open it full screen.
3. **Pick frames (optional).** Select the frames you want. If nothing is selected, the
   whole roll is developed.
4. **Save.** Click **Save…** and choose a folder. When the run finishes, **Show
   output** opens that folder.

Output format, quality, overwrite and theme are under the **gear** in the titlebar.

You can also open scans straight from the command line, which lets you set Film
Converter as the "Open with" app for image files:

```sh
film-converter roll-01/*.tif
```

### Keyboard and mouse

| In          | Input                              | Action                             |
| ----------- | ---------------------------------- | ---------------------------------- |
| Grid / list | Click, Ctrl/Cmd-click, Shift-click | Select, add to selection, select a range |
|             | Click on empty space               | Clear the selection                |
|             | Double-click or `Enter`            | Open in the viewer                 |
|             | `Ctrl/Cmd` + `A`                   | Select all                         |
|             | `Delete`                           | Remove the selected images         |
|             | `Esc`                              | Clear the selection                |
| Viewer      | `←` `→`                            | Previous / next image              |
|             | `B`                                | Switch between before and after    |
|             | Drag                               | Move the comparison wipe, or pan when zoomed |
|             | Scroll, `+` `−`                    | Zoom                               |
|             | `0`                                | Reset zoom                         |
|             | `I`                                | Show or hide details               |
|             | `Delete`                           | Remove the image                   |
|             | `Esc`                              | Close the viewer                   |

### Output

Each file is saved as `<name>_positive.<ext>`. If that name is already taken, a
numbered copy is written instead; turn on **Overwrite existing files** to replace it.
When two scans in one run share a name (frame `01` from two different rolls, for
example), each still gets its own file.

| Format | Quality setting                                                    |
| ------ | ------------------------------------------------------------------ |
| JPEG   | 50–100, encoded 4:4:4 so colour detail survives                    |
| PNG    | Lossless; the slider sets the compression level                    |
| TIFF   | Lossless and uncompressed; the slider has no effect                |

## Development

```sh
pnpm install
pnpm tauri dev     # run the app with hot reload
pnpm check         # type-check the front end (svelte-check)
pnpm test          # run the Rust tests
pnpm lint          # clippy (pedantic), warnings as errors
```

`pnpm dev` on its own serves only the front end in a browser, where every call to the
Rust side fails, so use `pnpm tauri dev`.

### Project structure

```
src/                   Svelte 5 front end (SvelteKit, SPA mode)
├── app/               the one route: the app shell
├── features/          roll (the images), shell (the app's chrome), viewer
├── components/        UI shared between features
├── state/             the session, develop runs, notices, settings
├── lib/               typed wrappers for the Rust commands and the desktop dialogs
├── styles/            design tokens, reset, motion, shared controls
└── hooks/ types/ utils/ data/
src-tauri/src/         Rust back end (Tauri 2)
├── commands.rs        the commands the front end calls
├── batch.rs           a develop run: output names, parallel develop, cancel
├── preview.rs         before/after previews from a single decode
├── discovery.rs       supported formats and the folder walk
├── image_io.rs        decode, downscale, encode, save
├── processing.rs      measure (percentiles) and develop
└── error.rs           the error type
```

[CLAUDE.md](CLAUDE.md) has more detailed architecture notes, including the app's flow,
UI conventions and design tokens.

### Matching the original script

The processing tests are the most important ones in the project. Film Converter has to
produce the same pixels as the Python script, so `percentile` must match
`np.percentile` exactly, down to the last bit. Numpy computes its virtual index as
`(q / 100) * (n - 1)` and interpolates from the upper sample once the fraction reaches
0.5. Writing either of those the other way round moves a clipping point by a fraction of
a level, which is enough to change the developed pixels. Regression tests cover both,
and another test compares the optimised single-pass pipeline byte for byte against a
literal invert → sort → stretch reference.

The port was checked against the Python script on 708 generated images, and every one
produced byte-identical output.

## Contributing

Issues and pull requests are welcome. Before you open a pull request, please run
`pnpm check`, `pnpm test` and `pnpm lint`. Any change to `processing.rs` must keep the
byte-exactness tests passing.

## Credits

- **Original idea:** Adrián Rascón
- **Developed by:** Diego Rascón and Adrián Rascón

Built with [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/) and
[SvelteKit](https://svelte.dev/docs/kit), with image processing by
[`image`](https://github.com/image-rs/image),
[`jpeg-encoder`](https://github.com/vstroebel/jpeg-encoder),
[`fast_image_resize`](https://github.com/Cykooz/fast_image_resize) and
[`rayon`](https://github.com/rayon-rs/rayon).

## License

Film Converter is released under the [MIT License](LICENSE).
