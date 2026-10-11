// Gemeinsame Typen der Oberflaechen-Komponenten

export type MenuItem =
  | { kind: "header"; label: string; hint?: string; tone?: "ok" | "off" }
  | { kind: "separator" }
  | {
      kind: "item";
      label: string;
      icon?: string;
      danger?: boolean;
      disabled?: boolean;
      onSelect: () => void;
    };

export interface SheetEntry {
  label: string;
  state: "pending" | "running" | "ok" | "fail";
  output: string;
}
