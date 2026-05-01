# Image Converter

여러 이미지 포맷 (PNG/JPG/JPEG/WebP/AVIF/TIFF/BMP/ICO) 을 PNG/JPG/WebP/AVIF 로 양방향 변환하는 Rust CLI 도구.

## 프로젝트 개요

- **언어/도구**: Rust (edition 2021)
- **지원 입력**: PNG, JPG, JPEG, WebP, AVIF (8-bit), TIFF, BMP, ICO
- **지원 출력**: PNG, JPG/JPEG, WebP, AVIF
- **AVIF 인코딩은 8-bit 로 고정** — `image` 0.24 의 AVIF 디코더가 8-bit 만 지원해서 라운드트립 호환성 유지
- **모드**: 단일 파일 변환 + 디렉토리 일괄 변환 (재귀 옵션, 스레드 수 명시 가능)
- **인터페이스**: 명령줄 모드 (`-i/-o/-f/-q/-r/-t`) + 대화형 모드 (`-I`)
- **변환 엔진**: WebP 인코딩은 `webp` 크레이트, AVIF 인코딩은 `ravif` (rav1e 기반), AVIF 디코딩은 `image` 의 `avif-decoder` feature (`dav1d` 시스템 의존), 그 외 PNG/JPEG/WebP/TIFF/BMP/ICO 디코딩과 PNG/JPEG 인코딩은 `image` 크레이트

세부 사용법은 `README.md` 참고.

## 모듈 구조

```
src/
├── main.rs            # CLI 진입점, 단일/일괄 분기
├── lib.rs             # 공개 API re-export (main.rs도 이 lib을 통해 import)
├── error.rs           # ConverterError + Result 타입 (thiserror 기반)
├── format.rs          # OutputFormat enum + CLI/내부 공통 출력 포맷 파싱
├── converter.rs       # 단일 변환. encode_to() 가 webp/avif/png/jpg|jpeg 분기를
│                      # 처리하고, convert_image(UI 포함) / convert_image_silent
│                      # (조용함) 가 동일 내부를 공유. AVIF 인코딩은 8-bit 고정
├── batch.rs           # 디렉토리 일괄 변환 (walkdir + rayon 병렬, 재귀 옵션, 구조 미러링).
│                      # 입력 화이트리스트: png/jpg/jpeg/webp/avif/tiff/tif/bmp/ico
├── interactive.rs     # 대화형 모드 (단일/디렉토리 선택 → 출력 포맷 → 옵션 → 실행).
│                      # PNG 출력 시 quality 단계는 자동 스킵
├── utils.rs           # format_file_size 등 헬퍼
└── tests/             # 단위 + 통합 테스트 (총 43개)
```

세부 책임은 `PROJECT_STRUCTURE.md` 참고.

## 시스템 요구사항

### Docker 사용 시 (권장)

- **Docker Desktop / Docker Engine**
- 로컬 OS 에 Rust, `nasm`, `dav1d` 를 직접 설치하지 않아도 됨

```bash
docker compose build
docker compose run --rm dev cargo test
docker compose run --rm dev cargo build --release
docker compose run --rm dev
```

### 로컬 설치 시

- **Rust toolchain** (cargo 1.94+)
- **nasm** — AVIF 인코더(`rav1e`) 빌드에 필요
- **libdav1d** — AVIF 디코더(`dav1d-sys`) 빌드에 필요 (`pkg-config` 로 동적 링크)

```bash
# WSL/Ubuntu
sudo apt-get install -y nasm libdav1d-dev

# macOS
brew install nasm dav1d
```

빌드 시 `pkg-config: Package dav1d was not found` 에러가 나면 `libdav1d-dev` (Linux) / `dav1d` (macOS) 가 빠졌다는 신호.

## 개발 명령어

