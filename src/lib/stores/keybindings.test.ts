import { beforeEach, describe, expect, it } from "vitest";
import { KeybindingsStore } from "./keybindings.svelte";

/** A KeyboardEvent stand-in: `resolve` only reads these four fields. */
const press = (
  key: string,
  mods: { ctrl?: boolean; shift?: boolean; alt?: boolean } = {},
) =>
  ({
    key,
    ctrlKey: mods.ctrl ?? false,
    shiftKey: mods.shift ?? false,
    altKey: mods.alt ?? false,
  }) as KeyboardEvent;

let kb: KeybindingsStore;
beforeEach(() => {
  kb = new KeybindingsStore();
});

describe("resolve", () => {
  it("matches the defaults when nothing is overridden", () => {
    expect(kb.resolve(press(" "))).toBe("togglePause");
    expect(kb.resolve(press("s"))).toBe("screenshot");
    expect(kb.resolve(press("ArrowRight"))).toBe("seekForward");
    expect(kb.resolve(press("ArrowRight", { shift: true }))).toBe("seekForwardLong");
    expect(kb.resolve(press("o", { ctrl: true }))).toBe("openFile");
  });

  it("returns undefined for a key nothing is bound to", () => {
    expect(kb.resolve(press("z"))).toBeUndefined();
    // A modifier the binding does not ask for must not match either.
    expect(kb.resolve(press("s", { ctrl: true }))).toBeUndefined();
  });

  it("ignores Shift on symbol keys, which already bake it in", () => {
    // On most layouts "+" IS Shift+"=", so the browser reports key="+" with
    // shiftKey=true. Folding Shift into the combo would make the "+" default
    // permanently unmatchable and speed-up would silently never fire.
    expect(kb.resolve(press("+", { shift: true }))).toBe("speedUp");
    // A numpad "+" arrives without Shift and must still match.
    expect(kb.resolve(press("+"))).toBe("speedUp");
    // Named keys and letters keep Shift as a distinguishing modifier.
    expect(kb.resolve(press("ArrowLeft", { shift: true }))).toBe("seekBackLong");
    expect(kb.resolve(press("ArrowLeft"))).toBe("seekBack");
  });

  it("honours an override and frees the key it left behind", () => {
    kb.setKey("togglePause", "k");
    expect(kb.resolve(press("k"))).toBe("togglePause");
    expect(kb.resolve(press(" "))).toBeUndefined();
  });

  it("does not resolve an action that a conflict disabled", () => {
    // Bindings spell the spacebar "Space"; only the raw event uses " ".
    kb.setKey("screenshot", "Space");
    // togglePause was disabled, so Space now belongs to screenshot alone.
    expect(kb.getKey("togglePause")).toBe("");
    expect(kb.resolve(press(" "))).toBe("screenshot");
  });
});

describe("setKey", () => {
  it("records an override and reports it through getKey", () => {
    expect(kb.getKey("screenshot")).toBe("s");
    kb.setKey("screenshot", "F5");
    expect(kb.getKey("screenshot")).toBe("F5");
    expect(kb.toJSON()).toEqual({ screenshot: "F5" });
  });

  it("drops the override when a key is bound back to its default", () => {
    kb.setKey("screenshot", "F5");
    kb.setKey("screenshot", "s");
    // Back to default means no stored override at all, not an override that
    // happens to equal the default.
    expect(kb.toJSON()).toEqual({});
    expect(kb.getKey("screenshot")).toBe("s");
  });

  it("disables the default owner when its key is taken", () => {
    kb.setKey("screenshot", "Space");
    expect(kb.toJSON()).toEqual({ screenshot: "Space", togglePause: "" });
  });

  it("removes a conflicting override rather than disabling it", () => {
    kb.setKey("mute", "g");
    kb.setKey("screenshot", "g");
    // "g" is nobody's default, so mute simply reverts to its own default.
    expect(kb.getKey("mute")).toBe("m");
    expect(kb.getKey("screenshot")).toBe("g");
  });

  it("leaves other bindings untouched", () => {
    kb.setKey("screenshot", "F5");
    expect(kb.getKey("togglePause")).toBe("Space");
    expect(kb.getKey("mute")).toBe("m");
  });
});

describe("resetAll / loadFrom", () => {
  it("round-trips overrides through loadFrom and toJSON", () => {
    kb.loadFrom({ screenshot: "F5", mute: "g" });
    expect(kb.toJSON()).toEqual({ screenshot: "F5", mute: "g" });
    expect(kb.resolve(press("F5"))).toBe("screenshot");
  });

  it("resetAll restores every default", () => {
    kb.loadFrom({ screenshot: "F5" });
    kb.resetAll();
    expect(kb.toJSON()).toEqual({});
    expect(kb.resolve(press("s"))).toBe("screenshot");
  });
});

describe("keyLabel", () => {
  it("prettifies arrow keys and keeps the modifier prefix", () => {
    expect(KeybindingsStore.keyLabel("ArrowRight")).toBe("→");
    expect(KeybindingsStore.keyLabel("Shift+ArrowRight")).toBe("Shift+→");
    expect(KeybindingsStore.keyLabel("ArrowUp")).toBe("↑");
    expect(KeybindingsStore.keyLabel("ArrowDown")).toBe("↓");
  });

  it("shows a dash for a disabled binding instead of nothing", () => {
    expect(KeybindingsStore.keyLabel("")).toBe("—");
  });
});
