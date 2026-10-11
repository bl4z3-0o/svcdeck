// Templates for new apps: pick a type, enter a name, and start/stop/status/
// logs are generated. All user input is shell-quoted.
import { t } from "./i18n.svelte";
import type { Action, App } from "./types";

export type TemplateKind = "custom" | "systemd-user" | "systemd-system" | "docker" | "compose";

export const TEMPLATE_KINDS: { kind: TemplateKind; label: () => string }[] = [
  { kind: "custom", label: () => t("tpl.custom") },
  { kind: "systemd-user", label: () => t("tpl.systemdUser") },
  { kind: "systemd-system", label: () => t("tpl.systemdSystem") },
  { kind: "docker", label: () => t("tpl.docker") },
  { kind: "compose", label: () => t("tpl.compose") },
];

/** Single-quote for bash */
export function shq(s: string): string {
  return `'${s.replace(/'/g, `'\\''`)}'`;
}

function act(id: string, label: string, command: string, mode: Action["mode"] = "capture", confirm = false): Action {
  return { id, label, command, mode, confirm };
}

export interface TemplateInput {
  kind: TemplateKind;
  id: string;
  name: string;
  /** systemd unit or container name */
  target: string;
  /** project folder (compose) */
  dir: string;
}

export function buildApp(input: TemplateInput): App {
  const base: App = {
    id: input.id,
    name: input.name,
    url: "",
    icon: "",
    glyph: "",
    dir: input.dir,
    health: "",
    healthCommand: "",
    actions: [],
  };
  const q = shq(input.target.trim());

  switch (input.kind) {
    case "systemd-user":
      return {
        ...base,
        healthCommand: `systemctl --user is-active --quiet ${q}`,
        actions: [
          act("start", t("act.start"), `systemctl --user start ${q}`),
          act("stop", t("act.stop"), `systemctl --user stop ${q}`, "capture", true),
          act("restart", t("act.restart"), `systemctl --user restart ${q}`),
          act("status", t("act.status"), `systemctl --user status ${q} --no-pager -n 20`),
          act("logs", t("act.logs"), `journalctl --user -u ${q} -f -n 100`, "terminal"),
        ],
      };
    case "systemd-system":
      return {
        ...base,
        healthCommand: `systemctl is-active --quiet ${q}`,
        actions: [
          act("start", t("act.start"), `sudo systemctl start ${q}`),
          act("stop", t("act.stop"), `sudo systemctl stop ${q}`, "capture", true),
          act("restart", t("act.restart"), `sudo systemctl restart ${q}`),
          act("status", t("act.status"), `systemctl status ${q} --no-pager -n 20`),
          act("logs", t("act.logs"), `journalctl -u ${q} -f -n 100`, "terminal"),
        ],
      };
    case "docker":
      return {
        ...base,
        healthCommand: `[ "$(docker inspect -f '{{.State.Running}}' ${q} 2>/dev/null)" = true ]`,
        actions: [
          act("start", t("act.start"), `docker start ${q}`),
          act("stop", t("act.stop"), `docker stop ${q}`, "capture", true),
          act("restart", t("act.restart"), `docker restart ${q}`),
          act("status", t("act.status"), `docker ps -a --filter name=^/${input.target.trim().replace(/[^A-Za-z0-9_.-]/g, "")}$`),
          act("logs", t("act.logs"), `docker logs -f --tail 100 ${q}`, "terminal"),
        ],
      };
    case "compose":
      return {
        ...base,
        healthCommand: `[ -n "$(docker compose ps -q --status running)" ]`,
        actions: [
          act("start", t("act.start"), "docker compose up -d"),
          act("stop", t("act.stop"), "docker compose down", "capture", true),
          act("restart", t("act.restart"), "docker compose restart"),
          act("status", t("act.status"), "docker compose ps"),
          act("logs", t("act.logs"), "docker compose logs -f --tail 100", "terminal"),
        ],
      };
    default:
      return {
        ...base,
        actions: [
          act("start", t("act.start"), ""),
          act("stop", t("act.stop"), "", "capture", true),
          act("status", t("act.status"), ""),
        ],
      };
  }
}

/** Which extra input a template needs */
export function needsTarget(kind: TemplateKind): "unit" | "container" | null {
  if (kind === "systemd-user" || kind === "systemd-system") return "unit";
  if (kind === "docker") return "container";
  return null;
}
