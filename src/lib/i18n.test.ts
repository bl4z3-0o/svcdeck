import { describe, expect, it } from "vitest";
import { t } from "./i18n.svelte";

describe("t", () => {
  it("interpolates parameters", () => {
    expect(t("sheet.exit", { code: 3 })).toMatch(/3/);
  });

  it("leaves unknown placeholders visible instead of dropping them", () => {
    expect(t("sheet.exit")).toMatch(/\{code\}/);
  });
});
