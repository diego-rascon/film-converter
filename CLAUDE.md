# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
pnpm install
pnpm tauri dev                 # run the app (starts vite on :1420, then cargo)
pnpm tauri build               # release bundle in src-tauri/target/release/bundle

pnpm check                     # svelte-check (runs svelte-kit sync first)
cargo test --manifest-path src-tauri/Cargo.toml            # 22 tests
cargo test --manifest-path src-tauri/Cargo.toml percentile # a single test, by name substring
```

`pnpm dev` alone serves the front end in a browser, but every `invoke` fails there — the
UI is useless without the Rust side. Use `pnpm tauri dev`.

## What this is

A Tauri 2 desktop app that batch-develops scanned colour negatives. It is a rewrite of
`film_converter_v2.py` (kept at the repo root as the reference), and **the image pipeline
is a byte-exact port** — see the invariant below before touching `processing.rs`.

## Architecture

**Front end** — SvelteKit in SPA mode (`adapter-static` with an `index.html` fallback,
`ssr = false` in [+layout.ts](src/routes/+layout.ts)). Vite builds to `build/`, which
`tauri.conf.json` serves as `frontendDist`. There is one route,
[+page.svelte](src/routes/+page.svelte).

State lives in two singleton classes using Svelte 5 runes, exported as instances:

- [session.svelte.ts](src/lib/session.svelte.ts) — the loaded images, their order,
  selection, viewer index, batch progress, and the preview queue (4 decodes at a time).
  `images` is the one order the app has — the grid, the viewer's index and shift-click
  ranges all read it — so `sortBy` reorders the array itself rather than handing the list
  a second view of the same images. `ImageItem.sequence` keeps the import order
  recoverable, which is what a third click on a column goes back to.
- [settings.svelte.ts](src/lib/settings.svelte.ts) — output folder/format/quality plus
  theme and view mode, persisted to local storage. Nothing auto-saves: callers invoke
  `settings.save()` explicitly after a change. `directory` is not a setting the user
  edits: **Save…** asks for the destination on every run and stores the answer there,
  where it seeds the next dialog and is what **Show output** opens.

[api.ts](src/lib/api.ts) holds one thin typed wrapper per Rust command and nothing else;
components never call `invoke` directly. [types.ts](src/lib/types.ts) mirrors the serde
structs in `commands.rs`, which all use `rename_all = "camelCase"`.

**Rust side** — three modules under [src-tauri/src/](src-tauri/src/):

- [processing.rs](src-tauri/src/processing.rs) — invert, per-channel percentiles, stretch.
  Pure functions over `RgbImage`, no I/O.
- [image_io.rs](src-tauri/src/image_io.rs) — decode, recursive folder walk, downscale,
  encode, output naming, and `probe`, which reads a file's header for the properties
  dialog without decoding its pixels.
- [commands.rs](src-tauri/src/commands.rs) — the seven `#[tauri::command]`s, all registered
  in [lib.rs](src-tauri/src/lib.rs).

Adding a command means touching four places: the function in `commands.rs`, the
`invoke_handler!` list in `lib.rs`, a wrapper in `api.ts`, and its types in `types.ts`.

## The byte-exactness invariant

`percentile` in `processing.rs` must match `np.percentile` **bit for bit**, because a
clipping point that moves by a fraction of a level changes the developed pixels once
`astype(uint8)` truncates. Two things are load-bearing and look like they could be
simplified but cannot:

- the virtual index is `(q / 100) * (n - 1)`, in that association — not `(n - 1) * q / 100`;
- `lerp` interpolates from the *upper* sample once `t >= 0.5`, from the lower one below it.

`virtual_index_keeps_numpys_association` and `lerp_switches_formula_above_the_midpoint`
pin both down with values where the wrong form differs in the last bit. Similarly,
`stretch_lut` truncates via `as u8` rather than rounding, and a channel with
`high - low < 1.0` is left untouched. The port was verified against the Python on 708
generated images with byte-identical output; keep it that way.

