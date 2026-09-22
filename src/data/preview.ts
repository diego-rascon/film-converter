/**
 * What the app asks for when it decodes a preview. These are the front
 * end's own tuning knobs rather than anything the Rust side defines, and
 * the session (which fills the grid) and the viewer (which asks again for a
 * sharper pair) both read them, so they belong to neither.
 */

/** Longest edge, in pixels, of the thumbnails shown in the grid and list. */
export const CARD_PREVIEW_EDGE = 720;

/** Longest edge of the larger preview the fullscreen viewer requests. */
export const FULL_PREVIEW_EDGE = 2000;

/**
 * How many previews to decode at once. Previews are full decodes of
 * potentially very large scans, so this keeps memory and CPU in check while
 * still using more than one core.
 */
export const PREVIEW_CONCURRENCY = 4;
