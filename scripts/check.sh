#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

USE_LOCAL=0
RELEASE_TEST=0

usage() {
    cat <<'USAGE'
사용법: scripts/check.sh [--local] [--release]

기본값은 Docker 개발 컨테이너에서 실행합니다.
  scripts/check.sh           # fmt + clippy + test
  scripts/check.sh --local   # 호스트에 설치된 cargo 사용
  scripts/check.sh --release # release 모드 테스트 실행

필요 도구:
  기본: docker compose
  --local: cargo, rustfmt, clippy, nasm, dav1d/pkg-config
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --local)
            USE_LOCAL=1
            ;;
        --release)
            RELEASE_TEST=1
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "알 수 없는 옵션: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

run() {
    echo
    printf '==> '
    printf '%q ' "$@"
    echo
    "$@"
}

if [[ "$USE_LOCAL" -eq 1 ]]; then
    CARGO_CMD=(cargo)
else
    CARGO_CMD=(docker compose run --rm dev cargo)
fi

run "${CARGO_CMD[@]}" fmt --check
run "${CARGO_CMD[@]}" clippy --all-targets --all-features -- -D warnings

if [[ "$RELEASE_TEST" -eq 1 ]]; then
    run "${CARGO_CMD[@]}" test --release
else
    run "${CARGO_CMD[@]}" test
fi
