#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
EXAMPLES_DIR="$ROOT_DIR/examples"
GENERATED_DIR="$EXAMPLES_DIR/generated"
PROJECT_WEB="$GENERATED_DIR/web-smoke"
PROJECT_MOBILE="$GENERATED_DIR/mobile-smoke"

mkdir -p "$GENERATED_DIR"
rm -rf "$PROJECT_WEB" "$PROJECT_MOBILE"

echo "[examples] Building uto CLI..."
(cd "$ROOT_DIR" && cargo build -p uto-cli)

echo "[examples] Initializing web sample project..."
(cd "$ROOT_DIR" && cargo run -p uto-cli -- init "$PROJECT_WEB" --template web --uto-root "$ROOT_DIR")

echo "[examples] Running web sample project..."
(cd "$ROOT_DIR" && cargo run -p uto-cli -- run --project "$PROJECT_WEB" --target web --report-json "$PROJECT_WEB/.uto/reports/last-run.json")

echo "[examples] Reporting web sample project..."
(cd "$ROOT_DIR" && cargo run -p uto-cli -- report --project "$PROJECT_WEB")

if [ "${WITH_MOBILE:-0}" = "1" ]; then
  echo "[examples] Initializing mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- init "$PROJECT_MOBILE" --template mobile --uto-root "$ROOT_DIR")

  echo "[examples] Running mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- run --project "$PROJECT_MOBILE" --target mobile --report-json "$PROJECT_MOBILE/.uto/reports/last-run.json")

  echo "[examples] Reporting mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- report --project "$PROJECT_MOBILE")
fi

echo "[examples] CLI validation completed."
