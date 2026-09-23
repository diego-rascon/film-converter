import type { Attachment } from "svelte/attachments";

/** Who to tell about each element the shared observer watches. */
const listeners = new WeakMap<Element, (visible: boolean) => void>();
let observer: IntersectionObserver | undefined;

/**
 * An attachment that reports whether its element is on screen, as it
 * scrolls in and out and once more, as hidden, when it goes:
 *
 * ```svelte
 * <figure {@attach onVisibility((visible) => ...)}>
 * ```
 *
 * One observer serves every element that asks, however many tiles a roll
 * holds; it is created the first time one does.
 */
export function onVisibility(onchange: (visible: boolean) => void): Attachment {
  return (element) => {
    observer ??= new IntersectionObserver((entries) => {
      for (const entry of entries) listeners.get(entry.target)?.(entry.isIntersecting);
    });
    listeners.set(element, onchange);
    observer.observe(element);

    return () => {
      observer?.unobserve(element);
      listeners.delete(element);
      onchange(false);
    };
  };
}
