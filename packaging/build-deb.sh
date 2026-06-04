#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKG_NAME="md-reader"

cd "${ROOT}"
cargo build --release

VERSION="$("${ROOT}/target/release/md-reader" --version | awk '{print $2}')"
ARCH="$(dpkg --print-architecture)"
BUILD_DIR="${ROOT}/target/debian/${PKG_NAME}_${VERSION}_${ARCH}"
DEB_PATH="${ROOT}/target/debian/${PKG_NAME}_${VERSION}_${ARCH}.deb"

rm -rf "${BUILD_DIR}"
mkdir -p \
  "${BUILD_DIR}/DEBIAN" \
  "${BUILD_DIR}/usr/bin" \
  "${BUILD_DIR}/usr/share/applications" \
  "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps" \
  "${BUILD_DIR}/usr/share/doc/${PKG_NAME}" \
  "${BUILD_DIR}/usr/share/md-reader/models"

install -m 0755 "${ROOT}/target/release/md-reader" "${BUILD_DIR}/usr/bin/md-reader"
install -m 0644 "${ROOT}/packaging/md-reader.desktop" "${BUILD_DIR}/usr/share/applications/md-reader.desktop"
install -m 0644 "${ROOT}/packaging/md-reader.svg" "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/md-reader.svg"
install -m 0644 "${ROOT}/README.md" "${BUILD_DIR}/usr/share/doc/${PKG_NAME}/README.md"
install -m 0644 "${ROOT}/models/en_US-libritts_r-medium.onnx.json" "${BUILD_DIR}/usr/share/md-reader/models/en_US-libritts_r-medium.onnx.json"

if [[ -f "${ROOT}/models/en_US-libritts_r-medium.onnx" ]]; then
  install -m 0644 "${ROOT}/models/en_US-libritts_r-medium.onnx" "${BUILD_DIR}/usr/share/md-reader/models/en_US-libritts_r-medium.onnx"
fi

INSTALLED_SIZE="$(du -sk "${BUILD_DIR}" | awk '{print $1}')"
cat > "${BUILD_DIR}/DEBIAN/control" <<CONTROL
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: md-reader maintainers
Depends: libgtk-3-0, libasound2 | libasound2t64, libssl3, libxkbcommon0, libxcb-render0, libxcb-shape0, libxcb-xfixes0
Installed-Size: ${INSTALLED_SIZE}
Description: Markdown reader desktop app
 md-reader is a Rust and egui desktop app for reading Markdown files.
 It supports file/folder browsing, live reload, search, table of contents,
 themes, zoom, and Piper-based text to speech when model files are installed.
CONTROL

dpkg-deb --build --root-owner-group "${BUILD_DIR}" "${DEB_PATH}"
echo "${DEB_PATH}"
