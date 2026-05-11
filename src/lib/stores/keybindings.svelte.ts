import { invoke } from "@tauri-apps/api/core";

export interface KeyAction {
  id: string;
  i18nKey: string;
  category: string;
}

const ACTIONS: KeyAction[] = [
  // Playback
  { id: "togglePause", i18nKey: "playPause", category: "playback" },
  { id: "seekForward", i18nKey: "seekForward5", category: "playback" },
  { id: "seekForwardLong", i18nKey: "seekForward30", category: "playback" },
  { id: "seekBack", i18nKey: "seekBack5", category: "playback" },
  { id: "seekBackLong", i18nKey: "seekBack30", category: "playback" },
  { id: "nextFile", i18nKey: "next", category: "playback" },
  { id: "prevFile", i18nKey: "previous", category: "playback" },
  { id: "frameNext", i18nKey: "frameStep", category: "playback" },
  { id: "framePrev", i18nKey: "frameBack", category: "playback" },
  { id: "speedUp", i18nKey: "speedUp", category: "playback" },
  { id: "speedDown", i18nKey: "speedDown", category: "playback" },
  { id: "abLoop", i18nKey: "abLoop", category: "playback" },
  // Volume
  { id: "volumeUp", i18nKey: "volumeUp", category: "volume" },
  { id: "volumeDown", i18nKey: "volumeDown", category: "volume" },
  { id: "mute", i18nKey: "mute", category: "volume" },
  // Video
  { id: "fullscreen", i18nKey: "fullscreen", category: "video" },
  { id: "screenshot", i18nKey: "screenshot", category: "video" },
  { id: "aspectRatio", i18nKey: "aspectRatio", category: "video" },
  // Tracks
  { id: "cycleSub", i18nKey: "cycleSub", category: "tracks" },
  { id: "cycleAudio", i18nKey: "cycleAudio", category: "tracks" },
  // Navigation
  { id: "openFile", i18nKey: "openFile", category: "navigation" },
  { id: "openUrl", i18nKey: "openUrl", category: "navigation" },
  { id: "mediaInfo", i18nKey: "mediaInfo", category: "navigation" },
];

const DEFAULTS: Record<string, string> = {
  togglePause: "Space",
  seekForward: "ArrowRight",
  seekForwardLong: "Shift+ArrowRight",
  seekBack: "ArrowLeft",
  seekBackLong: "Shift+ArrowLeft",
  nextFile: "n",
  prevFile: "p",
  frameNext: ".",
  framePrev: ",",
  speedUp: "+",
  speedDown: "-",
  abLoop: "l",
  volumeUp: "ArrowUp",
  volumeDown: "ArrowDown",
  mute: "m",
  fullscreen: "f",
  screenshot: "s",
  aspectRatio: "r",
  cycleSub: "v",
  cycleAudio: "a",
  openFile: "Ctrl+o",
  openUrl: "Ctrl+u",
  mediaInfo: "i",
};

export class KeybindingsStore {
  custom = $state<Record<string, string>>({});
  #keyToAction = $derived.by(() => {
    const map = new Map<string, string>();
    for (const [action, defaultKey] of Object.entries(DEFAULTS)) {
      const key = this.custom[action] ?? defaultKey;
      map.set(key.toLowerCase(), action);
    }
    return map;
  });

  get actions() { return ACTIONS; }
  get defaults() { return DEFAULTS; }

  getKey(action: string): string {
    return this.custom[action] ?? DEFAULTS[action] ?? "";
  }

  resolve(e: KeyboardEvent): string | undefined {
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("ctrl");
    if (e.shiftKey) parts.push("shift");
    if (e.altKey) parts.push("alt");
    parts.push(e.key === " " ? "space" : e.key);
    return this.#keyToAction.get(parts.join("+").toLowerCase());
  }

  setKey(action: string, key: string) {
    // Remove any other action using this key
    for (const [a, k] of Object.entries(this.custom)) {
      if (k.toLowerCase() === key.toLowerCase() && a !== action) delete this.custom[a];
    }
    for (const [a, k] of Object.entries(DEFAULTS)) {
      if (k.toLowerCase() === key.toLowerCase() && a !== action) {
        this.custom[a] = ""; // disable conflicting default
      }
    }
    if (key.toLowerCase() === (DEFAULTS[action] ?? "").toLowerCase()) {
      delete this.custom[action]; // back to default, remove override
    } else {
      this.custom[action] = key;
    }
  }

  resetAll() {
    this.custom = {};
  }

  loadFrom(data: Record<string, string>) {
    this.custom = { ...data };
  }

  toJSON(): Record<string, string> {
    return { ...this.custom };
  }

  static keyLabel(key: string): string {
    if (!key) return "—";
    return key
      .replace("ArrowUp", "↑").replace("ArrowDown", "↓")
      .replace("ArrowLeft", "←").replace("ArrowRight", "→")
      .replace("Space", "Space").replace("Escape", "Esc")
      .replace("Shift+", "Shift+").replace("Ctrl+", "Ctrl+");
  }
}

export const keybindings = new KeybindingsStore();
