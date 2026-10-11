import { describe, expect, it } from "vitest";
import { checkForUpdate, isNewer, parseVersion } from "./update";

describe("versions", () => {
  it("parses with or without v and suffix", () => {
    expect(parseVersion("v1.2.3")).toEqual([1, 2, 3]);
    expect(parseVersion("0.10.0-rc.1")).toEqual([0, 10, 0]);
    expect(parseVersion("latest")).toBeNull();
  });

  it("compares numerically, not as text", () => {
    expect(isNewer("v0.10.0", "0.9.9")).toBe(true);
    expect(isNewer("v0.1.1", "0.1.0")).toBe(true);
    expect(isNewer("v0.1.0", "0.1.0")).toBe(false);
    expect(isNewer("v0.0.9", "0.1.0")).toBe(false);
    expect(isNewer("garbage", "0.1.0")).toBe(false);
  });
});

function fakeFetch(status: number, body: unknown): typeof fetch {
  return (async () => new Response(JSON.stringify(body), { status })) as unknown as typeof fetch;
}

describe("checkForUpdate", () => {
  it("reports a newer release", async () => {
    const u = await checkForUpdate("0.1.0", fakeFetch(200, { tag_name: "v0.2.0", html_url: "https://x/rel" }));
    expect(u).toEqual({ version: "0.2.0", url: "https://x/rel" });
  });

  it("ignores same, older, draft and pre-releases", async () => {
    expect(await checkForUpdate("0.2.0", fakeFetch(200, { tag_name: "v0.2.0" }))).toBeNull();
    expect(await checkForUpdate("0.2.0", fakeFetch(200, { tag_name: "v0.1.0" }))).toBeNull();
    expect(await checkForUpdate("0.1.0", fakeFetch(200, { tag_name: "v0.2.0", draft: true }))).toBeNull();
    expect(await checkForUpdate("0.1.0", fakeFetch(200, { tag_name: "v0.2.0", prerelease: true }))).toBeNull();
  });

  it("stays quiet on errors (no release yet, offline, rate limit)", async () => {
    expect(await checkForUpdate("0.1.0", fakeFetch(404, { message: "Not Found" }))).toBeNull();
    expect(await checkForUpdate("0.1.0", fakeFetch(403, { message: "rate limit" }))).toBeNull();
    const offline = (async () => {
      throw new TypeError("network down");
    }) as unknown as typeof fetch;
    expect(await checkForUpdate("0.1.0", offline)).toBeNull();
  });
});
