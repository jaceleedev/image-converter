# Memory

세션 간 인계용 작업 컨텍스트. 정적 컨벤션과 에이전트 공통 규칙은 루트 `AGENTS.md` 에 있고, 여기는 **시간에 따라 변하는 것** (최근 변경, 결정 기록, 진행/대기 항목) 만 기록.

새 세션에서 일을 이어 받을 때 이 파일을 읽으면 어디서 멈췄는지 빠르게 파악할 수 있도록.

## 사용자 컨텍스트

- 한국어 사용자
- 모든 출력 (응답·코드 주석·문서·커밋·CLI 메시지) 한국어로 작성
- 커밋 형식은 루트 `AGENTS.md` 의 "커밋 컨벤션" 섹션을 따름

## 최근 작업 로그

### 2026-05-01 — 출력 덮어쓰기 방지

- 단일 변환(`convert_image`, `convert_image_silent`) 에서 출력 경로가 이미 있으면 `ConverterError::OutputExists` 를 반환하고 기존 파일을 보존하도록 변경
- 출력 파일 쓰기는 `fs::write` 대신 `OpenOptions::create_new(true)` 를 사용해 check/write 사이의 경쟁 상황에서도 덮어쓰지 않도록 보강
- 일괄 변환에서는 기존 출력 파일을 실패가 아니라 `skipped` 로 집계하고, 나머지 파일 변환은 계속 진행
  - `process_one` 결과를 변환/건너뜀/실패로 구분
  - 배치 요약에 `건너뜀` 개수 표시
- 회귀 테스트 2개 추가
  - `test_single_conversion_rejects_existing_output`
  - `test_batch_skips_existing_output_files`
- 기존 `test_quality_parameter_bounds` 는 덮어쓰기 방지 정책에 맞춰 최소/최대 품질 출력 파일을 분리
- README / docs/architecture.md / docs/testing.md 갱신
- `./scripts/check.sh` 성공 — fmt 통과, Clippy 경고 없음, lib 39개 + bin 8개 = 총 47개 테스트 통과

### 2026-05-01 — 버전 단일화 + PNG 요약 라벨 개선

- `Cargo.toml` 패키지 버전을 `2.4.0` 으로 올리고, `src/main.rs` 의 clap 버전은 하드코딩 대신 Cargo 패키지 버전을 사용하도록 변경
  - `docker compose run --rm dev cargo run -- --version` 출력: `image_converter 2.4.0`
- 출력 요약의 품질 표시를 `src/utils.rs` 의 `format_quality_label(format, quality)` 로 공통화
  - WebP/JPEG/AVIF: 기존처럼 `품질: N%`
  - PNG: 품질 값 대신 `무손실`
- 단일 변환(`converter.rs`) 과 일괄 변환(`batch.rs`) 요약이 같은 라벨 규칙을 사용하도록 변경
- `format_quality_label` 단위 테스트 2개 추가 — 손실 포맷 품질 라벨 + PNG 무손실 라벨
- README / docs/testing.md / docs/architecture.md 갱신
- `./scripts/check.sh` 성공 — fmt 통과, Clippy 경고 없음, lib 37개 + bin 8개 = 총 45개 테스트 통과

### 2026-05-01 — 로컬 품질 검사 스크립트 추가

- GitHub Actions 같은 원격 CI 는 아직 도입하지 않고, 로컬에서 한 번에 검사하는 스크립트 방식으로 결정
- 신규 `scripts/check.sh` 추가
  - 기본: Docker 개발 컨테이너에서 `cargo fmt --check` → `cargo clippy --all-targets --all-features -- -D warnings` → `cargo test` 순서로 실행
  - `--local`: 호스트에 설치된 Cargo 로 같은 검사 실행
  - `--release`: 테스트만 release 모드로 실행
- `Dockerfile` 에 `clippy` 컴포넌트 설치 추가 (`rustup component add rustfmt clippy`)
- README / docs/testing.md / docs/architecture.md / AGENTS.md 에 로컬 검사 스크립트 사용법 반영
- 검증 완료:
  - `docker compose build` 성공
  - `./scripts/check.sh` 성공 — fmt 통과, Clippy 경고 없음, lib 35개 + bin 8개 = 총 43개 테스트 통과

### 2026-05-01 — 문서 구조 정리

- 루트 문서가 많아진 문제를 줄이기 위해 상세 개발 문서를 `docs/` 아래로 이동
  - `PROJECT_STRUCTURE.md` → `docs/architecture.md`
  - `TESTING.md` → `docs/testing.md`
  - `MEMORY.md` → `docs/memory.md`
- `docs/README.md` 문서 인덱스 추가
- `AGENTS.md` 를 에이전트 공통 작업 규칙의 단일 원천으로 재작성
- 중복 관리되던 `CLAUDE.md` 제거
- README 에 문서 링크 섹션 추가
- `docker compose run --rm dev cargo test` 성공 — lib 35개 + bin 8개 = 총 43개 테스트 통과

