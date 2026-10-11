#!/usr/bin/env bash
#
# install.sh - build and install SvcDeck for the current user (no root needed)
#
#   1. checks build dependencies and prints the missing packages
#   2. release build (npm + cargo) unless --skip-build is given
#   3. binary to ~/.local/bin/svcdeck, icon + .desktop entry (app launcher)
#   4. window rules so tiling compositors float the window:
#        - Hyprland (Lua config): writes ~/.config/hypr/svcdeck.lua and, after
#          asking, adds require("svcdeck") to hyprland.lua
#        - Hyprland (hyprland.conf), Sway, i3, niri: prints the rule to add
#        - GNOME, KDE and other floating desktops need nothing
#   5. removes an old "Local AI Hub" installation (its config is migrated by
#      SvcDeck itself on first start and left in place)
#
# Usage:
#   ./install.sh                # build and install
#   ./install.sh --skip-build   # install the existing release binary
#   ./install.sh --yes          # do not ask before editing compositor config
#   ./install.sh --uninstall    # remove everything (configuration stays)
#
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
APP="svcdeck"
BIN_SRC="${SCRIPT_DIR}/src-tauri/target/release/${APP}"
BIN_DIR="${HOME}/.local/bin"
DATA="${XDG_DATA_HOME:-${HOME}/.local/share}"
CONF="${XDG_CONFIG_HOME:-${HOME}/.config}"
DESKTOP="${DATA}/applications/${APP}.desktop"
ICON_SVG="${DATA}/icons/hicolor/scalable/apps/${APP}.svg"
ICON_PNG="${DATA}/icons/hicolor/128x128/apps/${APP}.png"
HYPR_DIR="${CONF}/hypr"
HYPR_MAIN="${HYPR_DIR}/hyprland.lua"
HYPR_RULES="${HYPR_DIR}/${APP}.lua"
REQUIRE_LINE="require(\"${APP}\")"
REQUIRE_COMMENT="-- SvcDeck: floating window (see ${APP}.lua)"

ASSUME_YES=0
MODE="install"
SKIP_BUILD=0

