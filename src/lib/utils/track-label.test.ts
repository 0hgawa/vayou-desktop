import { describe, expect, it } from "vitest";
import { trackFormat, trackLabel } from "./track-label";

describe("trackFormat", () => {
  it("tags subtitle and audio codecs with their familiar name", () => {
    expect(trackFormat("subrip")).toBe("SRT");
    expect(trackFormat("ass")).toBe("ASS");
    expect(trackFormat("hdmv_pgs_subtitle")).toBe("PGS");
    expect(trackFormat("dvd_subtitle")).toBe("VobSub");
    expect(trackFormat("eac3")).toBe("E-AC3");
    expect(trackFormat("truehd")).toBe("TrueHD");
  });

  it("folds the PCM sample-width variants into one tag", () => {
    expect(trackFormat("pcm_s16le")).toBe("PCM");
    expect(trackFormat("pcm_s24le")).toBe("PCM");
    expect(trackFormat("pcm_s32le")).toBe("PCM");
  });

  it("matches regardless of the case mpv reports", () => {
    expect(trackFormat("SubRip")).toBe("SRT");
    expect(trackFormat("AAC")).toBe("AAC");
  });

  it("returns empty for a codec with no tag, so no stray parentheses", () => {
    expect(trackFormat("h264")).toBe("");
    expect(trackFormat("")).toBe("");
  });
});

describe("trackLabel", () => {
  it("uses the language name alone when the title adds nothing", () => {
    expect(trackLabel("", "en-US", "subrip", 1)).toBe("English (SRT)");
    expect(trackLabel("", "ja", "aac", 6)).toBe("Japanese (AAC)");
  });

  it("appends a title that carries extra information", () => {
    expect(trackLabel("SDH", "eng", "ass", 2)).toBe("English · SDH (ASS)");
  });

  it("does not repeat a title that just restates the language", () => {
    expect(trackLabel("Korean", "kor", "subrip", 3)).toBe("Korean (SRT)");
    // The duplicate check is case-insensitive.
    expect(trackLabel("english", "eng", "subrip", 3)).toBe("English (SRT)");
  });

  it("falls back to the title when the track carries no language", () => {
    expect(trackLabel("Latin American", "", "subrip", 4)).toBe("Latin American (SRT)");
  });

  it("falls back to the track number when nothing else identifies it", () => {
    expect(trackLabel("", "", "", 5)).toBe("Track 5");
    // Still numbered, but the format is known.
    expect(trackLabel("", "", "flac", 7)).toBe("Track 7 (FLAC)");
  });
});
