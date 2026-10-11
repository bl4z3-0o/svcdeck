import { describe, expect, it } from "vitest";
import { parseAnsi } from "./ansi";

describe("parseAnsi", () => {
  it("returns plain text unchanged", () => {
    expect(parseAnsi("hello\nworld")).toEqual([{ text: "hello\nworld", color: undefined, bold: false }]);
  });

  it("maps colors and bold, resets on 0", () => {
    const segs = parseAnsi("\x1b[32m  OK\x1b[0m done \x1b[1;31mFAIL\x1b[0m");
    expect(segs).toEqual([
      { text: "  OK", color: "green", bold: false },
      { text: " done ", color: undefined, bold: false },
      { text: "FAIL", color: "red", bold: true },
    ]);
  });

  it("supports bright colors (90-97)", () => {
    expect(parseAnsi("\x1b[93mwarn")[0]).toMatchObject({ text: "warn", color: "yellow" });
  });

  it("drops non-color escape sequences and control characters", () => {
    const text = parseAnsi("\x1b[2Kline\x07\x1b[1A").map((s) => s.text).join("");
    expect(text).toBe("line");
  });

  it("turns lone carriage returns into newlines", () => {
    // Progress bars overwrite lines with \r; CRLF endings become plain \n
    expect(parseAnsi("a\rb\r\n").map((s) => s.text).join("")).toBe("a\nb\n");
  });
});
