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

echo "[examples] Generating HTML report for web sample project..."
(cd "$ROOT_DIR" && cargo run -p uto-cli -- report --project "$PROJECT_WEB" --html)

WEB_HTML="$PROJECT_WEB/.uto/reports/last-run.html"
if [ ! -f "$WEB_HTML" ]; then
  echo "[examples] ERROR: expected HTML report at $WEB_HTML"
  exit 1
fi

if [ "${WITH_MOBILE:-0}" = "1" ]; then
  echo "[examples] Initializing mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- init "$PROJECT_MOBILE" --template mobile --uto-root "$ROOT_DIR")

  echo "[examples] Running mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- run --project "$PROJECT_MOBILE" --target mobile --report-json "$PROJECT_MOBILE/.uto/reports/last-run.json")

  echo "[examples] Reporting mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- report --project "$PROJECT_MOBILE")

  echo "[examples] Generating HTML report for mobile sample project..."
  (cd "$ROOT_DIR" && cargo run -p uto-cli -- report --project "$PROJECT_MOBILE" --html)

  MOBILE_HTML="$PROJECT_MOBILE/.uto/reports/last-run.html"
  if [ ! -f "$MOBILE_HTML" ]; then
    echo "[examples] ERROR: expected HTML report at $MOBILE_HTML"
    exit 1
  fi
fi

PHASE4_PROJECT="$ROOT_DIR/examples/phases/phase4-framework"
PHASE4_JSON="$PHASE4_PROJECT/.uto/reports/last-run.json"
PHASE4_HTML="$PHASE4_PROJECT/.uto/reports/last-run.html"

echo "[examples] Running committed Phase 4 reference project (web)..."
(cd "$ROOT_DIR" && cargo run -p uto-cli -- run --project "$PHASE4_PROJECT" --target web --report-json "$PHASE4_JSON")

echo "[examples] Reporting committed Phase 4 reference project..."
(cd "$ROOT_DIR" && cargo run -p uto-cli -- report --project "$PHASE4_PROJECT" --input "$PHASE4_JSON" --html)

if [ ! -f "$PHASE4_HTML" ]; then
  echo "[examples] ERROR: expected HTML report at $PHASE4_HTML"
  exit 1
fi

echo "[examples] CLI validation completed."
