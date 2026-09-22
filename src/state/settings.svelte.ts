import type { OutputFormat, OutputSettings } from "$types";
import { defaultOutputDir } from "$lib/api";

const STORAGE_KEY = "film-converter.settings";

export type ViewMode = "grid" | "list";
/** `auto` follows the system; the other two override it. */
export type Theme = "auto" | "light" | "dark";
export type ResolvedTheme = Exclude<Theme, "auto">;

interface Persisted {
  directory: string;
  format: OutputFormat;
  quality: number;
  overwrite: boolean;
  theme: Theme;
  viewMode: ViewMode;
  cardSize: number;
}

function systemTheme(): ResolvedTheme {
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

/**
 * User preferences, persisted to local storage so a session picks up where
 * the last one left off.
 */
class Settings {
  directory = $state("");
  format = $state<OutputFormat>("jpg");
  /** Matches the Python's 50-100 slider. */
  quality = $state(95);
  overwrite = $state(false);

  theme = $state<Theme>("auto");
  /** Kept in state so `auto` repaints when the system flips mid-session. */
  #systemTheme = $state<ResolvedTheme>("dark");
  viewMode = $state<ViewMode>("grid");
  /** Minimum width of a grid card, in pixels. */
  cardSize = $state(240);

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
   * The theme actually painted. `app.css` carries no `prefers-color-scheme`
   * query — dark is `:root[data-theme="dark"]` — so `auto` is resolved here
   * rather than in CSS.
   */
  get resolvedTheme(): ResolvedTheme {
    return this.theme === "auto" ? this.#systemTheme : this.theme;
  }

  /** Quality only means something for the lossy and compressed formats. */
  get qualityApplies(): boolean {
    return this.format !== "tiff";
  }

  async load() {
    let stored: Partial<Persisted> = {};
    try {
      stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    } catch {
      // Corrupt or unreadable storage just means defaults.
    }

    this.format = stored.format ?? this.format;
    this.quality = stored.quality ?? this.quality;
    this.overwrite = stored.overwrite ?? this.overwrite;
    this.theme = stored.theme ?? this.theme;
    this.viewMode = stored.viewMode ?? this.viewMode;
    this.cardSize = stored.cardSize ?? this.cardSize;

    this.directory = stored.directory ?? (await defaultOutputDir());

    this.#systemTheme = systemTheme();
    this.#followSystemTheme();
    this.applyTheme();
  }

  /**
   * Repaints on a system theme change while the preference is `auto`. The
   * listener lives as long as the app does, so it is never torn down.
   */
  #followSystemTheme() {
    window
      .matchMedia?.("(prefers-color-scheme: dark)")
      .addEventListener("change", (event) => {
        this.#systemTheme = event.matches ? "dark" : "light";
        if (this.theme === "auto") this.applyTheme();
      });
  }

  save() {
    const snapshot: Persisted = {
      directory: this.directory,
      format: this.format,
      quality: this.quality,
      overwrite: this.overwrite,
      theme: this.theme,
      viewMode: this.viewMode,
      cardSize: this.cardSize,
    };
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
    } catch {
      // Not being able to persist is not worth interrupting the user over.
    }
  }

  applyTheme() {
    document.documentElement.dataset.theme = this.resolvedTheme;
  }

  /** Sets the preference and repaints; the caller still persists it. */
  setTheme(theme: Theme) {
    this.theme = theme;
    this.applyTheme();
  }
}

export const settings = new Settings();
