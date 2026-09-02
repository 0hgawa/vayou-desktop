import { describe, expect, it } from "vitest";
import { formatTime } from "./format-time";

describe("formatTime", () => {
  it("omits the hour field below one hour", () => {
    expect(formatTime(0)).toBe("0:00");
    expect(formatTime(9)).toBe("0:09");
    expect(formatTime(62)).toBe("1:02");
    expect(formatTime(599)).toBe("9:59");
    expect(formatTime(3599)).toBe("59:59");
  });

  it("adds the hour field and zero-pads minutes from one hour up", () => {
    expect(formatTime(3600)).toBe("1:00:00");
    expect(formatTime(3661)).toBe("1:01:01");
    // Minutes are padded once hours are shown, so 1h2m does not read as "1:2".
    expect(formatTime(3720)).toBe("1:02:00");
    expect(formatTime(36000)).toBe("10:00:00");
  });

  it("truncates fractional seconds rather than rounding", () => {
    // The playhead ticks on fractions; rounding up would show a second the
    // video has not reached yet.
    expect(formatTime(1.9)).toBe("0:01");
    expect(formatTime(59.99)).toBe("0:59");
  });

  it("falls back to zero for missing or nonsensical input", () => {
    expect(formatTime(-1)).toBe("0:00");
    expect(formatTime(NaN)).toBe("0:00");
    expect(formatTime(undefined as unknown as number)).toBe("0:00");
  });
});
