#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL_DIR="${ROOT}/models"
MODEL_NAME="en_US-libritts_r-medium.onnx"
CONFIG_NAME="${MODEL_NAME}.json"
MODEL_URL="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/${MODEL_NAME}"
CONFIG_URL="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/${CONFIG_NAME}"

mkdir -p "${MODEL_DIR}"

fetch_if_missing() {
  local url="$1"
  local path="$2"

  if [[ -f "${path}" ]]; then
    return
  fi

  echo "Downloading $(basename "${path}")"
  curl --fail --location --show-error --progress-bar "${url}" --output "${path}"
}

fetch_if_missing "${MODEL_URL}" "${MODEL_DIR}/${MODEL_NAME}"
fetch_if_missing "${CONFIG_URL}" "${MODEL_DIR}/${CONFIG_NAME}"
