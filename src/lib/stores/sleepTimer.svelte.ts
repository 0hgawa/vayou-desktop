import { pause } from "$lib/bindings/playback";

class SleepTimerStore {
  remainingMs = $state<number | null>(null);
  activeMinutes = $state<number | null>(null);

  #interval: ReturnType<typeof setInterval> | null = null;

  get formatted(): string | null {
    if (this.remainingMs === null) return null;
    const m = Math.floor(this.remainingMs / 60_000);
    const s = Math.floor((this.remainingMs % 60_000) / 1_000);
    return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  }

  setTimer(minutes: number) {
    this.cancel();
    this.activeMinutes = minutes;
    this.remainingMs = minutes * 60_000;
    this.#interval = setInterval(() => {
      const cur = this.remainingMs;
      if (cur === null) return;
      if (cur <= 1_000) {
        this.cancel();
        pause().catch(() => {});
      } else {
        this.remainingMs = cur - 1_000;
      }
    }, 1_000);
  }

  cancel() {
    if (this.#interval) {
      clearInterval(this.#interval);
      this.#interval = null;
    }
    this.remainingMs = null;
    this.activeMinutes = null;
  }
}

export const sleepTimer = new SleepTimerStore();