### 2026-05-01 — 출력 포맷 `OutputFormat` enum 타입화

- 신규 `src/format.rs` — `OutputFormat` enum (`png`/`jpg`/`jpeg`/`webp`/`avif`) 추가
- `clap::ValueEnum` 으로 `--format` 허용값을 CLI 파싱 단계에서 검증하도록 변경 (`ignore_case = true`)
- `converter.rs` / `batch.rs` / `interactive.rs` 내부 API 가 문자열 대신 `OutputFormat` 을 받도록 전환
- 기존 invalid format 테스트는 `OutputFormat` 파싱 에러 검증으로 변경
- `src/main.rs` 에 `--format WEBP` 대소문자 무시 파싱과 `--format xyz` 거부 단위 테스트 2개 추가
- Docker 개발 이미지에 `rustfmt` 컴포넌트 추가 (`cargo fmt` 사용 가능)
- `docker compose run --rm dev cargo fmt` 성공
- `docker compose run --rm dev cargo test` 성공 — lib 35개 + bin 8개 = 총 43개 테스트 통과

### 2026-05-01 — Docker 개발 환경 추가

- WSL 에서 작업하던 Rust 프로젝트를 macOS / 새 MacBook 으로 옮겨도 같은 개발 환경을 쓰기 위해 Docker 기반 격리 환경 추가
- 신규 `Dockerfile` — `rust:1-trixie` 기반, AVIF 빌드/디코딩에 필요한 `nasm`, `libdav1d-dev`, `pkg-config` 설치. `CARGO_TARGET_DIR=/workspace/target`, `RUST_BACKTRACE=1` 설정
- 신규 `docker-compose.yml` — 현재 repo 를 `/workspace` 로 bind mount, Cargo registry/git/target 은 Docker named volume 으로 분리해서 호스트에 Rust 빌드 산출물이 남지 않게 구성
- 신규 `.dockerignore` — `target`, `.git`, `.DS_Store` 를 Docker build context 에서 제외
- README/TESTING/PROJECT_STRUCTURE/CLAUDE 문서에 Docker 사용법 추가
- 기본 이미지는 `rust:1-trixie`; `dav1d-sys` 가 `dav1d >= 1.3.0` 을 요구해서 Debian bookworm 의 `libdav1d-dev 1.0.0` 으로는 빌드 실패
- 완전 고정이 필요하면 `RUST_IMAGE=rust:1.94-trixie docker compose build` 형태로 override 가능
- 검증 완료:
  - `docker compose config` 성공
  - `docker compose build` 성공
  - `docker compose run --rm dev cargo test` 성공 — lib 35개 + bin 6개 = 총 41개 테스트 통과
  - 컨테이너 버전: `cargo 1.95.0`, `rustc 1.95.0`, `dav1d 1.5.1`

### 2026-04-30 — `refactor: 대화형 모드 검증 분리 + 단위 테스트 + threads 질문 추가` (PR 진행 중)

- `src/interactive.rs` — `validate_with` 인라인 클로저와 디폴트 출력 경로 인라인 코드를 5개 순수 함수로 분리:
  - `validate_input_path(input, is_batch) -> Result<(), &'static str>` — 경로 존재 + 단일/배치 모드 타입 검증
  - `validate_quality_input(input)` — 1.0~100.0 부동소수 범위
  - `validate_threads_input(input)` — 1 이상 정수
  - `default_output_path_for_file(path, format)` — `{stem}_converted.{format}`
  - `default_output_path_for_dir(path, format)` — `{dirname}_converted_{format}`
- 디렉토리 모드 흐름에 **스레드 수 질문 단계** 추가 — `Input::allow_empty(true)` + `validate_with` 로 빈 입력 = `None` (rayon default), 숫자 = `Some(n)`. 출력 경로 입력 직전에 위치
- `convert_directory` 호출 시 hardcoded `None` 대신 위에서 받은 `threads: Option<usize>` 전달
- `src/interactive.rs` 끝에 `#[cfg(test)] mod tests` 추가 — 12 개 단위 테스트 (5 함수 × 2~4 케이스). `validate_input_path` 테스트는 `tempfile::TempDir` 로 실제 fs 케이스 검증
- 기존 단일 모드의 `file_stem().unwrap().to_str().unwrap()` 이중 unwrap 도 함께 안전화 (`.and_then(|s| s.to_str()).unwrap_or("output")`)
- README/PROJECT_STRUCTURE/CLAUDE/TESTING/MEMORY 갱신 — README 의 대화형 단계 안내가 PNG/JPEG 출력 + 디렉토리 모드 단계까지 누락된 상태였는데 이번 기회에 같이 갱신
- 통합 21 + 단위 (lib) 14 + 단위 (main) 6 = **41 테스트 모두 통과**
- `dialoguer` 자체 PTY 통합 테스트는 비용 대비 가치가 낮아 보류 (검증 로직만 분리해서 단위 커버)

