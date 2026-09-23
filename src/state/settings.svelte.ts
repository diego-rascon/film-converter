import { MediaQuery } from "svelte/reactivity";

import { OUTPUT_FORMATS } from "$data/formats";
import { defaultOutputDir } from "$lib/api";
import type { OutputFormat, OutputSettings } from "$types";

const STORAGE_KEY = "film-converter.settings";

export type ViewMode = "grid" | "list";
/** `auto` follows the system; the other two override it. */
export type Theme = "auto" | "light" | "dark";
export type ResolvedTheme = Exclude<Theme, "auto">;

/** The output quality slider's range: the Python's 50-100. */
export const QUALITY_RANGE = { min: 50, max: 100 } as const;
/** The thumbnail slider's range: a grid card's minimum width, in pixels. */
export const CARD_SIZE_RANGE = { min: 140, max: 420, step: 10 } as const;

const THEMES: readonly Theme[] = ["auto", "light", "dark"];
const VIEW_MODES: readonly ViewMode[] = ["grid", "list"];

/** The system's own preference, for `auto`. A flip mid-session repaints. */
const systemDark = new MediaQuery("(prefers-color-scheme: dark)");

interface Persisted {
  directory: string;
  format: OutputFormat;
  quality: number;
  overwrite: boolean;
  theme: Theme;
  viewMode: ViewMode;
  cardSize: number;
}

/**
 * User preferences, persisted to local storage so a session picks up where
 * the last one left off. They are read back synchronously as the module
 * loads, so the first frame is already in the right theme and view, and
 * written back by an effect the moment any of them changes — a component
 * only ever assigns.
 */
class Settings {
  /**
   * Where the last run was saved. Not a setting the user edits: Save asks
   * for the destination every time and keeps the answer here, where it
   * seeds the next dialog and is what "Show output" opens.
   */
  directory = $state("");
  format = $state<OutputFormat>("jpg");
  quality = $state(95);
  overwrite = $state(false);

  theme = $state<Theme>("auto");
  viewMode = $state<ViewMode>("grid");
  /** Minimum width of a grid card, in pixels. */
  cardSize = $state(240);

  constructor() {
    this.#restore();

    // For the life of the app: the painted theme follows the preference and
    // the system, and every change reaches storage without anyone saving.
    $effect.root(() => {
      $effect(() => {
        document.documentElement.dataset.theme = this.resolvedTheme;
      });
      $effect(() => {
        persist(this.#snapshot());
      });
    });

    // Until a folder has been chosen once, the dialog opens on the system's
    // pictures folder.
    if (!this.directory) {
      defaultOutputDir().then(
        (directory) => (this.directory ||= directory),
        () => {},
      );
    }
  }

  /** The subset the Rust side needs to write files. */
  get output(): OutputSettings {
    return {
      directory: this.directory,
      format: this.format,
      quality: this.quality,
      overwrite: this.overwrite,
    };
  }

  /**
   * The theme actually painted. `tokens.css` carries no
   * `prefers-color-scheme` query — dark is `:root[data-theme="dark"]` — so
   * `auto` is resolved here rather than in CSS.
   */
  get resolvedTheme(): ResolvedTheme {
    if (this.theme !== "auto") return this.theme;
    return systemDark.current ? "dark" : "light";
  }

  /** Quality only means something for the lossy and compressed formats. */
  get qualityApplies(): boolean {
    return this.format !== "tiff";
  }

  /**
   * Storage can hold anything — a value an older version wrote, or one
   * edited by hand — so each field is checked, and one that does not fit
   * keeps its default rather than reaching the Rust side.
   */
  #restore() {
    const stored = read();
    if (typeof stored.directory === "string") this.directory = stored.directory;
    this.format = oneOf(
      stored.format,
      OUTPUT_FORMATS.map((f) => f.value),
      this.format,
    );
    this.quality = within(stored.quality, QUALITY_RANGE, this.quality);
    if (typeof stored.overwrite === "boolean") this.overwrite = stored.overwrite;
    this.theme = oneOf(stored.theme, THEMES, this.theme);
    this.viewMode = oneOf(stored.viewMode, VIEW_MODES, this.viewMode);
    this.cardSize = within(stored.cardSize, CARD_SIZE_RANGE, this.cardSize);
  }

  #snapshot(): Persisted {
    return {
      directory: this.directory,
      format: this.format,
      quality: this.quality,
      overwrite: this.overwrite,
      theme: this.theme,
      viewMode: this.viewMode,
      cardSize: this.cardSize,
    };
  }
}

function read(): Partial<Record<keyof Persisted, unknown>> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    return typeof parsed === "object" && parsed !== null ? parsed : {};
  } catch {
    // Corrupt or unreadable storage just means defaults.
    return {};
  }
}

function persist(snapshot: Persisted) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  } catch {
    // Not being able to persist is not worth interrupting the user over.
  }
}

function oneOf<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}

function within(value: unknown, range: { min: number; max: number }, fallback: number): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return fallback;
  return Math.min(range.max, Math.max(range.min, value));
}

export const settings = new Settings();
