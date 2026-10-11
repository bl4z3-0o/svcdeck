#!/usr/bin/env bash
#
# make-tarball.sh <version> - pack the release build for GitHub releases
#
# Expects a finished `tauri build` (src-tauri/target/release/svcdeck) and
# writes dist-release/svcdeck-<version>-x86_64-linux.tar.gz plus the deb/rpm
# bundles (if built) and SHA256SUMS. The tarball layout is what the
# svcdeck-bin AUR package installs from.
#
set -euo pipefail

VERSION="${1:?usage: make-tarball.sh <version>}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." &>/dev/null && pwd)"
OUT="${ROOT}/dist-release"
NAME="svcdeck-${VERSION}"
STAGE="$(mktemp -d)"
trap 'rm -rf "${STAGE}"' EXIT

BIN="${ROOT}/src-tauri/target/release/svcdeck"
[[ -x "${BIN}" ]] || { echo "missing ${BIN}, run tauri build first" >&2; exit 1; }

mkdir -p "${STAGE}/${NAME}" "${OUT}"
install -m755 "${BIN}" "${STAGE}/${NAME}/svcdeck"
install -m644 "${ROOT}/packaging/svcdeck.desktop" "${STAGE}/${NAME}/svcdeck.desktop"
install -m644 "${ROOT}/app-icon.svg" "${STAGE}/${NAME}/svcdeck.svg"
install -m644 "${ROOT}/src-tauri/icons/128x128.png" "${STAGE}/${NAME}/svcdeck.png"
install -m644 "${ROOT}/LICENSE" "${ROOT}/README.md" "${STAGE}/${NAME}/"

tar -C "${STAGE}" --owner=0 --group=0 --numeric-owner -czf "${OUT}/${NAME}-x86_64-linux.tar.gz" "${NAME}"

# deb/rpm from `tauri build --bundles deb,rpm`, renamed to a stable pattern
shopt -s nullglob
for f in "${ROOT}"/src-tauri/target/release/bundle/deb/*.deb; do cp "${f}" "${OUT}/${NAME}-amd64.deb"; done
for f in "${ROOT}"/src-tauri/target/release/bundle/rpm/*.rpm; do cp "${f}" "${OUT}/${NAME}-x86_64.rpm"; done

(cd "${OUT}" && rm -f SHA256SUMS && sha256sum -- *.tar.gz *.deb *.rpm > "${STAGE}/SHA256SUMS" && mv "${STAGE}/SHA256SUMS" SHA256SUMS)
ls -l "${OUT}"