```bash
# Docker (권장)
docker compose run --rm dev cargo build
docker compose run --rm dev cargo fmt
docker compose run --rm dev cargo test
docker compose run --rm dev cargo build --release

# 빌드
cargo build              # 디버그
cargo build --release    # 릴리즈 (실제 변환 속도 측정/사용 시)

# 테스트
cargo test                              # 기본
cargo test --release                    # 릴리즈 모드 (AVIF 테스트가 빠름)
cargo test -- --test-threads=1 --nocapture  # 출력 깔끔하게 (TESTING.md 추천)
cargo test test_batch                   # 일괄 변환 테스트만

# 실행 (릴리즈)
./target/release/image_converter -I                                # 대화형
./target/release/image_converter -i a.png -o a.webp -f webp        # 단일
./target/release/image_converter -i photos -o out -f webp -r       # 디렉토리 재귀
./target/release/image_converter -i photos -o out -f webp -r -t 4  # 4 스레드로 제한
```

## 코드 컨벤션

- **언어**
  - 사용자 향 출력 (CLI 메시지, 에러, 진행 안내), 주석, 문서, 커밋 메시지: **한국어**
  - 함수/변수/타입 식별자, 외부 API 이름: 영어
  - 한국어 안에 영어 고유명사·기술용어·심볼은 그대로 (예: `WebP 인코더 생성 중...`)
- **이모지**
  - CLI 출력과 README 예제에서 의미 있는 시각 단서로 사용 (`🚀 시작`, `✅ 성공`, `❌ 실패`, `📁 입력`, `💾 출력`)
  - 코드 주석/내부 로그에는 사용하지 않음
- **에러 처리**
  - `src/error.rs` 의 `ConverterError` enum (thiserror 기반) + `Result<T>` 별칭 사용
  - 외부 크레이트 에러(io, image, dialoguer, ravif, ParseFloat) 는 `#[from]` 으로 자동 변환
  - WebP 인코더의 `&str` 에러는 `to_string()` 으로 String 변환 후 `Webp(String)` variant
  - 사용자 입력성 에러는 `UnsupportedFormat(String)`, `InvalidPath(String)` 등 명시적 variant — Display 메시지에 컨텍스트 포함
- **테스트 출력**
  - `test_description!`, `test_step!`, `test_success!` 매크로로 가독성 있는 로그 (정의: `src/tests/test_utils.rs`)
  - 통합 테스트는 `tempfile` 로 격리

## 커밋 컨벤션

Conventional Commits + 한국어 본문.

```
<type>: <한국어 제목>

- 변경 내용 1
- 변경 내용 2

Co-authored-by: <agent-name> <agent-email>
```

**type**: `feat` · `fix` · `refactor` · `chore` · `docs` · `style` · `test` · `perf` · `ci` · `build` · `revert` · `init` · `remove` · `rename` · `hotfix`

**규칙**
- 제목은 type 뒤 한국어. 영어 고유명사/기술용어/심볼은 그대로
- 본문은 `-` 리스트, 백틱으로 코드/심볼 인용 가능
- 마지막 줄에 `Co-authored-by:` (소문자 `a`/`b`) 트레일러 — 함께 작업한 에이전트에 맞는 이름/이메일 사용
- PR squash merge 시 제목 끝에 `(#N)` 자동 부착

## 향후 개선 후보

(우선순위 추정 순)

1. **`image` 0.25 업그레이드** — 0.24 의 AVIF 디코더가 8-bit 만 지원하는 한계 해소 (10-bit AVIF 입력 지원). breaking change 가 있어 별도 작업
2. **HEIC 입력** — iPhone 사진 변환용. `libheif` 시스템 의존성 + `libheif-rs` 등 외부 크레이트 도입 필요
3. **대화형 모드 통합 테스트** — 검증/경로 빌더는 단위 테스트로 커버됨. `dialoguer::Select` 가 raw TTY 라 `rexpect` 등 PTY 도구 필요
4. **다국어/메시지 분리** — 메시지를 별도 파일/리소스로

## 관련 문서

- `README.md` — 사용자용 사용법, 옵션, 예제 출력
- `PROJECT_STRUCTURE.md` — 모듈별 책임, 의존성 흐름, 향후 개선 제안
- `TESTING.md` — 테스트 실행 방법, 테스트 목록, 매크로 사용법
- `MEMORY.md` — 작업 컨텍스트, 결정 기록, 진행 중/대기 항목 (세션 간 인계용)
