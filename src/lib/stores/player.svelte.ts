import { formatTime } from "$lib/utils/format-time";

class PlayerStore {
  playing = $state(false);
  currentTime = $state(0);
  duration = $state(0);
  volume = $state(100);
  muted = $state(false);
  speed = $state(1.0);
  fullscreen = $state(false);
  controlsVisible = $state(true);
  title = $state("");
  /** Monotonic counter incremented on every `mpv:file-loaded`. Use this
   * (not `title`) to detect "is this a new file?": re-opening the same file
   * leaves the title identical, and two files in different folders can share
   * one. */
  fileEpoch = $state(0);

  /** Bumped only by user volume/mute actions to flash the volume OSD. Driven
   * this way — not off `volume` itself — so mpv's own volume echoes on load
   * (and the saved level applied on open) don't pop the OSD. */
  volumeOsdTick = $state(0);
  pulseVolumeOsd() {
    this.volumeOsdTick++;
  }

  get progress(): number {
    return this.duration > 0 ? (this.currentTime / this.duration) * 100 : 0;
  }

  get formattedTime(): string {
    return `${formatTime(this.currentTime)} / ${formatTime(this.duration)}`;
  }
}

export const player = new PlayerStore();
