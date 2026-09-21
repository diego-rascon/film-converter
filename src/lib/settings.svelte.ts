import type { OutputFormat, OutputSettings } from "./types";
import { defaultOutputDir } from "./api";

const STORAGE_KEY = "film-converter.settings";

export type ViewMode = "grid" | "list";
export type Theme = "light" | "dark";

interface Persisted {
  directory: string;
  format: OutputFormat;
  quality: number;
  overwrite: boolean;
  theme: Theme;
  viewMode: ViewMode;
  cardSize: number;
}

function systemTheme(): Theme {
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

  theme = $state<Theme>("dark");
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
    this.theme = stored.theme ?? systemTheme();
    this.viewMode = stored.viewMode ?? this.viewMode;
    this.cardSize = stored.cardSize ?? this.cardSize;

    this.directory = stored.directory ?? (await defaultOutputDir());
    this.applyTheme();
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
    document.documentElement.dataset.theme = this.theme;
  }

  toggleTheme() {
    this.theme = this.theme === "dark" ? "light" : "dark";
    this.applyTheme();
  }
}

export const settings = new Settings();
