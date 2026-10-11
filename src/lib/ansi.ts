// Minimaler ANSI-Parser fuer die Skriptausgaben (Farben 30-37/90-97, fett).
// Alles andere (Cursorsteuerung usw.) wird entfernt.

export interface Segment {
  text: string;
  color?: string;
  bold?: boolean;
}

const COLORS: Record<number, string> = {
  30: "black", 31: "red", 32: "green", 33: "yellow",
  34: "blue", 35: "magenta", 36: "cyan", 37: "white",
};

const SEQ = /\x1b\[([0-9;]*)([A-Za-z])/g;

export function parseAnsi(input: string): Segment[] {
  const out: Segment[] = [];
  let color: string | undefined;
  let bold = false;
  let last = 0;
  const text = input.replace(/\r(?!\n)/g, "\n");

  for (const m of text.matchAll(SEQ)) {
    if (m.index! > last) out.push({ text: text.slice(last, m.index), color, bold });
    last = m.index! + m[0].length;
    if (m[2] !== "m") continue;
    const codes = m[1] === "" ? [0] : m[1].split(";").map(Number);
    for (const c of codes) {
      if (c === 0) { color = undefined; bold = false; }
      else if (c === 1) bold = true;
      else if (c === 22) bold = false;
      else if (c === 39) color = undefined;
      else if (COLORS[c]) color = COLORS[c];
      else if (c >= 90 && c <= 97) color = COLORS[c - 60];
    }
  }
  if (last < text.length) out.push({ text: text.slice(last), color, bold });
  // Restliche Steuerzeichen (ausser Zeilenumbruch/Tab) entfernen
    return out.map((s) => ({ ...s, text: s.text.replace(/[\x00-\x08\x0b-\x1f\x7f]/g, "") }));
}
