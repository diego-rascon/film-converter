/**
 * Runs keyed tasks a few at a time, each key once. Keys marked as wanted —
 * for previews, the tiles on screen — jump the queue; the rest run in the
 * order they were asked for.
 *
 * `run` reports its own failures: the queue only needs to know that a task
 * is over, so a rejection frees the slot the same way a result does.
 */
export class TaskQueue<Key> {
  readonly #limit: number;
  readonly #run: (key: Key) => Promise<void>;

  #waiting: Key[] = [];
  #running = 0;
  /** Every key asked for and not forgotten since, waiting or running. */
  #requested = new Set<Key>();
  #wanted = new Set<Key>();

  constructor(limit: number, run: (key: Key) => Promise<void>) {
    this.#limit = limit;
    this.#run = run;
  }

  /** Queues the keys not already asked for, and starts what there is room for. */
  request(keys: Iterable<Key>) {
    for (const key of keys) {
      if (this.#requested.has(key)) continue;
      this.#requested.add(key);
      this.#waiting.push(key);
    }
    this.#pump();
  }

  /** Marks a key as wanted sooner — or no longer — whether or not it is queued. */
  want(key: Key, wanted: boolean) {
    if (wanted) this.#wanted.add(key);
    else this.#wanted.delete(key);
  }

  /**
   * Drops keys still waiting and lets them be asked for again. A task
   * already running finishes; `run` is expected to notice that what it was
   * for has gone.
   */
  forget(keys: Iterable<Key>) {
    const gone = new Set(keys);
    this.#waiting = this.#waiting.filter((key) => !gone.has(key));
    for (const key of gone) {
      this.#requested.delete(key);
      this.#wanted.delete(key);
    }
  }

  clear() {
    this.#waiting = [];
    this.#requested.clear();
    this.#wanted.clear();
  }

  #pump() {
    while (this.#running < this.#limit && this.#waiting.length > 0) {
      const key = this.#next();
      const settle = () => {
        this.#running -= 1;
        this.#pump();
      };
      this.#running += 1;
      this.#run(key).then(settle, settle);
    }
  }

  /** The first waiting key that is wanted, or else simply the first. */
  #next(): Key {
    const wanted = this.#waiting.findIndex((key) => this.#wanted.has(key));
    return this.#waiting.splice(Math.max(wanted, 0), 1)[0];
  }
}
