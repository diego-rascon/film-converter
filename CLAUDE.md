# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

**Film Converter** is a desktop app that turns scanned colour negatives into positives,
a whole roll at a time: drop the scans in, compare before and after, save the positives
out.

It is a Tauri 2 app (Rust back end, SvelteKit + Svelte 5 front end) and a rewrite of
`film_converter_v2.py`, a Tkinter script kept at the repo root as the reference. **The
image pipeline is a byte-exact port of that script.** Read
[the byte-exactness invariant](#the-byte-exactness-invariant) before touching
`processing.rs`.

Each image is developed in three steps:

1. **Invert:** `255 - v`.
2. **Measure** a black and a white point per channel: the 0.5th and 99.5th percentiles
   of the inverted image.
3. **Stretch** each channel between its own two points, clip to 0–255 and truncate to
   `u8`. A channel whose points are less than one level apart is left alone.

Giving each channel its own points is what removes the orange mask.

## Commands

```sh
pnpm install
pnpm tauri dev      # run the app (vite on :1420, then cargo)
pnpm tauri build    # release bundle in src-tauri/target/release/bundle

pnpm check          # svelte-check (runs svelte-kit sync first)
pnpm test           # cargo test
pnpm lint           # clippy, pedantic, warnings as errors
cargo test --manifest-path src-tauri/Cargo.toml percentile   # one test, by name substring
```

- `pnpm dev` on its own serves the front end in a browser, but every `invoke` fails
  there. Use `pnpm tauri dev`.
- Every dev session shares webview storage (origin `localhost:1420`), so settings,
  including the last output folder, carry over from one run to the next.
- Before finishing a change, run `pnpm check` for the front end and
  `pnpm test && pnpm lint` for Rust.

## Tech stack

| Layer     | What                                                                                                                         |
| --------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Shell     | Tauri 2 with the `dialog` and `opener` plugins, and a custom window frame (`decorations: false`)                             |
| Back end  | Rust, edition 2024, MSRV 1.88: `image` 0.25 (no AVIF), `jpeg-encoder` (4:4:4), `fast_image_resize`, `rayon`, `thiserror`, `base64` |
| Front end | SvelteKit 2 as an SPA (`adapter-static`, `ssr = false`), Svelte 5 runes only, TypeScript, Vite                               |
| Styling   | Plain CSS on custom-property tokens, with no CSS framework or component library                                              |
| Tooling   | pnpm, clippy pedantic, svelte-check                                                                                          |

## Features

What the user can do. Check this list when a change touches behaviour.

- **Import:** drop files or folders on the window, use the titlebar's **Add** menu
  (images or a folder), or pass paths on the command line (`film-converter roll-01/*.tif`,
  which is how it works as an "open with" handler). Folders are searched recursively. It
  reads JPEG, PNG, TIFF, BMP and WebP.
- **Browse** in a grid with a thumbnail-size slider, or in a list. Sort by name or size;
  a third click on a column goes back to import order. A toolbar toggle flips every
  thumbnail between the scan and the result.
- **Select** the way a file manager does: click, Ctrl/Cmd-click, Shift-click, the
  checkboxes, or a click on the empty background to let go. `Ctrl/Cmd+A` selects
  everything, `Delete` removes the selection and `Esc` clears it.
- **Act on images:** each image's ⋯ menu, and the toolbar for a selection, offer
  properties, show in folder and remove. **Clear** in the titlebar empties the session.
- **Viewer** (double-click or `Enter`): before, compare (drag a wipe divider) and after
  modes; zoom and pan with the wheel or `+` `−` `0`. `←` `→` step through images, `B`
  flips before/after, `I` toggles a details sidebar, `Delete` removes the image and `Esc`
  closes the viewer.
- **Save…** asks for a folder on every run, then develops the selection (or everything
  when nothing is selected) in parallel, with live progress and **Cancel**.
  **Show output** opens the last folder.
- **Settings** (gear): output format (JPEG, PNG or TIFF), quality 50–100 (compression
  level for PNG), overwrite, and theme (auto, light or dark). All of them persist.
- **Output names:** `<name>_positive.<ext>`. A name that already exists gets a numbered
  suffix unless **Overwrite** is on.

## How it flows

1. **Import:** a drop, a dialog or the command line hands paths to `session.add`, which
   calls `import_paths`. `discovery.rs` walks folders, keeps the supported extensions,
   dedupes by resolved path and walks a symlink loop only once. Each file comes back as
   an `ImportedImage` and becomes an `ImageItem`.
2. **Preview:** each tile requests a 720px pair (`build_preview`) through the session's
   `TaskQueue`, four at a time, with tiles on screen first. `preview::build` decodes the
   full file, measures the clipping points on it, applies them to a downscaled copy, and
   returns before and after as JPEG `data:` URLs from that single decode. Measuring
   after downscaling would show something other than what gets written.
3. **Inspect:** the properties dialog and the viewer's details read `image_metadata`,
   which probes the header without decoding pixels. After the viewer has stayed on an
   image for 150ms it requests a sharper 2000px pair, and it keeps the last six.
4. **Develop:** Save… picks a folder, then `batch.develop` calls `develop_batch` with a
   `Channel<BatchProgress>`. `batch::run` plans every output name up front, in the order
   the paths were sent, fans the scans out over rayon and reports each image through a
   callback. The front end marks each image as its result arrives and posts a summary.
   `cancel_batch` sets an `AtomicBool`: images already in flight finish, and the rest
   return to `pending`.

## Layout

### Front end: `src/`

Each top-level folder is one layer with its own path alias in
[svelte.config.js](svelte.config.js), so an import shows which layer it crosses into.
Only siblings inside the same folder import each other relatively.

```
src/
├── app/                routes (kit.files.routes, so there is no src/routes)
│   ├── +layout.ts      ssr = false
│   └── +page.svelte    the shell: bars, the current view, overlays
├── components/         $components: UI that more than one feature draws
│   ├── image/          ImageFacts, StatusChip
│   ├── overlay/        Modal, InfoModal (the properties dialog)
│   ├── window/         WindowControls, WindowResizeEdges
│   └── Icon.svelte     every icon, as stroked SVG paths in one map
├── data/               $data: constants that more than one module reads
│   ├── formats.ts      input format names, output format choices
│   └── preview.ts      preview edges (720 / 2000) and concurrency (4)
├── features/           $features: one slice of the app per folder
│   ├── roll/           the loaded images: DropZone, ImageGrid + ImageCard,
│   │                   ImageList + ImageRow, ViewHeader, ImageMenu, utils/tile.ts
│   ├── shell/          the chrome: AppHeader, Toolbar, StatusBar, NoticePanel,
│   │                   SettingsPanel, AboutModal, CreditsModal
│   └── viewer/         Viewer, ViewerHeader, ViewerStage, ViewerDetails, ViewerPod,
│                       state/zoom.svelte.ts (ZoomPan), state/sharp.svelte.ts
├── hooks/              $hooks: popover (dismissOnOutside, PopoverGroup), listening,
│                       visibility, focus (holdFocus)
├── lib/                $lib: api.ts (one wrapper per command), files.ts (dialogs, opener)
├── state/              $state: session, batch, notices, settings
├── styles/             $styles: app.css imports tokens, base, motion, controls
├── types/              $types: ipc.ts (mirrors the Rust DTOs), image.ts (ImageItem)
└── utils/              $utils: format.ts, task-queue.ts (framework-free)
```

**Where a new file goes:** a feature owns everything that only it uses, in its own
`components/`, `types/`, `utils/`, `state/` or `data/`, mirroring the top level. When
something outside the feature needs a file, the file moves up to the top level.
**Nothing imports across features**; `+page.svelte` is the only file that names two of
them. Constants follow the same rule: shared ones go in `data/`, and a number that only
one component tunes stays in that component.

### State: four rune singletons

These live in `src/state/` and are exported as instances. Dependencies run one way,
`session` → `batch` → `notices`, and none of them imports a component.

- **session:** the roll. It holds the images (the app's one order, so sorting reorders
  the array), the selection with `pick(path, modifiers)` and the Shift anchor, `targets`
  (the selection, or everything), the viewer's image *by path*, and the preview queue.
  The counting members (`total`, `selected`, `hasSelection`, `allSelected`) are
  `$derived` because every tile reads them. Add new ones as `$derived` too.
- **batch:** one develop run: `running`, `total` (fixed when the run starts),
  `completed` and the last run's `failures`.
- **notices:** the one toast. `show(kind, text)` replaces whatever is showing, and
  `clear(notice)` clears only the notice it is given.
- **settings:** format, quality, overwrite, theme, view mode, card size and the last
  output directory. They are read from local storage synchronously on load and
  validated field by field. An `$effect.root` writes every change back, so components
  only assign.

### Back end: `src-tauri/src/`

The layers call downward only: commands → app logic → primitives.

| Module          | Role                                                                                                         |
| --------------- | ------------------------------------------------------------------------------------------------------------ |
| `lib.rs`        | the builder, the plugins and `generate_handler!`                                                             |
| `commands.rs`   | the eight `#[tauri::command]`s, thin doorways onto the blocking pool; owns `ImportedImage` and `ImageMetadata` |
| `batch.rs`      | a develop run: the `BatchControl` cancel flag, output-name planning, and `run` with a plain callback (testable without Tauri) |
| `preview.rs`    | the before/after pair from one decode                                                                        |
| `discovery.rs`  | `SUPPORTED_EXTENSIONS` and the folder walk                                                                   |
| `image_io.rs`   | decode, `probe`, downscale, encode and `save_image`                                                          |
| `processing.rs` | `measure` and `develop`: pure functions over `RgbImage`, with no I/O                                         |
| `error.rs`      | the one `Error` enum; each message is written for the user and serializes as a plain string                  |

The commands are `import_paths`, `supported_extensions`, `build_preview`,
`image_metadata`, `develop_batch`, `cancel_batch`, `default_output_dir` and
`startup_paths`.

**Adding a command touches four places:** the function in `commands.rs`, the
`generate_handler!` list in `lib.rs`, a wrapper in `lib/api.ts`, and its types in
`types/ipc.ts`. The Rust DTOs use `rename_all = "camelCase"`. A new plugin permission
also has to go in [capabilities/default.json](src-tauri/capabilities/default.json), or
the call is denied at runtime.

## The byte-exactness invariant

`percentile` in `processing.rs` must match `np.percentile` **bit for bit**: a clipping
point that moves by a fraction of a level changes the output pixels once they are
truncated to `u8`. Two things look like they could be simplified, and cannot be:

- the virtual index is `(q / 100) * (n - 1)`, in that association, not
  `(n - 1) * q / 100`;
- `lerp` interpolates from the *upper* sample once `t >= 0.5`, and from the lower sample
  below that.

`virtual_index_keeps_numpys_association` and `lerp_switches_formula_above_the_midpoint`
test both. `stretch_lut` also truncates with `as u8` and never rounds, and it leaves a
channel with `high - low < 1.0` untouched.

The pipeline never builds the inverted image. `measure` reads the inversion's
percentiles off the scan's histogram in reverse, and `develop_table` combines invert and
stretch into one 256-entry table per channel. `develop_matches_the_three_step_pipeline`
compares that byte for byte against a literal invert → percentile → stretch reference,
and any change to the pipeline has to keep it passing. The port was verified against
the Python on 708 generated images.

There are two deliberate departures from the Python, and both are safe: EXIF orientation
is applied on load, and output names get a numbered suffix instead of overwriting an
existing file, unless Overwrite is on.

**Concurrency in `measure`:** the histogram is counted in parallel only when the caller
is *not* a rayon worker, which is the case for a preview (it runs on the blocking pool).
A batch runs one scan per worker and keeps every pass on that worker's thread, because a
worker waiting on its pieces steals other work, which can pull a whole second scan into
memory. The per-piece counts are boxed because, held by value, they overflowed worker
stacks. `measuring_many_scans_at_once_stays_off_the_workers_stacks` is the regression
test.

## UI

### The window

```
┌─ AppHeader ─────────────────────────────────────────────────────────────┐
│ [Add v] [Clear]        Film Converter        [gear] [menu]   _  []  x   │
├─ Toolbar ───────────────────────────────────────────────────────────────┤
│ [grid|list] [size] [before/after]              [info] [folder] [remove] │
├─ ViewHeader (sticky) ───────────────────────────────────────────────────┤
│ [ ] select all                          sort buttons or column headings │
│                                                                         │
│ ImageGrid of ImageCards, or ImageList of ImageRows                      │
│ (DropZone when the session is empty)                                    │
│                                                     NoticePanel (toast) │
├─ StatusBar ─────────────────────────────────────────────────────────────┤
│ N images                                 [Show output] [Cancel] [Save…] │
└─────────────────────────────────────────────────────────────────────────┘
```

- `+page.svelte` is only the shell; each view owns its own layout. While the viewer or a
  dialog is open, the shell is `inert` and its shortcuts are disabled. Whatever is on
  top handles its own keyboard.
- The titlebar shows **Add** and **Clear** only when images are loaded. The gear opens
  settings, and the menu opens Credits and About.
- The toolbar has *view* controls on the left and *selection* actions on the right:
  properties, show in folder and remove, the same three as in `ImageMenu`. The selection
  actions are always shown and are disabled when nothing is selected, so the toolbar
  never shifts. **Remove** is muted and turns red only on hover.
- `ViewHeader` is the only place with select-all and sorting. When images are selected,
  its right side shows "N images selected" instead. Its height is fixed at 40px so that
  switching views does not make the images jump.
- `NoticePanel` dismisses itself after a few seconds, except for errors, runs with
  failures and runs in progress. Those stay until the user closes them.

### Interaction model

Selection follows file-explorer rules, decided in one place: `session.pick`, with the
modifiers mapped in `roll/utils/tile.ts`. One click selects, a double click opens,
Ctrl/Cmd toggles, Shift extends from the anchor and a plain click replaces the selection.
A click on the backdrop (elements marked `data-backdrop`) or `Esc` deselects. A tile's
`<button>` is its tab stop: `Enter` opens and `Space` selects. Checkboxes always toggle,
and they fade in on hover or whenever anything is selected.

In the list, a row's full width is the hit target. The handlers sit on `.row` and skip
its own controls with `closest(...)`. Do not cover the row with an absolutely positioned
button, because it would swallow the name and status tooltips.

### The viewer

The viewer is a fullscreen overlay rendered beside `main`, with a frosted
(`backdrop-filter`) backdrop over the app, which stays mounted behind it. `Viewer.svelte`
owns the current image, the mode, the sharp pair and the keyboard. The drawing is split
into components:

- `ViewerHeader`: the titlebar.
- `ViewerStage`: the pictures, the wipe and dragging.
- `ViewerDetails`: the sidebar, which uses `ImageFacts dense`.
- `ViewerPod`: a floating glass cluster, used twice (paging bottom-left, zoom
  bottom-right).

Zoom and pan are `ZoomPan` state, shared by the keyboard, the zoom pod and dragging. The
stage binds itself into `zoom.stage` because every measurement uses its rectangle. The
viewer opens in whichever mode (before or after) the grid was showing.

Keep these in place:

- The zoom transform sits *inside* the wipe's clip, so the divider stays a 2px line.
- A drag is a wipe unless the picture is zoomed, in which case it pans. The divider is
  the exception: it can still be dragged while zoomed.
- `clampPan` measures the fitted picture, not the stage.
- The padding belongs to the body, not the stage. Padding on the stage would make the
  measured rectangle drift from the picture.

### Custom window frame

The window is created with `decorations: false`, so the app draws its own titlebar
([AppHeader](src/features/shell/components/AppHeader.svelte)) and window buttons
([WindowControls](src/components/window/WindowControls.svelte)).

- Anything that drags the window needs `data-tauri-drag-region` on the element the
  pointer actually lands on, because Tauri checks the event target, not its ancestors.
- A full-window overlay covers the titlebar, so it has to repeat the drag region and the
  window buttons. The viewer does both.
- GTK stops handling the resize border on an undecorated window, so `WindowResizeEdges`
  draws its own 4px grips. Controls stay at least 10px from the window edge, or a grip
  covers their corner.
- Each window action needs its own permission in `capabilities/default.json`;
  `core:default` grants only the queries.

**Stacking order:** resize grips 70 › modals 60 › viewer 50 › popovers 40 › notice 30 ›
drop overlay 20 › sticky `ViewHeader` 1. Nothing may go above the grips.

**Popovers** are absolutely positioned inside a `position: relative` host marked
`data-popover`. `dismissOnOutside` binds the outside pointerdown and a capture-phase
`Escape` only while the popover is open. The titlebar's popovers are one `PopoverGroup`,
which makes them mutually exclusive. `ImageMenu` (one per image) deliberately has no
`data-popover`, so that opening it closes the others, and it counts anything outside its
own host as outside.

**List rows:** `.row` stays unpositioned, and nothing inside it takes a `z-index`. A
child's `z-index` would escape to the root and paint over the sticky header, and a
`z-index` on `.row` would trap the menu's popover under the next row.

## Style

- **Plain CSS.** `styles/app.css` contains only imports:
  - `tokens.css`: every colour, radius and measure, for both themes.
  - `base.css`: the reset and element defaults, including the system font stack at 14px.
  - `motion.css`: every `@keyframes`, plus the reduced-motion switch. The one exception
    is `dialog-in`, which stays local to `Modal.svelte`.
  - `controls.css`: rules that more than one component uses: `.btn` and its variants,
    `.icon-btn`, `.caps` and `.field-label`, `.segmented`, `.popover` and `.menu`,
    `.shimmer`, `.tile` and checkboxes.

  A component styles only its own elements. A rule moves to `controls.css` once a second
  component needs it.
- **Never restate a shared rule to change one value.** Svelte scopes selectors with
  `:where()`, which adds no specificity, so a local `.segmented button { color }` silently
  overrides the shared `:hover` and `.active` states. Shared rules accept overrides
  through custom properties, for example `var(--segmented-rest, var(--text-muted))`. Give
  any new shared rule the same hook.
- **Surfaces are neutral grey** in both themes, so the chrome does not bias how the
  developed colours look. There is one accent, a burnt orange (`--accent`, `#c75f29`).
  Light mode adds `--accent-solid` for filled buttons (`.btn-primary`); dark mode points
  it back at `--accent`.
- **Theme:** dark mode is `:root[data-theme="dark"]`, set by the settings effect. `auto`
  is resolved in JS with `MediaQuery` (`settings.resolvedTheme`), and `tokens.css` has no
  `prefers-color-scheme` query.
- **Glass:** anything floating over a photograph (the card checkbox, the viewer's tags
  and pods) uses `--glass`, `--glass-hover` and `--glass-blur`. These tokens are not
  theme-scoped; use them rather than a new rgba.
- **`--gutter` is the one margin** between the window edge and the content, at both ends
  of every bar. What sits on the gutter line is a control's visible edge. A filled or
  bordered button gets exactly `var(--gutter)`. An icon in an oversized box subtracts the
  box's empty space: the toolbar uses `calc(var(--gutter) - 9px)` and the list uses
  `- 6px`. Work out a new control's offset rather than copying a number. The status bar's
  height is derived from `--action-height` (28px) plus two gutters, so shrink the button
  before you shorten the bar.
- **Shapes:** `--radius-sm` is 6px, `--radius` 10px and `--radius-lg` 14px. A fully
  rounded end marks a chip or a label, never a control.
- **The list and its header must line up.** `ViewHeader` and `ImageRow` share the
  `--col-*` widths set on `.list` in `ImageList`, but each writes out its own side padding
  and checkbox margin. Changing one means changing the other.
- **Icons** are paths in the map in `Icon.svelte`. Add new ones there; never inline an
  SVG.

## Front-end conventions

- Use Svelte 5 runes only. `compilerOptions.runes` turns legacy syntax into a build
  error. No stores and no `createEventDispatcher`; components take callback props
  (`onopen`, `onremove`, …).
- Constants a component reads go in `<script module>`, so they are built once rather
  than once per instance. The per-image components render hundreds of times.
- State that is only ever replaced whole (a notice, the failures, metadata) is
  `$state.raw`. A value that resets when something else changes but can also be set by
  hand is a writable `$derived`, not an `$effect`.
- Behaviour attached to an element is an attachment (`{@attach holdFocus}`,
  `previewPriority`). Manual listeners use `on` from `svelte/events`.
- Components reach outside the app only through `lib/api.ts` and `lib/files.ts`.
  `files.ts` reports failures as notices instead of throwing.
- Drag and drop uses Tauri's `onDragDropEvent`, which carries real paths, not the HTML5
  API.
- Previews are `data:` URLs on purpose; object URLs were measured and rejected. The CSP
  in `tauri.conf.json` allows `data:` in `img-src`, so any new image source needs the CSP
  updated as well.

## Build notes

- `image` is built with `default-features = false` to keep AVIF (ravif/dav1d) out, and
  `fast_image_resize` declares its own `image` dependency the same way.
- JPEG output goes through `jpeg-encoder`, because `image` cannot turn off chroma
  subsampling and the reference writes 4:4:4. Its `simd` feature stays off because it
  changes the encoded bytes.
- The decoder's memory cap is raised to 4 GiB so large 16-bit scans can load.
- `[profile.dev.package."*"] opt-level = 3` is set because image decoding in debug
  builds is otherwise unusably slow.
- The lints live in `Cargo.toml`: `unsafe_code` is forbidden, and clippy runs pedantic
  with a few deliberate exceptions (image-arithmetic casts, exact float comparisons,
  by-value Tauri arguments). `pnpm lint` fails on any warning.