### 2026-04-30 — `fix: CLI 인자 검증 강화 (--threads, --quality)` (PR 진행 중)

- `src/main.rs` — `parse_quality(s) -> Result<f32, String>` (1.0~100.0 범위 검증), `parse_threads(s) -> Result<usize, String>` (1 이상 정수 검증) 두 사용자 정의 파서 추가. clap `#[arg(... value_parser = parse_quality)]` / `value_parser = parse_threads` 로 연결
- 한국어 에러 메시지: `"품질은 1.0~100.0 범위여야 합니다 (입력: {q})"`, `"스레드 수는 1 이상이어야 합니다"`, `"'{s}' 는 유효한 숫자가 아닙니다"` — clap 의 영어 wrapper 안에 그대로 노출
- 기존엔 `--threads 0` 이 `rayon::ThreadPoolBuilder::new().num_threads(0).build()` 단계에서 panic 가능, `--quality 200` 은 인코더 분기의 `clamp(1, 100)` 에 의해 silent 100 으로 처리되던 사각지대 해소
- `src/main.rs` 끝에 `#[cfg(test)] mod tests` 추가 — 단위 테스트 6개 (`parse_quality_*` 3개 + `parse_threads_*` 3개): 정상 범위 통과, 범위 외 거부, 비숫자/음수 거부, 한국어 에러 메시지 포함 확인
- `--threads` doc 주석 / README.md `-t` 옵션 설명에 "1 이상" 명시
- 23 통합 + 6 단위 = 29 테스트 모두 통과
- **clap 한계**: `-q -10` 같이 음수 값은 clap 이 `-1` 을 short 옵션으로 해석해서 우리 검증 도달 전에 "unexpected argument" 에러가 남. `--quality=-10` 형태나 `-q 0.5` 로는 정상 차단. 이 한계는 일반적인 clap 동작이라 별도 워크어라운드 안 함
- TESTING.md / CLAUDE.md / README.md / MEMORY.md 갱신

### 2026-04-30 — `test: JPG/JPEG 단일 입력 회귀 테스트 추가` (PR 진행 중)

- `src/tests/integration_tests.rs` — 통합 테스트 3개 추가 (총 20 → 23, 모두 통과):
  - `test_jpeg_input_to_webp` — JPEG 시드를 만든 뒤 WebP 로 변환. RIFF/WEBP 시그니처 검증
  - `test_jpeg_input_to_png` — JPEG → PNG. PNG 매직 바이트(`89 50 4E 47`) 검증
  - `test_jpg_extension_input` — `.jpg` 확장자 입력도 JPEG 디코더에 인식되는지 확인 (입력 측 별칭 회귀 방지)
- 시드 생성: `create_test_image()` 가 `ImageBuffer::save(path)` 로 확장자 자동 감지하므로 `.jpeg`/`.jpg` 로 저장하면 image 크레이트의 JPEG 인코더가 직접 호출됨 (별도 헬퍼 불필요)
- 기존 회귀 테스트 (`test_jpeg_output_from_png_input`, `test_jpg_alias_for_jpeg`, `test_batch_mixed_input_formats`) 가 출력 측 / 혼합 배치만 커버하던 공백을 채움
- TESTING.md / CLAUDE.md 갱신 — 테스트 개수 20 → 23, 향후 후보에서 "JPG/JPEG 단일 입력 명시 케이스" 제거

### 2026-04-30 — `feat: --threads CLI 옵션 추가` (PR 진행 중)

- `src/main.rs` — `Cli` 에 `threads: Option<usize>` 필드 (`-t/--threads <N>`) 추가. 단일 변환에는 영향 없고 디렉토리 모드일 때만 `convert_directory` 에 전달. 버전 `2.3` → `2.4`
- `src/batch.rs` — `convert_directory` 시그니처에 `threads: Option<usize>` 인자 추가. `Some(n)` 이면 `rayon::ThreadPoolBuilder::new().num_threads(n).build()?` 로 local pool 만들고 `pool.install(|| par_iter().map().collect())` 패턴으로 scoped 실행. `None` 이면 rayon 전역 풀(`RAYON_NUM_THREADS` 또는 CPU 코어) 사용
- `src/error.rs` — `ThreadPool(#[from] rayon::ThreadPoolBuildError)` variant 추가. 시그니처 외 외부 크레이트 에러는 모두 `#[from]` 패턴 일관 유지
- `src/interactive.rs` — `convert_directory` 호출에 `None` 전달 (대화형 모드는 default 사용, 질문 추가는 향후 후보)
- 통합 테스트 1개 추가 (총 19 → 20, 모두 통과): `test_batch_with_explicit_threads` — `threads=None` 과 `threads=Some(1)` 두 모드가 같은 4개 입력에 대해 같은 성공 개수를 내는지 검증 (스레드 수와 결과 무관 보장)
- 기존 batch 테스트 6개 모두 시그니처 변경에 맞춰 마지막 인자에 `None` 추가
- README/CLAUDE/PROJECT_STRUCTURE/TESTING 갱신 — `-t/--threads` 옵션 안내 + 향후 후보에서 `--threads` 제거

