import { describe, expect, it } from "vitest";
import { buildApp, needsTarget, shq, TEMPLATE_KINDS, type TemplateKind } from "./templates";

const base = { id: "x", name: "X", target: "", dir: "" };

describe("shq", () => {
  it("wraps in single quotes", () => {
    expect(shq("foo.service")).toBe("'foo.service'");
  });

  it("escapes embedded single quotes", () => {
    expect(shq("it's")).toBe(`'it'\\''s'`);
  });

  it("neutralizes shell metacharacters", () => {
    // Everything stays inside one quoted word
    expect(shq("a; rm -rf ~ $(id) `id`")).toBe("'a; rm -rf ~ $(id) `id`'");
  });
});

describe("buildApp", () => {
  it("systemd user service: quoted unit, user scope, status command", () => {
    const app = buildApp({ ...base, kind: "systemd-user", target: "syncthing.service" });
    expect(app.healthCommand).toBe("systemctl --user is-active --quiet 'syncthing.service'");
    const byId = Object.fromEntries(app.actions.map((a) => [a.id, a]));
    expect(byId.start.command).toBe("systemctl --user start 'syncthing.service'");
    expect(byId.stop.confirm).toBe(true);
    expect(byId.logs.mode).toBe("terminal");
    expect(app.actions.map((a) => a.id)).toEqual(["start", "stop", "restart", "status", "logs"]);
  });

  it("systemd system service uses sudo only where needed", () => {
    const app = buildApp({ ...base, kind: "systemd-system", target: "nginx.service" });
    const byId = Object.fromEntries(app.actions.map((a) => [a.id, a]));
    expect(byId.start.command).toMatch(/^sudo systemctl start /);
    expect(byId.status.command).not.toMatch(/sudo/);
    expect(app.healthCommand).not.toMatch(/sudo/);
  });

  it("docker container: unsafe characters stay quoted or are stripped", () => {
    const target = "web'; reboot #";
    const app = buildApp({ ...base, kind: "docker", target });
    const byId = Object.fromEntries(app.actions.map((a) => [a.id, a]));
    // Passed as a single quoted argument
    expect(byId.start.command).toBe(`docker start ${shq(target)}`);
    expect(byId.logs.command).toBe(`docker logs -f --tail 100 ${shq(target)}`);
    expect(app.healthCommand).toContain(shq(target));
    // The name filter is a regex, not quoted: only safe characters survive
    expect(byId.status.command).toBe("docker ps -a --filter name=^/webreboot$");
  });

  it("compose project runs in the given folder", () => {
    const app = buildApp({ ...base, kind: "compose", dir: "/srv/app" });
    expect(app.dir).toBe("/srv/app");
    expect(app.actions.find((a) => a.id === "start")?.command).toBe("docker compose up -d");
  });

  it("custom template has empty start/stop/status", () => {
    const app = buildApp({ ...base, kind: "custom" });
    expect(app.actions.map((a) => [a.id, a.command])).toEqual([
      ["start", ""],
      ["stop", ""],
      ["status", ""],
    ]);
  });

  it("every template produces unique action IDs and no URL", () => {
    for (const { kind } of TEMPLATE_KINDS) {
      const app = buildApp({ ...base, kind, target: "t", dir: "/d" });
      const ids = app.actions.map((a) => a.id);
      expect(new Set(ids).size).toBe(ids.length);
      expect(app.url).toBe("");
    }
  });
});

describe("needsTarget", () => {
  it.each<[TemplateKind, string | null]>([
    ["systemd-user", "unit"],
    ["systemd-system", "unit"],
    ["docker", "container"],
    ["compose", null],
    ["custom", null],
  ])("%s -> %s", (kind, expected) => {
    expect(needsTarget(kind)).toBe(expected);
  });
});