Deliberate departures from the Python, both safe: EXIF orientation is applied on load
(rotation cannot affect the colour pipeline), and the output name gets a numbered suffix
instead of clobbering unless **Overwrite** is on.

## Preview pipeline

Previews are honest: `build_preview` decodes the full-resolution file, measures the
clipping points on it (`invert_and_measure`), then applies those levels to a *downscaled*
copy (`develop_with_levels`). Measuring after downscaling would show the user something
different from what gets written. Both preview images come back as JPEG `data:` URLs from
a single decode — the CSP in `tauri.conf.json` allows `data:` in `img-src`, so any new
image source needs that CSP updated too.

Cards request `CARD_PREVIEW_EDGE` (720px); the fullscreen viewer re-requests at
`FULL_PREVIEW_EDGE` (2000px).

## The custom window frame

The window is created with `decorations: false`, so the app draws its own
titlebar: [AppHeader.svelte](src/lib/components/AppHeader.svelte) is the titlebar,
and [WindowControls.svelte](src/lib/components/WindowControls.svelte) holds the
minimise/maximise/close buttons at its right end. Three consequences:

- Anything that wants to drag the window needs `data-tauri-drag-region` on the
  element the pointer actually lands on — Tauri checks the event target itself,
  not its ancestors. Hence the attribute on the header *and* on the viewer's
  title spans, and `pointer-events: none` on the brand.
- [Viewer.svelte](src/lib/components/Viewer.svelte) is a fullscreen overlay that
  covers the header, so it repeats the drag region and the window buttons.
  Any new full-window overlay has to do the same or the window becomes
  unmovable while it is open.
- GTK stops handling the resize border on an undecorated window, so
  [WindowResizeEdges.svelte](src/lib/components/WindowResizeEdges.svelte) draws
  its own 4px grips and calls `startResizeDragging`. They sit at `z-index: 70`,
  above the viewer (50) and the modals (60), which caps how high anything else
  may go. They also overlay the window's outermost pixels, so the headers keep a
  10px gutter: a control flush to the edge would have its corner swallowed by a
  grip.

The buttons, the drag region and the grips each need their own permission in
[capabilities/default.json](src-tauri/capabilities/default.json) —
`core:default` grants the *queries* (`is-maximized`) but none of the actions.

## Batch runs

`develop_batch` fans out over rayon and emits a `develop://progress` event per image,
which `+page.svelte` forwards to `session.applyProgress`. Cancellation is a shared
`AtomicBool` in the managed `BatchControl` state: images already in flight finish, and
the front end resets anything still marked `developing` back to `pending`.

## Front-end conventions

- Svelte 5 runes throughout (`$state`, `$props`, `$derived`). No stores, no
  `createEventDispatcher` — components take callback props (`onopen`, `onremove`,
  `ontoggleSelect`).