### 2026-04-30 — `feat: AVIF 입력 디코딩 추가 (B안)` (PR #11, merged)

- `Cargo.toml` — `image = { version = "0.24.2", features = ["avif-decoder"] }` 로 변경. `mp4parse` / `dcv-color-primitives` / `dav1d-sys` / `dav1d` 의존성 자동 추가
- 시스템 의존성 추가: `libdav1d-dev` (apt) / `dav1d` (brew). 빌드 시 `pkg-config` 로 동적 링크
- `src/batch.rs` `is_supported_image()` 화이트리스트에 `"avif"` 한 단어 추가 + 빈 디렉토리 안내 메시지 갱신
- **호환성 이슈 + 해결**: `ravif` default 인코딩이 10-bit AVIF 를 만들지만 `image` 0.24 의 AVIF 디코더는 8-bit 만 지원해서 라운드트립이 깨졌음. `src/converter.rs` AVIF 분기에 `.with_bit_depth(BitDepth::Eight)` 추가하여 인코딩을 8-bit 로 강제. `use ravif::BitDepth` import 추가
- `src/converter.rs` 의 AVIF 인코더 호출에 8-bit 강제 결정 이유 한 줄 주석
- `ravif::Encoder::with_depth(Some(8))` 는 deprecated 됐고 `with_bit_depth(BitDepth::Eight)` 가 신 API
- 통합 테스트 1개 추가 + 1개 갱신 (총 18 → 19, 모두 통과):
  - 신규 `test_avif_input_to_png` — PNG 시드를 AVIF 로 인코딩 → 다시 PNG 로 디코딩하는 라운드트립
  - 갱신 `test_batch_mixed_input_formats` — 입력 포맷 4개 (PNG/WebP/TIFF/BMP) → 5개 (+ AVIF)
- README/CLAUDE/PROJECT_STRUCTURE/TESTING 갱신 — 매트릭스 표 AVIF 입력 ✅, 시스템 요구사항에 `libdav1d-dev` 추가, AVIF 8-bit 한계 명시, 향후 후보에 `image` 0.25 업그레이드 추가

### 2026-04-30 — `feat: 다중 입출력 포맷 지원 (역변환 + 추가 입력)` (PR #10, merged)

- `Cargo.toml` **변경 없음** — 모든 신규 포맷이 `image` 0.24.9 의 default features 로 이미 가능
- `src/converter.rs` `encode_to()` — 기존 `webp`/`avif` 분기에 `png`, `jpg|jpeg` 추가
  - **PNG**: `image::ImageOutputFormat::Png` + `Cursor` 로 무손실 인코딩. `quality` 인자는 조용히 무시
  - **JPEG**: 알파 채널 미지원이라 `to_rgb8()` 로 RGB 다운샘플 후 `ImageOutputFormat::Jpeg(quality.clamp(1, 100) as u8)`. `jpg`/`jpeg` 둘 다 같은 분기로 처리
- `src/batch.rs` `is_supported_image()` — 입력 화이트리스트에 `webp`, `tiff`, `tif`, `bmp`, `ico` 추가. AVIF 는 의도적으로 제외 (별도 PR)
- `src/interactive.rs` — 출력 포맷 선택지에 PNG/JPEG 추가. PNG 선택 시 quality 단계 자동 스킵 + "무손실이라 적용 안 됨" 한 줄 안내 출력
- `src/main.rs` — doc 주석 / `-f` value 설명 / `-q` 설명에 PNG 무손실 안내 / 버전 `2.2` → `2.3`
- 통합 테스트 6개 추가 (총 12 → 18, 모두 통과):
  - `test_png_output_from_webp_input` — WebP → PNG 역변환 + PNG 시그니처 검증
  - `test_jpeg_output_from_png_input` — PNG → JPEG + JPEG 시그니처 검증
  - `test_jpg_alias_for_jpeg` — `jpg` 가 `jpeg` 와 동일 분기인지 확인
  - `test_tiff_input_to_webp` — TIFF 입력 디코딩
  - `test_bmp_input_to_jpeg` — BMP 입력 디코딩
  - `test_batch_mixed_input_formats` — PNG + WebP + TIFF + BMP 4종이 한 디렉토리에 섞인 일괄 변환 (모두 PNG 로 통합 출력)
- README/CLAUDE/PROJECT_STRUCTURE 갱신 — 지원 포맷 매트릭스 표 추가

### 2026-04-30 — `perf: rayon 으로 일괄 변환 병렬화` (PR #9, merged)

- `Cargo.toml` — `rayon = "1.10"` 추가

