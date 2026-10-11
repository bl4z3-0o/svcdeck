# SvcDeck

**Dienste starten, stoppen und öffnen, die du sonst im Terminal steuerst.**

[English](README.md)

![SvcDeck](docs/screenshot.png)

SvcDeck ist ein kleines schwebendes Fenster mit einer Kachel pro App oder Dienst: eine systemd-Unit, ein Docker-Container, ein Compose-Projekt oder ein beliebiges Shell-Skript. Linksklick öffnet die App im Browser, Rechtsklick zeigt ihre Aktionen (Start, Stop, Status, Logs oder was du selbst festlegst).

- **Jeder Befehl.** Jede Aktion ist ein bash-Befehl, der im Ordner der App läuft. Vorlagen für systemd (Benutzer und System), Docker und Docker Compose füllen Start, Stop, Neustart, Status und Logs automatisch aus.
- **Status auf einen Blick.** Ein Punkt zeigt, ob ein Dienst läuft. Geprüft wird per Befehl (Rückgabewert 0) oder per TCP-Port.
- **sudo ohne Terminal.** Befehle, die Root brauchen, fragen das Passwort in einem grafischen Dialog ab (rofi, zenity, kdialog oder ssh-askpass).
- **Passt zum Desktop.** Übernimmt unter Hyprland live die HyDE/wallbash-Farben, sonst die Hell/Dunkel-Einstellung des Systems. Englisch und Deutsch.
- **Klein.** Tauri 2 (Rust) mit Svelte-5-Oberfläche, ein einziges Programm von wenigen MB mit dem WebKitGTK des Systems.

## Installation

SvcDeck richtet sich derzeit an Arch Linux und Ableger (CachyOS, EndeavourOS, Manjaro). Andere Distributionen funktionieren, wenn die [Tauri-Voraussetzungen](https://tauri.app/start/prerequisites/) installiert sind. Ein AUR-Paket ist geplant.

```bash
sudo pacman -S --needed base-devel webkit2gtk-4.1 gtk3 librsvg openssl nodejs npm rustup
rustup default stable

git clone https://github.com/bl4z3-0o/svcdeck.git
cd svcdeck
./install.sh
```

`install.sh` installiert nur für den aktuellen Benutzer (ohne Root): `~/.local/bin/svcdeck`, ein Icon und einen Starter-Eintrag. Danach **SvcDeck** im App-Starter öffnen.

| Option | Wirkung |
|---|---|
| `--skip-build` | vorhandenen Release-Build installieren |
| `--yes` | vor Änderungen an der Compositor-Konfiguration nicht nachfragen |
| `--uninstall` | SvcDeck entfernen (deine Konfiguration bleibt) |

### Kachelnde Compositoren

Auf kachelnden Compositoren soll das Fenster schweben. Das erledigt `install.sh`:

- **Hyprland (Lua-Konfiguration, ab 0.55):** schreibt `~/.config/hypr/svcdeck.lua` und ergänzt nach Rückfrage `require("svcdeck")` in `hyprland.lua`.
- **Hyprland (`hyprland.conf`), Sway, i3, niri:** zeigt die Regel an, die du eintragen musst.
- **GNOME, KDE Plasma und andere schwebende Desktops:** nichts zu tun.

Die Fensterklasse ist `svcdeck`. Terminals, die SvcDeck öffnet, bekommen `svcdeck-term`, sofern das Terminal das unterstützt.

## Konfiguration

Alles lässt sich in der App bearbeiten (Zahnrad). Die Datei liegt unter `~/.config/svcdeck/apps.json`. Ein Beispiel und alle Felder stehen in der [englischen README](README.md#configuration).

Kurz zusammengefasst:

- `url`: wird per Linksklick geöffnet; leer = Linksklick führt die erste Aktion aus
- `icon`: Bildpfad oder Icon-Name aus deinem Icon-Theme; leer = `glyph` oder Standard-Icon
- `dir`: Arbeitsverzeichnis aller Aktionen (Standard: Home)
- `healthCommand`: Statusprüfung, Rückgabewert 0 = läuft; hat Vorrang vor `health` (`host:port`)
- `actions[].mode`: `capture` (Ausgabe in SvcDeck) oder `terminal` (eigenes Terminalfenster)
- `terminal`: leer = automatisch erkannt (`$TERMINAL`, kitty, foot, alacritty, wezterm, ghostty, konsole, gnome-terminal, …)

**Alle starten / Alle stoppen** (Rechtsklick auf den Hintergrund) führen in jeder App die Aktion mit der ID `start` bzw. `stop` aus; Stop läuft in umgekehrter Reihenfolge.

### Sicherheit

SvcDeck führt genau die Befehle aus deiner Konfiguration aus, mit deinen Benutzerrechten. Genau dafür ist es da, also behandle `apps.json` wie ein Shell-Skript: Nutze nur Konfigurationen, die du verstehst, und übernimm sie nicht aus unbekannten Quellen. Die Oberfläche schickt nie freie Befehle ans Backend; Aktionen werden per ID in der gespeicherten Datei nachgeschlagen. Statusprüfungen fragen nie nach einem Passwort.

## Entwicklung

```bash
npm install
PATH="$HOME/.cargo/bin:$PATH" npx tauri dev   # mit Live-Reload
npm run check                                  # svelte-check
cd src-tauri && cargo clippy                   # Rust-Lints
```

## Lizenz

[GPL-3.0-or-later](LICENSE). Du darfst SvcDeck nutzen, untersuchen, weitergeben und verändern. Weitergegebene und veränderte Versionen müssen unter der GPL bleiben, mit Quellcode.
