/** What a notice is about, which decides its glyph and whether it waits. */
export type NoticeKind = "info" | "error" | "success";

/** A message shown in the floating notice panel over the images. */
export interface Notice {
  kind: NoticeKind;
  text: string;
}

/**
 * The one message on screen. Anything can post one — the session, a run,
 * a desktop call that failed — and the newest replaces whatever was there,
 * so this is its own small singleton rather than a field of any of them.
 */
class Notices {
  /** Replaced wholesale and never edited, so it is not deeply reactive. */
  current = $state.raw<Notice | null>(null);

  show(kind: NoticeKind, text: string) {
    this.current = { kind, text };
  }

  /**
   * Takes the notice down — or, given one, takes it down only if it is
   * still the one showing, so a timer set for an old message cannot clear
   * a newer one.
   */
  clear(notice?: Notice) {
    if (notice === undefined || this.current === notice) this.current = null;
  }
}

export const notices = new Notices();