- `src/batch.rs` — 직렬 `for file in &files` 루프를 `files.par_iter().map(process_one).collect()` 패턴으로 전환. 각 파일 처리는 `process_one(file, in_dir, out_dir, fmt, q, &pb) -> Option<ConvertStats>` 헬퍼로 분리. 결과는 직렬 합산
- 한 파일의 OsStr→str 인코딩 실패 시 기존엔 `?` 로 outer 함수가 통째로 실패했지만, 이제 그 파일만 실패 처리하고 나머지는 그대로 진행 (병렬 의미상 더 정확)
- ProgressBar 와 `pb.println` 은 indicatif 내부 Mutex 로 thread-safe — 별도 락 없이 그대로 공유
- 진행률 메시지(`pb.set_message`)는 race 가 noisy 해서 제거. 진행률 카운트 + 마지막 finish 메시지만 표시
- **성능**: 16코어 환경에서 800x800 노이즈 PNG 8장 → AVIF q80 변환 비교
  - 직렬 (`RAYON_NUM_THREADS=1`): **28.87s**
  - 병렬 (default=16): **3.48s** (≈ **8.3배** 속도 향상, 8개 파일 / 거의 이론적 최대치에 근접)
- 12개 테스트 그대로 통과, 단일 변환과 빈 디렉토리 처리 회귀 없음
- 사용자가 스레드 수 조절 시 `RAYON_NUM_THREADS` 환경변수 사용 (별도 CLI 플래그는 향후 후보)
- README/CLAUDE.md/PROJECT_STRUCTURE.md 갱신

### 2026-04-30 — `refactor: thiserror 기반 ConverterError 도입` (PR #8, merged)

- 신규 `src/error.rs` — `ConverterError` enum + `Result<T>` 별칭. variant: `Io` / `Image` / `Webp(String)` / `Avif` / `UnsupportedFormat(String)` / `InvalidPath(String)` / `Dialog` / `QualityParse`
- 외부 크레이트 에러는 `#[from]` 으로 자동 변환 (`std::io::Error`, `image::ImageError`, `ravif::Error`, `dialoguer::Error`, `std::num::ParseFloatError`). webp 의 `&str` 에러만 수동 `to_string()` 매핑
- `converter.rs` / `batch.rs` / `interactive.rs` 시그니처를 `Result<T, Box<dyn Error>>` → `Result<T>` 로 통일
- `interactive.rs` 의 dialoguer `validate_with` 클로저는 `Result<(), &str>` 시그니처를 강제하므로, 이름 충돌 회피 위해 시그니처에 `crate::error::Result<()>` fully qualified 사용 (use 제거)
- 에러 메시지가 컨텍스트를 포함하는 형태로 풍부해짐 (`"지원하지 않는 포맷입니다: xyz"`, `"입출력 오류: No such file or directory"` 등)
- `test_invalid_format` 의 메시지 비교를 `assert_eq!` → `contains` 검증으로 갱신 (포맷명까지 검증 추가)
- 12개 테스트 그대로 통과, 단일/배치/재귀 CLI 동작 회귀 없음
- `Cargo.toml` 에 `thiserror = "1.0"` 추가
- `CLAUDE.md` / `PROJECT_STRUCTURE.md` 갱신

### 2026-04-30 — `refactor: clap v3 → v4 마이그레이션` (PR #7, merged)

- `Cargo.toml` — `clap = "3.2.8"` → `clap = { version = "4.5", features = ["derive"] }`
- `src/main.rs` — Builder 패턴(`App::new(...).arg(...)`)을 `#[derive(Parser)]` 구조체로 전면 재작성. 도움말은 doc 주석으로, `quality` 는 `f32` 자동 파싱(`.parse::<f32>().expect()` 제거), `interactive`/`recursive` 는 `bool` 자동 매핑
- 코드량: 79줄 → 56줄 (약 30% 감소)
- 도움말 출력이 v3 보다 깔끔해지고, 인자 누락 시 어떤 인자가 빠졌는지 명확하게 안내됨
- 12개 테스트 그대로 통과, 단일/배치/재귀 CLI smoke test 회귀 없음
- 버전 표시 `2.1` → `2.2`

### 2026-04-30 — `feat: 디렉토리 일괄 변환 모드 추가` (PR #5, merged)

- 신규 `src/batch.rs` 모듈 — `walkdir` 기반 디렉토리 순회, `BatchSummary` 통계 반환
- `src/converter.rs` 리팩토링 — `encode_to()` 헬퍼 분리로 `convert_image` / `convert_image_silent` 가 동일 내부 공유
- `lib.rs` 의 중복 `convert_image_silent` 정의 제거, `main.rs` 가 lib을 통해 import (bin/lib 중복 컴파일 제거)
- CLI: 입력 경로가 디렉토리이면 자동 일괄 모드, `-r/--recursive` 추가
- 대화형 모드: 첫 단계에 단일/디렉토리 선택, 디렉토리 모드에서 재귀 여부 질문
- 통합 테스트 5개 추가 (총 7→12, 모두 통과)
- `walkdir = "2.5"` 추가
- README/PROJECT_STRUCTURE/TESTING 문서 갱신

