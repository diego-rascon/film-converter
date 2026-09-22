/**
 * How the app words numbers and errors. These were spread across
 * `session.svelte.ts` and the components that needed them, which meant a
 * row importing `formatBytes` from the session store and two copies of the
 * same error-to-string fallback.
 */

export function plural(count: number, word: string): string {
  return count === 1 ? word : `${word}s`;
}

/** `3 images`, `1 image` — a count with its noun already agreed. */
export function counted(count: number, word: string): string {
  return `${count} ${plural(count, word)}`;
}

/** A byte count as a file manager would state it. */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unit]}`;
}

/**
 * Whatever a rejected `invoke` threw, as something worth showing. Tauri
 * rejects with the command's error string, but a failure on the way there
 * arrives as an `Error`.
 */
export function describeError(reason: unknown, fallback = "Something went wrong"): string {
  if (typeof reason === "string") return reason;
  if (reason instanceof Error) return reason.message;
  return fallback;
}