- Styling is plain CSS with the design tokens defined in [app.css](src/app.css); dark mode
  is `:root[data-theme="dark"]`, set from `settings.applyTheme()`. The theme preference is
  `auto` by default, and `app.css` carries no `prefers-color-scheme` query, so `auto` is
  resolved in JS: `settings.resolvedTheme` is what reaches the attribute, and a `matchMedia`
  listener repaints when the system flips mid-session. Surfaces are
  deliberately neutral grey so chrome does not bias how developed colours look.
  Light mode carries two accents because the one colour has two jobs: `--accent` is the
  accent drawn *as* text or a border on a white surface, and `--accent-solid` is the
  accent *filled* under `--accent-text`, which wants more light in it. `.btn-primary`
  (**Save…** and the drop zone's button) is the filled one. Dark mode points
  `--accent-solid` back at `--accent`, whose lighter value already fills well.
- The toolbar's left cluster is the selection's actions — properties, show in folder,
  remove — the same three an image's own [ImageMenu](src/lib/components/ImageMenu.svelte)
  carries, as icons. They are always there and go disabled without a selection rather
  than appearing with one, so the toolbar never shifts under the pointer; properties
  needs exactly one image, the other two take any number. **Remove** is muted at rest
  and only turns red under the pointer, because it is on screen the whole time.
- [ViewHeader.svelte](src/lib/components/ViewHeader.svelte) is the bar above the images
  in *both* views, and the only place select-all and the sort controls live — the toolbar
  deliberately carries neither. It reads `settings.viewMode` itself: in list view the sort
  controls double as column headings and take the column widths, in grid view they are
  plain buttons. Its list columns and [ImageRow.svelte](src/lib/components/ImageRow.svelte)
  have to agree: the widths are `--col-*` custom properties set on `.list` in
  [+page.svelte](src/routes/+page.svelte), but the side padding is not, so both hard-code
  the same `18px`/`20px`. A column added to one needs the same slot in the other.
- The header sits *inside* the scroll container in both views so it can be `sticky`.
  In grid view that puts it in the path of the backdrop click that clears the selection,
  which is why that handler tests `event.target === event.currentTarget` rather than
  having children stop propagation.
- List rows are deliberately full-bleed — no gutter on `.list`, no gap, no radius — so
  the zebra stripes run as continuous bands the way Finder's do. That is also what lets
  the sticky header hide the rows passing under it, and what makes a run of selected
  rows read as one block. The stripe is `--stripe`, an overlay rather than a fourth
  surface colour, and its parity comes from the `striped` prop rather than
  `:nth-child`, which the header would otherwise throw off by one.
- `--gutter` is the one left margin in the app: the brand in the titlebar, the toolbar's
  actions, the status bar's tally, the cards, the rows and the header checkbox all sit on
  it, which is why the checkbox does not move when the view is switched. A cluster pays
  for whatever its first control hangs left of its ink — the toolbar spends
  `calc(var(--gutter) - 9px)`, half the difference between its 34px icon buttons and
  their 16px glyphs. `input[type="checkbox"]` has its UA margin zeroed in
  [app.css](src/app.css) for the same reason — it would otherwise sit 4px inside.
- Popovers — the header's settings and menu, the toolbar's **Add** — are absolutely
  positioned inside a `position: relative` host marked `data-popover`. Their owner closes
  them on a window `pointerdown` whose target has no `[data-popover]` ancestor, and
  swallows `Escape` in the *capture* phase so the same key does not also reach
  [+page.svelte](src/routes/+page.svelte)'s shortcut handler and clear the selection.
  [ImageMenu.svelte](src/lib/components/ImageMenu.svelte) — the three-dot menu that
  carries a single image's actions in *both* views — follows the same etiquette with two
  deliberate differences, because there is one of them per image and a session holds
  hundreds: its window listeners are bound by an `$effect` only while it is open, and
  "outside" means outside its own host rather than outside any `[data-popover]`, so
  pressing one image's button closes the menu another image left open. It carries no
  `data-popover` attribute for the same reason — opening it should dismiss the toolbar's.
  On a card it has to sit *outside* `.frame`, whose `overflow: hidden` would clip it.
  Adding files goes through the toolbar's one **Add** menu (images or a folder); the
  [DropZone](src/lib/components/DropZone.svelte) still offers both as separate buttons
  because it has the room and nothing else to show.
- Icons are stroked SVG paths in a single map inside
  [Icon.svelte](src/lib/components/Icon.svelte); add a path there rather than inlining SVG.
- Drag and drop uses Tauri's `getCurrentWebview().onDragDropEvent`, not the HTML5 API —
  only the Tauri event carries real filesystem paths.
- New Tauri plugin permissions must be added to
  [capabilities/default.json](src-tauri/capabilities/default.json) or the call is denied at
  runtime.

## Build notes

`image` is built with `default-features = false` to keep AVIF (ravif/dav1d) out of the
dependency tree. JPEG encoding uses the separate `jpeg-encoder` crate because `image`'s
encoder cannot disable chroma subsampling, and the Python writes 4:4:4 (`subsampling=0`).
`[profile.dev.package."*"] opt-level = 3` is there because debug-build image decoding is
otherwise unusably slow.
