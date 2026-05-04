#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_DIR="$ROOT_DIR/apps/web"

cd "$WEB_DIR"

run() {
    echo
    printf '==> '
    printf '%q ' "$@"
    echo
    "$@"
}

if [[ ! -d node_modules ]]; then
    run npm install
fi

run npm run lint
run npm run build