## 결정 기록

- **`--threads` 는 local thread pool 패턴 (전역 풀 변경 X)** — `rayon::ThreadPoolBuilder::build_global()` 은 프로세스당 한 번만 호출 가능해서 라이브러리 코드에서 사용하면 사용자 코드와 충돌 가능. 대신 `build()?` 로 local pool 을 만들고 `pool.install(|| par_iter)` 로 scope 안에서만 적용. `convert_directory` 의 시그니처에 `threads: Option<usize>` 를 받아 명시적이면 local pool, `None` 이면 전역 풀 그대로.
- **`Option<usize>` vs `usize` (default 0)** — `clap` 으로 받을 때 `Option<usize>` 가 "사용자가 명시했는지" 와 "default 사용" 을 명확히 구분해줌. `0 = default` 컨벤션은 마술적이라 회피.
- **CLI 플래그가 `RAYON_NUM_THREADS` 환경변수보다 우선** — `-t N` 으로 명시한 경우 local pool 을 만들어 그 안에서 실행하므로 환경변수가 무시됨. 직관적이고 사용자가 한 번에 통제 가능.
- **대화형 모드는 스레드 수 질문 안 함** — 단계 늘리기보다 default(=모든 코어) 가 일반 사용자에게 합리적. CLI 사용자만 명시적 통제. 추가 질문은 향후 후보.
- **AVIF 인코딩을 8-bit 로 강제 (`with_bit_depth(BitDepth::Eight)`)** — `ravif` default 가 10-bit (Auto = Ten) 인데 `image` 0.24 AVIF 디코더는 8-bit 만 지원. 강제 안 하면 우리가 만든 AVIF 를 우리가 디코딩 못 하는 라운드트립 모순 발생. 일반 사진 변환 용도에서는 8-bit 로도 충분하고 호환성(타 뷰어/디코더) 도 더 좋음. 단점은 HDR / 부드러운 그라디언트 케이스에서 약간 손해 — `image` 0.25 업그레이드로 10-bit 디코딩 지원되면 default 풀어줄 후보.
- **외부 10-bit AVIF 입력은 명시적으로 미지원** — `image` 0.24 의 디코더 한계. 사용자가 다른 도구로 만든 10-bit AVIF 를 입력으로 쓰면 `Only 8 bit depth is supported but was 10` 에러. README 매트릭스 표 비고에 한 줄 명시. `image` 0.25 업그레이드를 향후 후보로 정리.
- **A안 채택 (1단계에서 AVIF 입력 제외)** — `image` 크레이트의 `avif-decoder` feature 는 `dav1d` 시스템 라이브러리 (`libdav1d-dev` apt / `dav1d` brew) 를 요구. 1단계는 시스템 의존성 추가 없는 작업으로 분리하고, B안 (AVIF 입력) 은 별도 PR 로 진행해 의존성 변경의 의도를 명확히 함.
- **PNG quality 는 조용히 무시** — `encode_to("png", ...)` 가 quality 인자를 받지만 사용하지 않음. CLI 와 대화형 모드에서 사용자에게 "무손실이라 적용 안 됨" 한 줄 안내 + `-q` doc 주석에도 명시. 에러로 거부하지 않는 이유는 배치 모드에서 한 quality 값을 여러 출력 포맷에 공통 적용하는 흐름을 깨지 않기 위함.
- **JPEG 출력 시 `to_rgb8()` 로 명시적 RGB 다운샘플** — `image` 의 `write_to(..., ImageOutputFormat::Jpeg(q))` 는 RGBA 입력도 받지만 동작이 버전에 따라 다를 수 있음. 명시적으로 `DynamicImage::ImageRgb8(img.to_rgb8())` 로 변환 후 인코딩하여 알파 채널 처리를 확정적으로 만듦. 알파 픽셀은 검정 배경 위에 합성된 형태로 처리됨 (image 크레이트 기본 동작).
- **`jpg` 와 `jpeg` 를 같은 분기로** — `match` 의 or-pattern (`"jpg" | "jpeg"`) 으로 한 분기에서 처리. 사용자 친화적이면서 코드 중복 없음. 출력 확장자도 사용자가 명시한 그대로 사용 (둘 다 동일 JPEG 컨테이너).
- **WebP 픽스처는 webp 크레이트로 생성** — `image::ImageBuffer::save("path.webp")` 는 `image` 의 `webp-encoder` feature (`libwebp` 의존) 가 필요해서 사용 못 함. 테스트에서는 PNG 시드를 만든 후 `convert_image_silent(seed.png, fixture.webp, "webp", ...)` 로 우회.
- **rayon `par_iter().map().collect()` + 직렬 합산** — Atomic 카운터나 `fold/reduce` 보다 단순. `BatchSummary` 구조체 변경 없이 결과만 병렬 수집 후 한 번에 누적.
- **rayon 도입 이후 path 인코딩 실패의 의미 변경** — 직렬 시절엔 `?` 로 outer 함수까지 propagation 되어 한 파일이 실패하면 전체 일괄 변환이 중단됐는데, 병렬화하면서 "그 파일만 실패" 패턴으로 변경. 일괄 변환의 자연스러운 의미와 더 잘 맞음.
- **진행률 메시지 제거** — `pb.set_message(file_name)` 은 병렬에서 race 로 깜빡깜빡거려 정보보다 noise. 카운트(`{pos}/{len}`)만 보여주고 finish 메시지로 마무리.
- **스레드 수는 환경변수로** — `RAYON_NUM_THREADS=N` 으로 제어. 명시적 `--threads` CLI 플래그는 단순함을 위해 보류, 필요해지면 추가.
- **`crate::error::Result` 를 1-generic alias 로 정의** — `pub type Result<T> = std::result::Result<T, ConverterError>`. 코드량은 줄지만 dialoguer 의 `validate_with` 클로저(`Result<(), &str>` 시그니처) 와 이름이 충돌함. interactive.rs 에서는 use 를 제거하고 시그니처에 `crate::error::Result<()>` fully qualified 사용으로 회피.
- **WebP 에러는 `String` 으로 owned 변환** — `webp::Encoder::from_image` 가 `Result<Self, &str>` 를 반환. `&str` 은 `#[from]` 대상이 안 되고, `'static` 이 아니라 그대로 들고 있을 수도 없음. `Webp(String)` variant 로 받고 `.to_string()` 으로 매핑.
- **`UnsupportedFormat` 에 포맷 문자열 포함** — 기존 `"지원하지 않는 포맷입니다"` 정적 메시지를 `"지원하지 않는 포맷입니다: {0}"` 로 변경. 디버그/사용자 경험 모두 향상되지만 기존 `assert_eq!` 테스트가 깨짐 → `contains` 검증으로 함께 갱신.
- **clap v4 derive 매크로 채택** — 단순 builder 버전 업이 아니라 derive 로 전환. 코드 30% 감소, doc 주석이 자동으로 도움말이 됨, 타입 자동 파싱(특히 `quality: f32`). 변경 범위는 `main.rs` + `Cargo.toml` 로 한정되어 안전했음.
- **`required_unless_present = "interactive"` 패턴 유지** — derive 에서도 동일한 의미. `Option<String>` + 비대화형 모드에서 `expect()` 로 추출. `ArgGroup` 도입은 과한 복잡도라 보류.
- **`walkdir` 채택** — 재귀+필터 조합을 std `read_dir` 로만 처리하면 verbose 함. 비재귀에도 `WalkDir::new(path).max_depth(1)` 로 통일하여 코드 분기 단순화.
- **`encode_to()` 헬퍼 분리** — 단일 변환의 UI 포함 버전과 조용한 버전, 그리고 일괄 변환에서 호출되는 inner 까지 세 곳이 동일 인코딩 로직을 쓰는 상황. 헬퍼로 분리하여 한 군데에서만 유지.
- **`convert_image_silent` 의 반환 타입을 `()` → `ConvertStats`** — 일괄 모드의 합계 통계 계산을 위해. 기존 테스트는 `?` 로만 결과를 처리해서 시그니처 변경에 영향 없음.
- **bin/lib 중복 컴파일 제거** — `main.rs` 가 `mod converter;` 등으로 모듈을 직접 포함하면 lib 과 bin 양쪽에서 같은 코드가 컴파일되고 `convert_image_silent` 가 bin 측에서 dead_code 경고. `main.rs` 를 `image_converter::*` 로 import 하도록 변경하여 해결.
- **CLI 인자 검증은 사용자 정의 파서로 통일** — clap v4 의 `clap::value_parser!().range(..)` 는 정수 (`u16`/`usize` 등) 만 직접 지원하고 `f32`/`f64` 는 안 됨. quality (float) 와 threads (usize) 두 인자가 있어서 한쪽만 빌트인 range 를 쓰면 일관성이 깨지므로, 양쪽 다 `fn parse_quality(&str) -> Result<f32, String>` / `fn parse_threads(&str) -> Result<usize, String>` 사용자 정의 파서로 통일. 부수 효과로 한국어 에러 메시지 (`"품질은 1.0~100.0 범위여야 합니다"`) 를 직접 작성 가능 — clap 의 영어 wrapper (`"invalid value '0' for ..."`) 안에 우리 사유가 그대로 노출됨.
- **음수 인자 (`-q -10`) 는 clap 한계로 별도 워크어라운드 안 함** — clap 은 `-` 시작 토큰을 short option 으로 해석해서 `-1` 이 unknown flag 로 거부됨. `allow_negative_numbers` attribute 를 켜면 우회 가능하지만 사용자가 음수 quality 를 의도적으로 넣는 경우는 거의 없고, `--quality=-10` / `-q 0.5` 형태로는 우리 검증이 정상 차단하므로 단순함을 위해 보류.
- **대화형 모드 검증은 dialoguer 모킹 대신 순수 함수 분리로 단위 테스트** — `dialoguer::Select` 는 raw TTY 모드라 stdin pipe 시뮬레이션이 안 됨. PTY 도구 (`rexpect` 등) 도입은 환경 의존성이 커서 flaky 위험. 대신 `validate_with` 클로저와 디폴트 경로 빌더처럼 **검증 가치가 큰 순수 로직만 함수로 분리**하면 dialoguer 한 줄도 안 건드리고 단위 테스트 가능. 흐름 자체의 통합 테스트는 비용 대비 가치가 낮아 보류.
- **대화형 모드 스레드 수 질문은 한 단계 (`allow_empty=true`) 패턴** — 이전 결정 ("대화형 모드는 스레드 수 질문 안 함") 을 부분 뒤집음. `Confirm` (예/아니오) + `Input` (값) 두 단계로 나누면 친절하지만 늘어짐. 한 단계 (`Input::allow_empty(true)`) 로 통합: 빈 입력 = `None` (모든 코어), 숫자 = `Some(n)`. 사용자 친화 + 단계 수 최소.
- **기존 출력 파일은 덮어쓰지 않음** — 단일 변환은 `OutputExists` 에러로 중단, 일괄 변환은 해당 파일만 `skipped` 로 집계. 출력 쓰기는 `create_new(true)` 로 처리해 변환 도중 같은 출력 경로가 생겨도 덮어쓰지 않음.

