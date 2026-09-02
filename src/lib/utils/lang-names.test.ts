import { describe, expect, it } from "vitest";
import { langName } from "./lang-names";

describe("langName", () => {
  it("maps both the 2- and 3-letter codes for a language", () => {
    expect(langName("en")).toBe("English");
    expect(langName("eng")).toBe("English");
    // Some languages carry two 3-letter codes (ISO 639-2/T and /B).
    expect(langName("deu")).toBe("German");
    expect(langName("ger")).toBe("German");
    expect(langName("ces")).toBe("Czech");
    expect(langName("cze")).toBe("Czech");
  });

  it("ignores a region that does not change the name", () => {
    expect(langName("en-US")).toBe("English");
    expect(langName("de-AT")).toBe("German");
    expect(langName("fr-CA")).toBe("French");
  });

  it("keeps a region that does change the name", () => {
    expect(langName("pt-BR")).toBe("Portuguese (BR)");
    expect(langName("pt-PT")).toBe("Portuguese (PT)");
    expect(langName("zh-TW")).toBe("Chinese (Traditional)");
    expect(langName("zh-CN")).toBe("Chinese (Simplified)");
    expect(langName("es-MX")).toBe("Spanish (LA)");
  });

  it("normalises case and the underscore separator muxers emit", () => {
    expect(langName("PT_BR")).toBe("Portuguese (BR)");
    expect(langName("EN")).toBe("English");
  });

  it("falls back to the raw code so an unknown track stays identifiable", () => {
    expect(langName("xyz")).toBe("xyz");
    expect(langName("")).toBe("");
  });
});
