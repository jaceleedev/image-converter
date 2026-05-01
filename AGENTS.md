# Image Converter 에이전트 가이드

이 파일은 Codex, Claude Code 등 에이전트가 새 세션에서 먼저 참고해야 하는 단일 작업 규칙입니다. 자세한 사용자 문서는 `README.md`, 상세 개발 문서는 `docs/` 아래를 참고하세요.

## 프로젝트 요약

- Rust 2021 기반 이미지 변환 CLI
- 입력: PNG, JPG/JPEG, WebP, AVIF(8-bit), TIFF/TIF, BMP, ICO
- 출력: PNG, JPG/JPEG, WebP, AVIF
- 모드: 단일 파일 변환, 디렉토리 일괄 변환, 대화형 모드
- AVIF 인코딩은 `image` 0.24 AVIF 디코더와 라운드트립 호환성을 위해 8-bit 로 고정
- 출력 포맷은 `src/format.rs` 의 `OutputFormat` enum 으로 관리

## 문서 구조

- `README.md`: 사용자용 설치, 사용법, 옵션
- `docs/README.md`: 개발 문서 인덱스
- `docs/architecture.md`: 모듈 구조와 의존성 흐름
- `docs/testing.md`: 테스트 실행법과 테스트 목록
- `docs/memory.md`: 최근 작업 로그, 결정 기록, 진행/대기 항목
- `AGENTS.md`: 에이전트 공통 작업 규칙

`CLAUDE.md` 는 중복 관리를 피하기 위해 사용하지 않습니다. 에이전트별 차이가 필요하면 이 파일에 공통 규칙을 먼저 반영하고, 각 도구 고유 설정은 최소화하세요.

## 개발 환경

Docker 사용을 기본으로 합니다. 로컬 OS 에 Rust, `nasm`, `dav1d` 를 직접 설치하지 않아도 됩니다.

```bash
docker compose build
./scripts/check.sh
docker compose run --rm dev cargo build --release
docker compose run --rm dev cargo run --release -- -I
```

로컬 Rust 를 쓸 때는 `nasm`, `libdav1d`/`dav1d`, `pkg-config` 가 필요합니다. 자세한 설치법은 `README.md` 를 참고하세요.

## 코드 컨벤션

- 사용자 향 출력, 에러 메시지, 문서, 커밋 메시지는 한국어로 작성
- 함수/변수/타입/외부 API 식별자는 영어 사용
- CLI 출력과 README 예제에는 기존 스타일에 맞춰 의미 있는 이모지를 사용할 수 있음
- 코드 주석은 필요한 경우에만 짧게 작성
- 에러 처리는 `src/error.rs` 의 `ConverterError` 와 `Result<T>` 별칭 사용
- 출력 포맷은 문자열 분기 대신 `OutputFormat` 사용
- 테스트 출력은 `src/tests/test_utils.rs` 의 `test_description!`, `test_step!`, `test_success!` 매크로 사용

## 작업 원칙

- 변경 전 관련 모듈과 테스트를 먼저 확인
- 기능 변경은 가능한 한 작은 단위로 분리
- Docker 환경에서 `./scripts/check.sh` 로 포맷팅, Clippy, 테스트를 함께 검증
- 문서 구조나 개발 흐름을 바꾸면 `README.md`, `docs/architecture.md`, `docs/testing.md`, `docs/memory.md` 중 필요한 문서를 함께 갱신
- `docs/memory.md` 에는 시간에 따라 변하는 작업 로그와 결정 기록만 추가

## 커밋 컨벤션

Conventional Commits + 한국어 본문을 사용합니다.

```text
<type>: <한국어 제목>

- 변경 내용 1
- 변경 내용 2

Co-authored-by: <agent-name> <agent-email>
```

type 후보: `feat`, `fix`, `refactor`, `chore`, `docs`, `style`, `test`, `perf`, `ci`, `build`, `revert`, `init`, `remove`, `rename`, `hotfix`

규칙:
- 제목은 type 뒤 한국어로 작성
- 본문은 `-` 리스트 중심으로 작성
- 마지막 줄의 `Co-authored-by:` 는 실제 함께 작업한 에이전트 이름/이메일에 맞춤
- PR squash merge 시 제목 끝의 `(#N)` 은 GitHub 가 자동 부착