c_reset=$'\033[0m'; c_bold=$'\033[1m'; c_green=$'\033[32m'; c_yellow=$'\033[33m'; c_red=$'\033[31m'; c_blue=$'\033[34m'
info() { printf '%s==>%s %s\n' "${c_blue}${c_bold}" "${c_reset}" "$*"; }
ok()   { printf '%s  OK%s %s\n' "${c_green}" "${c_reset}" "$*"; }
warn() { printf '%s  !!%s %s\n' "${c_yellow}" "${c_reset}" "$*"; }
die()  { printf '%sERROR%s %s\n' "${c_red}${c_bold}" "${c_reset}" "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

ask() {
  # ask "question" -> 0 = yes
  [[ ${ASSUME_YES} -eq 1 ]] && return 0
  [[ -t 0 ]] || return 1
  local reply
  read -r -p "  ?? $1 [y/N] " reply
  [[ "${reply,,}" == y || "${reply,,}" == yes || "${reply,,}" == j || "${reply,,}" == ja ]]
}

# --- old "Local AI Hub" installation ----------------------------------------

remove_legacy() {
  local found=0 f
  for f in "${BIN_DIR}/local-ai-hub" "${DATA}/applications/local-ai-hub.desktop" \
           "${DATA}/icons/hicolor/scalable/apps/local-ai-hub.svg" \
           "${DATA}/icons/hicolor/128x128/apps/local-ai-hub.png" "${HYPR_DIR}/local-ai-hub.lua"; do
    if [[ -e "${f}" ]]; then
      [[ ${found} -eq 0 ]] && info "Removing old Local AI Hub installation"
      found=1
      rm -f "${f}"
      ok "removed ${f}"
    fi
  done
  if [[ -f "${HYPR_MAIN}" ]] && grep -qxF 'require("local-ai-hub")' "${HYPR_MAIN}"; then
    [[ ${found} -eq 0 ]] && info "Removing old Local AI Hub installation"
    found=1
    sed -i -e '/^-- Local AI Hub: schwebendes Fenster/d' -e '/^require("local-ai-hub")$/d' "${HYPR_MAIN}"
    ok "removed require(\"local-ai-hub\") from ${HYPR_MAIN}"
  fi
  if [[ ${found} -eq 1 ]]; then
    pkill -x local-ai-hub 2>/dev/null || true
    [[ -d "${CONF}/local-ai-hub" ]] && ok "kept ${CONF}/local-ai-hub (SvcDeck copies it on first start)"
  fi
}

# --- dependencies -------------------------------------------------------------

check_deps() {
  info "Checking build dependencies"
  export PATH="${HOME}/.cargo/bin:${PATH}"
  local missing=() pkgs=()
  have cargo || { missing+=("Rust (cargo)"); pkgs+=(rustup); }
  have npm   || { missing+=("Node.js + npm"); pkgs+=(nodejs npm); }
  have pkg-config || { missing+=("pkg-config"); pkgs+=(base-devel); }
  if have pkg-config; then
    pkg-config --exists webkit2gtk-4.1 || { missing+=("WebKitGTK 4.1"); pkgs+=(webkit2gtk-4.1); }
    pkg-config --exists gtk+-3.0       || { missing+=("GTK 3"); pkgs+=(gtk3); }
    pkg-config --exists librsvg-2.0    || { missing+=("librsvg"); pkgs+=(librsvg); }
    pkg-config --exists openssl        || { missing+=("OpenSSL"); pkgs+=(openssl); }
  fi
  if [[ ${#missing[@]} -gt 0 ]]; then
    warn "missing: ${missing[*]}"
    if have pacman; then
      echo "     Arch: sudo pacman -S --needed ${pkgs[*]}"
      [[ " ${pkgs[*]} " == *" rustup "* ]] && echo "           rustup default stable"
    else
      echo "     See https://tauri.app/start/prerequisites/ for your distribution."
    fi
    die "install the missing packages and run ./install.sh again"
  fi
  ok "cargo $(cargo --version | cut -d' ' -f2), node $(node --version), WebKitGTK $(pkg-config --modversion webkit2gtk-4.1)"
}

build() {
  info "Building release (takes a few minutes the first time)"
  (cd "${SCRIPT_DIR}" && npm ci --no-audit --no-fund && npx tauri build --no-bundle)
  ok "built ${BIN_SRC}"
}

# --- window rules -------------------------------------------------------------

hyprland_lua_rules() {
  # Quoted heredoc: written literally (regex backslashes)
  cat > "${HYPR_RULES}" <<'EOF'
-- Generated by svcdeck/install.sh, rewritten on every install.

-- SvcDeck: floating and centered instead of tiled; it sizes itself.
hl.window_rule({
  name = "svcdeck",
  match = { class = "^(svcdeck)$" },
  float = true,
  center = true,
})

-- Terminal windows opened by SvcDeck (e.g. "Follow logs")
hl.window_rule({
  name = "svcdeck_term",
  match = { class = "^(svcdeck-term)$" },
  float = true,
  center = true,
  size = "(monitor_w*0.6) (monitor_h*0.6)",
})
EOF
  ok "wrote ${HYPR_RULES}"
  if grep -qxF "${REQUIRE_LINE}" "${HYPR_MAIN}"; then
    ok "already loaded from ${HYPR_MAIN}"
  elif ask "Add ${REQUIRE_LINE} to ${HYPR_MAIN}?"; then
    printf '\n%s\n%s\n' "${REQUIRE_COMMENT}" "${REQUIRE_LINE}" >> "${HYPR_MAIN}"
    ok "added ${REQUIRE_LINE} to ${HYPR_MAIN}"
  else
    warn "not changed. To float the window, add this line to ${HYPR_MAIN}:"
    echo "     ${REQUIRE_LINE}"
  fi
  have hyprctl && hyprctl reload >/dev/null 2>&1 || true
}

window_rules() {
  local desktop="${XDG_CURRENT_DESKTOP:-}"
  info "Window rules"
  if [[ -f "${HYPR_MAIN}" ]]; then
    hyprland_lua_rules
  elif [[ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" || -f "${HYPR_DIR}/hyprland.conf" ]]; then
    warn "Hyprland with hyprland.conf: add a float rule for class svcdeck, e.g. (Hyprland 0.48-0.52):"
    echo '     windowrule = float, class:^(svcdeck)$'
    echo '     windowrule = float, class:^(svcdeck-term)$'
    echo "     The rule syntax changed in newer versions, see https://wiki.hypr.land/Configuring/Window-Rules/"
  elif [[ -n "${SWAYSOCK:-}" || "${desktop,,}" == *sway* ]]; then
    warn "Sway: add to ~/.config/sway/config:"
    echo '     for_window [app_id="^svcdeck(-term)?$"] floating enable'
  elif [[ "${desktop,,}" == *niri* ]]; then
    warn "niri: add to ~/.config/niri/config.kdl:"
    echo '     window-rule { match app-id="^svcdeck(-term)?$"; open-floating true; }'
  elif [[ -n "${I3SOCK:-}" || "${desktop,,}" == *i3* ]]; then
    warn "i3: add to ~/.config/i3/config:"
    echo '     for_window [class="^svcdeck(-term)?$"] floating enable'
  else
    ok "not needed on this desktop (${desktop:-unknown})"
  fi
}

# --- install / uninstall --------------------------------------------------------

install_files() {
  [[ -x "${BIN_SRC}" ]] || die "binary missing: ${BIN_SRC} (run without --skip-build)"

  info "Installing binary, icon and launcher entry"
  install -Dm755 "${BIN_SRC}" "${BIN_DIR}/${APP}"
  install -Dm644 "${SCRIPT_DIR}/app-icon.svg" "${ICON_SVG}"
  install -Dm644 "${SCRIPT_DIR}/src-tauri/icons/128x128.png" "${ICON_PNG}"
  mkdir -p "$(dirname "${DESKTOP}")"
  cat > "${DESKTOP}" <<EOF
[Desktop Entry]
Type=Application
Name=SvcDeck
GenericName=Service launcher
GenericName[de]=Dienste-Starter
Comment=Start, stop and open the services you would otherwise control from a terminal
Comment[de]=Dienste starten, stoppen und öffnen, die du sonst im Terminal steuerst
Exec=${BIN_DIR}/${APP}
Icon=${APP}
Terminal=false
Categories=Utility;
Keywords=service;systemd;docker;compose;start;stop;launcher;dienst;
StartupWMClass=${APP}
EOF
  have update-desktop-database && update-desktop-database "${DATA}/applications" || true
  have gtk-update-icon-cache && gtk-update-icon-cache -q -t "${DATA}/icons/hicolor" || true
  ok "${BIN_DIR}/${APP}"
  ok "${DESKTOP}"
  [[ ":${PATH}:" == *":${BIN_DIR}:"* ]] || warn "${BIN_DIR} is not in PATH (the launcher entry works anyway)"
}

uninstall() {
  info "Removing SvcDeck"
  pkill -x "${APP}" 2>/dev/null || true
  rm -f "${BIN_DIR}/${APP}" "${DESKTOP}" "${ICON_SVG}" "${ICON_PNG}" "${HYPR_RULES}"
  if [[ -f "${HYPR_MAIN}" ]] && grep -qxF "${REQUIRE_LINE}" "${HYPR_MAIN}"; then
    sed -i -e "\|^${REQUIRE_COMMENT}\$|d" -e '/^require("svcdeck")$/d' "${HYPR_MAIN}"
    ok "removed ${REQUIRE_LINE} from ${HYPR_MAIN}"
    have hyprctl && hyprctl reload >/dev/null 2>&1 || true
  fi
  have update-desktop-database && update-desktop-database "${DATA}/applications" || true
  ok "removed (configuration in ${CONF}/${APP} is kept)"
}

for arg in "$@"; do
  case "${arg}" in
    --skip-build) SKIP_BUILD=1 ;;
    --yes|-y)     ASSUME_YES=1 ;;
    --uninstall)  MODE="uninstall" ;;
    -h|--help)    sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) die "unknown option: ${arg} (see --help)" ;;
  esac
done

if [[ "${MODE}" == uninstall ]]; then
  uninstall
  exit 0
fi

if [[ ${SKIP_BUILD} -eq 0 ]]; then
  check_deps
  build
fi
remove_legacy
install_files
window_rules
echo
info "Done. Start SvcDeck from your app launcher or run: ${BIN_DIR}/${APP}"
