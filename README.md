# SvcDeck

**Start, stop and open the services you would otherwise control from a terminal.**

[Deutsch](README.de.md)

[![CI](https://github.com/bl4z3-0o/svcdeck/actions/workflows/ci.yml/badge.svg)](https://github.com/bl4z3-0o/svcdeck/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/bl4z3-0o/svcdeck)](https://github.com/bl4z3-0o/svcdeck/releases/latest)
[![AUR](https://img.shields.io/aur/version/svcdeck-bin?label=AUR)](https://aur.archlinux.org/packages/svcdeck-bin)

![SvcDeck](docs/screenshot.png)

SvcDeck is a small floating window with one tile per app or service: a systemd unit, a Docker container, a Compose project or any shell script. Left-click opens the app in your browser, right-click shows its actions (start, stop, status, logs, whatever you define).

- **Any command.** Every action is a bash command run in the app's folder. Templates for systemd (user and system), Docker and Docker Compose fill in start/stop/restart/status/logs for you.
- **Status at a glance.** A dot shows whether a service is running, checked by a command (exit code 0) or a TCP port.
- **sudo without a terminal.** Commands that need root ask for the password in a graphical dialog (rofi, zenity, kdialog or ssh-askpass).
- **Fits your desktop.** Follows your HyDE/wallbash colors live on Hyprland, otherwise the system light/dark preference. English and German.
- **Small.** Tauri 2 (Rust) with a Svelte 5 UI, a single binary of a few MB using the system WebKitGTK.

## Install

**Arch Linux and derivatives** (CachyOS, EndeavourOS, Manjaro) from the AUR:

```bash
yay -S svcdeck-bin    # prebuilt
yay -S svcdeck        # built from source
```

**Debian/Ubuntu and Fedora:** `.deb` and `.rpm` from the [latest release](https://github.com/bl4z3-0o/svcdeck/releases/latest).

**From source** (any distribution with the [Tauri prerequisites](https://tauri.app/start/prerequisites/)):

```bash
sudo pacman -S --needed base-devel webkit2gtk-4.1 gtk3 librsvg openssl nodejs npm rustup
rustup default stable

git clone https://github.com/bl4z3-0o/svcdeck.git
cd svcdeck
./install.sh
```

`install.sh` installs for the current user only (no root): `~/.local/bin/svcdeck`, an icon and a launcher entry. Then start **SvcDeck** from your app launcher.

### Updates

SvcDeck checks GitHub for a newer release on start and shows a hint. It never installs anything itself: update with your package manager (`yay -Syu`) or, for source installs, `git pull && ./install.sh`. The check sends a single request to `api.github.com` (GitHub sees your IP address) and can be turned off in Settings → General.

| Option | Effect |
|---|---|
| `--skip-build` | install the existing release build |
| `--yes` | do not ask before editing compositor config |
| `--uninstall` | remove SvcDeck (your configuration stays) |

### Tiling compositors

On tiling compositors the window should float. `install.sh` handles this:

- **Hyprland (Lua config, 0.55+):** writes `~/.config/hypr/svcdeck.lua` and, after asking, adds `require("svcdeck")` to `hyprland.lua`.
- **Hyprland (`hyprland.conf`), Sway, i3, niri:** prints the rule to add.
- **GNOME, KDE Plasma and other floating desktops:** nothing to do.

The window class is `svcdeck`, terminals opened by SvcDeck use `svcdeck-term` where the terminal supports it.

## Configuration

Everything can be edited in the app (gear icon). The file is `~/.config/svcdeck/apps.json`:

```json
{
  "title": "SvcDeck",
  "columns": 2,
  "terminal": [],
  "theme": "auto",
  "language": "auto",
  "apps": [
    {
      "id": "syncthing",
      "name": "Syncthing",
      "url": "http://127.0.0.1:8384",
      "icon": "syncthing",
      "glyph": "",
      "dir": "",
      "health": "",
      "healthCommand": "systemctl --user is-active --quiet syncthing.service",
      "actions": [
        { "id": "start", "label": "Start", "command": "systemctl --user start syncthing.service", "mode": "capture", "confirm": false },
        { "id": "stop", "label": "Stop", "command": "systemctl --user stop syncthing.service", "mode": "capture", "confirm": true },
        { "id": "logs", "label": "Follow logs", "command": "journalctl --user -u syncthing.service -f", "mode": "terminal", "confirm": false }
      ]
    }
  ]
}
```

| Field | Meaning |
|---|---|
| `url` | opened on left click; empty = left click runs the first action |
| `icon` | image path or icon name from your icon theme; empty = `glyph`, or the default icon |
| `dir` | working directory for all actions (default: home) |
| `healthCommand` | status check, exit code 0 = running; takes precedence over `health` |
| `health` | `host:port` for the status dot; empty = derived from `url` |
| `actions[].mode` | `capture` (output shown in SvcDeck) or `terminal` (own terminal window) |
| `actions[].confirm` | ask before running |
| `terminal` | terminal invocation, the command is appended as `bash -lc …`; empty = detected (`$TERMINAL`, kitty, foot, alacritty, wezterm, ghostty, konsole, gnome-terminal, …) |
| `theme` | `auto` (HyDE colors or system), `light`, `dark` |
| `language` | `auto` (system), `en`, `de` |

**Start all / Stop all** (right-click on the background) run the action with the ID `start` or `stop` of every app; stop runs in reverse order.

### Security

SvcDeck runs exactly the commands in your configuration, with your user rights. That is its purpose, so treat `apps.json` like a shell script: only use configurations you understand, and do not import them from untrusted sources. The UI never sends free-form commands to the backend; actions are looked up by ID in the saved file. Status checks never prompt for a password.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for checks and commit conventions. Dependencies are kept current by Renovate; releases are made with release-please.

```bash
npm install
PATH="$HOME/.cargo/bin:$PATH" npx tauri dev   # hot reload
npm run check                                  # svelte-check
cd src-tauri && cargo clippy                   # Rust lints
```

Project layout: `src/` Svelte UI, `src-tauri/src/` Rust backend (`config.rs` configuration, `actions.rs` running commands, `terminal.rs` terminal detection, `icons.rs` icon lookup, `theme.rs` HyDE colors).

## License

[GPL-3.0-or-later](LICENSE). You may use, study, share and modify SvcDeck. Distributed copies and modified versions must stay under the GPL, with source code.
