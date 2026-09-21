# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
pnpm install
pnpm tauri dev                 # run the app (starts vite on :1420, then cargo)
pnpm tauri build               # release bundle in src-tauri/target/release/bundle

pnpm check                     # svelte-check (runs svelte-kit sync first)
cargo test --manifest-path src-tauri/Cargo.toml            # 21 tests
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

- [session.svelte.ts](src/lib/session.svelte.ts) — the loaded images, selection, viewer
  index, batch progress, and the preview queue (4 decodes at a time).
- [settings.svelte.ts](src/lib/settings.svelte.ts) — output folder/format/quality plus
  theme and view mode, persisted to local storage. Nothing auto-saves: callers invoke
  `settings.save()` explicitly after a change.

[api.ts](src/lib/api.ts) holds one thin typed wrapper per Rust command and nothing else;
components never call `invoke` directly. [types.ts](src/lib/types.ts) mirrors the serde
structs in `commands.rs`, which all use `rename_all = "camelCase"`.

**Rust side** — three modules under [src-tauri/src/](src-tauri/src/):

- [processing.rs](src-tauri/src/processing.rs) — invert, per-channel percentiles, stretch.
  Pure functions over `RgbImage`, no I/O.
- [image_io.rs](src-tauri/src/image_io.rs) — decode, recursive folder walk, downscale,
  encode, output naming.
- [commands.rs](src-tauri/src/commands.rs) — the six `#[tauri::command]`s, all registered
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
  is `:root[data-theme="dark"]`, set from `settings.applyTheme()`. Surfaces are
  deliberately neutral grey so chrome does not bias how developed colours look.
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
