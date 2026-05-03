import { cycleAbLoop, setAbLoopA, setAbLoopB, clearAbLoop } from "$lib/bindings/playback";

class AbLoopStore {
  enabled = $state(false);
  a = $state<number | null>(null);
  b = $state<number | null>(null);

  enable() {
    this.enabled = true;
  }

  /** Cycle A → B → clear. Backend snapshots time-pos directly (no UI latency)
   * and returns the new state, which becomes the source of truth. */
  async cycle() {
    try {
      const state = await cycleAbLoop();
      this.a = state.a;
      this.b = state.b;
      this.enabled = state.a !== null || state.b !== null;
    } catch {}
  }

  async setA(time: number) {
    this.enabled = true;
    this.a = time;
    await setAbLoopA(time).catch(() => {});
  }

  async setB(time: number) {
    if (this.a === null) return; // B requires A
    this.enabled = true;
    this.b = time;
    await setAbLoopB(time).catch(() => {});
  }

  async clear() {
    this.enabled = false;
    this.a = null;
    this.b = null;
    await clearAbLoop().catch(() => {});
  }

  reset() {
    // Local-only reset (e.g. on file change). mpv clears its own ab-loop on file load.
    this.enabled = false;
    this.a = null;
    this.b = null;
  }
}

export const abLoop = new AbLoopStore();
