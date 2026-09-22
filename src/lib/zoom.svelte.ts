/**
 * The viewer stage's zoom and pan, as one object rather than five loose
 * pieces of component state. Pulling it out is what lets the viewer's
 * keyboard shortcuts, its zoom pod and its drag handling all work the same
 * control without passing four setters around.
 */
export class ZoomPan {
  /**
   * The pair the viewer shows is fetched at `FULL_PREVIEW_EDGE`, which runs
   * out of detail somewhere around 4x on a large stage; 6 leaves a little
   * headroom for pixel-peeping without pretending there is more.
   */
  static readonly MAX = 6;
  static readonly STEP = 1.4;

  /** A multiple of the *fitted* picture, so 1 is fit and there is nothing
      below it — the viewer never shows a scan smaller than the room it has. */
  zoom = $state(1);
  x = $state(0);
  y = $state(0);
  zoomed = $derived(this.zoom > 1);

  /** The box the picture is fitted into. The stage binds itself here. */
  stage = $state<HTMLElement | null>(null);
  /** The picture's aspect, so the pan limit is measured on the fitted box
      rather than on the stage. Null falls back to the stage's own. */
  aspect = $state<number | null>(null);

  /** Where a pan began, so the picture tracks the pointer exactly rather
      than accumulating a rounding error per move. */
  #from: { pointerX: number; pointerY: number; x: number; y: number } | null =
    null;

  reset() {
    this.zoom = 1;
    this.x = 0;
    this.y = 0;
  }

  /**
   * Pulls the offset back inside the room there is to pan. `object-fit:
   * contain` means the picture is the largest box of its aspect that fits,
   * so the overflow is measured on *that* — otherwise a letterboxed scan
   * pans into its own empty margins.
   */
  clamp() {
    const box = this.stage?.getBoundingClientRect();
    if (!box) return;

    const aspect = this.aspect ?? box.width / box.height;
    const width = Math.min(box.width, box.height * aspect);
    const height = Math.min(box.height, box.width / aspect);
    const limitX = Math.max(0, (width * this.zoom - box.width) / 2);
    const limitY = Math.max(0, (height * this.zoom - box.height) / 2);

    this.x = Math.min(limitX, Math.max(-limitX, this.x));
    this.y = Math.min(limitY, Math.max(-limitY, this.y));
  }

  /**
   * Scales by `factor` about (`atX`, `atY`), given from the stage's centre —
   * the origin the transform itself scales about. Keeping whatever sits
   * under that point where it is means the pan absorbs the difference,
   * which is what lets the wheel zoom into the corner it is pointing at.
   */
  by(factor: number, atX = 0, atY = 0) {
    const next = Math.min(ZoomPan.MAX, Math.max(1, this.zoom * factor));
    if (next === this.zoom) return;

    const ratio = next / this.zoom;
    this.x = atX - ratio * (atX - this.x);
    this.y = atY - ratio * (atY - this.y);
    this.zoom = next;
    this.clamp();
  }

  /** One wheel notch, zooming about the pointer. */
  wheel(deltaY: number, clientX: number, clientY: number) {
    const box = this.stage?.getBoundingClientRect();
    if (!box) return;
    this.by(
      deltaY < 0 ? ZoomPan.STEP : 1 / ZoomPan.STEP,
      clientX - box.left - box.width / 2,
      clientY - box.top - box.height / 2,
    );
  }

  startPan(clientX: number, clientY: number) {
    this.#from = { pointerX: clientX, pointerY: clientY, x: this.x, y: this.y };
  }

  panTo(clientX: number, clientY: number) {
    if (!this.#from) return;
    this.x = this.#from.x + (clientX - this.#from.pointerX);
    this.y = this.#from.y + (clientY - this.#from.pointerY);
    this.clamp();
  }

  endPan() {
    this.#from = null;
  }
}
