/**
 * For an `$effect`: keeps a Tauri listener for exactly as long as the effect
 * lives.
 *
 * ```ts
 * $effect(() => listening(appWindow.onResized(sync)));
 * ```
 *
 * Tauri registers a listener asynchronously, so the effect can be torn down
 * before the registration lands. Then the listener is dropped the moment it
 * does, instead of outliving the component that asked for it.
 */
export function listening(registration: Promise<() => void>): () => void {
  let unlisten: (() => void) | undefined;
  let torn = false;

  registration.then((stop) => {
    if (torn) stop();
    else unlisten = stop;
  });

  return () => {
    torn = true;
    unlisten?.();
  };
}
