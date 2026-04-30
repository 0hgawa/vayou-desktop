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

  get progress(): number {
    return this.duration > 0 ? (this.currentTime / this.duration) * 100 : 0;
  }

  get formattedTime(): string {
    return `${formatTime(this.currentTime)} / ${formatTime(this.duration)}`;
  }
}

export const player = new PlayerStore();