## 진행 중 / 대기

- [x] **clap v4 마이그레이션** — 완료. derive 매크로로 전환.
- [x] **커스텀 에러 타입 (`thiserror`)** — 완료. `ConverterError` enum + 9 variant.
- [x] **일괄 변환 병렬화 (`rayon`)** — 완료. 16코어에서 8장 AVIF 변환 8.3배 ↑.
- [x] **다중 입출력 포맷 지원 (A안)** — 완료. PNG/JPG/JPEG 출력 + WebP/TIFF/BMP/ICO 입력 추가. 18 테스트 통과.
- [x] **AVIF 입력 (B안)** — 완료. `avif-decoder` feature 활성화 + `libdav1d-dev` 의존성 추가. 라운드트립을 위해 ravif 인코딩을 8-bit 로 강제. 19 테스트 통과.
- [x] **`--threads` CLI 옵션** — 완료. `convert_directory` 가 `Option<usize>` 를 받아 local pool 로 scoped 실행. CLI 우선, 환경변수 fallback. 20 테스트 통과.
- [x] **JPG/JPEG 단일 입력 명시 회귀 테스트** — 완료. 통합 테스트 3개 추가 (jpeg→webp, jpeg→png, .jpg 확장자 입력). 23 테스트 통과.
- [x] **CLI 인자 검증 강화** — 완료. `parse_quality` / `parse_threads` 사용자 정의 파서 + 단위 테스트 6개. `--threads 0` 한국어 메시지로 차단, `--quality 200/0.5/abc` 거부. 29 테스트 통과.
- [x] **대화형 모드 리팩토링 + 단위 테스트 + `--threads` 질문 추가** — 완료. 검증/디폴트 빌더 5개 함수 분리, 단위 테스트 12개 추가, 디렉토리 모드에 빈-입력=`None` 패턴의 스레드 수 질문 단계 삽입. 41 테스트 통과.
- [x] **출력 덮어쓰기 방지** — 완료. 단일 변환은 기존 출력 파일에 `OutputExists` 에러, 일괄 변환은 기존 출력 파일을 `skipped` 로 집계. 47 테스트 통과.
- [ ] **`image` 0.25 업그레이드** — 10-bit AVIF 디코딩 지원. breaking change 가 있어 별도 작업. 업그레이드 후 ravif 의 8-bit 강제도 풀어줄 수 있음
- [ ] **HEIC 입력** — `libheif` 시스템 의존성 + `libheif-rs` 등 외부 크레이트. 부담 큼.
- [ ] **대화형 모드 통합 테스트** — `dialoguer::Select` 가 PTY 필요해서 `rexpect` 등 도입 부담. 우선순위 낮음 (위 리팩토링으로 검증/경로 로직은 단위 테스트로 커버됨).

## 환경 메모

- 빌드 시 두 시스템 라이브러리가 필요:
  - `nasm` — rav1e (AVIF 인코딩) 빌드용
  - `libdav1d-dev` (apt) / `dav1d` (brew) — dav1d-sys (AVIF 디코딩) 빌드용. `pkg-config` 로 동적 링크
- 한 줄 설치: `sudo apt-get install -y nasm libdav1d-dev` (Ubuntu/WSL) / `brew install nasm dav1d` (macOS)
- 빌드 실패 시그널: `Package dav1d was not found in the pkg-config search path` 는 `libdav1d-dev` 누락. nasm 누락은 rav1e 컴파일 단계에서 명확한 에러로 안내됨.
