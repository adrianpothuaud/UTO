#!/usr/bin/env bash
set -euo pipefail

PORT="${1:-4173}"
ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$ROOT_DIR/uto-site/dist"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] cargo is required but was not found in PATH."
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "[ERROR] python3 is required to serve the static site locally."
  exit 1
fi

echo "[INFO] Building static site..."
(
  cd "$ROOT_DIR"
  cargo run -p uto-site
)

echo "[INFO] Static site generated at: $DIST_DIR"
if [[ ! -d "$DIST_DIR" ]]; then
  echo "[ERROR] Dist directory not found: $DIST_DIR"
  exit 1
fi

URL="http://127.0.0.1:$PORT"
echo "[INFO] Serving at $URL"

if command -v xdg-open >/dev/null 2>&1; then
  # Best effort only; do not fail if browser cannot be opened.
  xdg-open "$URL" >/dev/null 2>&1 || true
fi

cd "$DIST_DIR"
python3 -m http.server "$PORT"
