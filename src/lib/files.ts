import { open } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";

import { session } from "$state/session.svelte";
import { supportedExtensions } from "./api";

/**
 * Everything the app asks the desktop to do: the file dialogs and the two
 * ways of handing a path to the file manager. They live here for the reason
 * `invoke` lives in [api.ts](src/lib/api.ts) — one place per outside surface
 * — and because each of them fails the same way, by leaving a notice rather
 * than throwing at a component.
 */

/** The importer's own list, asked for the first time a dialog needs it. */
let extensions: Promise<string[]> | undefined;

/** Asks for scans and adds whatever was chosen to the session. */
export async function pickImages() {
  extensions ??= supportedExtensions();
  const chosen = await open({
    multiple: true,
    title: "Choose film scans",
    filters: [{ name: "Images", extensions: await extensions }],
  });
  await session.add(asPaths(chosen));
}

/** Asks for a folder and adds every supported scan under it. */
export async function pickFolder() {
  const chosen = await open({
    directory: true,
    multiple: false,
    title: "Choose a folder of scans",
  });
  await session.add(asPaths(chosen));
}

/** Asks where to write, seeded with the last answer. Null if cancelled. */
export async function pickOutputFolder(current: string): Promise<string | null> {
  const chosen = await open({
    directory: true,
    multiple: false,
    title: "Save developed images to",
    defaultPath: current || undefined,
  });
  return typeof chosen === "string" ? chosen : null;
}

/**
 * Opens the enclosing folder with the files picked out. A whole selection
 * goes in one call: the plugin shows them together rather than opening a
 * window each.
 */
export async function revealItems(paths: string | string[]) {
  if (paths.length === 0) return;
  try {
    await revealItemInDir(paths);
  } catch {
    session.notice = { kind: "error", text: "Could not open the file manager" };
  }
}

/** Opens a folder itself, rather than picking something out inside it. */
export async function openFolder(path: string) {
  try {
    await openPath(path);
  } catch {
    session.notice = {
      kind: "error",
      text: "Could not open the output folder",
    };
  }
}

/** A dialog answers with a string, a list, or null when it was cancelled. */
function asPaths(chosen: string | string[] | null): string[] {
  if (Array.isArray(chosen)) return chosen;
  return typeof chosen === "string" ? [chosen] : [];
}
