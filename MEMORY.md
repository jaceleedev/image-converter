# Memory

세션 간 인계용 작업 컨텍스트. 정적 컨벤션과 구조는 `CLAUDE.md` 에 있고, 여기는 **시간에 따라 변하는 것** (최근 변경, 결정 기록, 진행/대기 항목) 만 기록.

새 세션에서 일을 이어 받을 때 이 파일을 읽으면 어디서 멈췄는지 빠르게 파악할 수 있도록.

## 사용자 컨텍스트

- 한국어 사용자
- 모든 출력 (응답·코드 주석·문서·커밋·CLI 메시지) 한국어로 작성
- 커밋 형식은 `CLAUDE.md` 의 "커밋 컨벤션" 섹션을 따름

## 최근 작업 로그

### 2026-04-30 — `feat: 다중 입출력 포맷 지원 (역변환 + 추가 입력)` (PR 진행 중)

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

- **A안 채택 (AVIF 입력 제외)** — `image` 크레이트의 `avif-decoder` feature 는 `dav1d` 시스템 라이브러리 (`libdav1d-dev` apt / `dav1d` brew) 를 요구. 현재 `nasm` 한 가지만 시스템 의존성으로 잡혀 있는 깔끔함을 유지하기 위해 1단계에서 AVIF 입력만 제외. WebP 디코딩은 `image` 0.24 default features 에 포함되어 Cargo.toml 변경 없이 동작.
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

## 진행 중 / 대기

- [x] **clap v4 마이그레이션** — 완료. derive 매크로로 전환.
- [x] **커스텀 에러 타입 (`thiserror`)** — 완료. `ConverterError` enum + 8 variant.
- [x] **일괄 변환 병렬화 (`rayon`)** — 완료. 16코어에서 8장 AVIF 변환 8.3배 ↑.
- [x] **다중 입출력 포맷 지원 (A안)** — 완료. PNG/JPG/JPEG 출력 + WebP/TIFF/BMP/ICO 입력 추가. 18 테스트 통과.
- [ ] **AVIF 입력 (B안 후속 PR)** — `Cargo.toml` 에 `image = { ..., features = ["avif-decoder"] }` + `is_supported_image()` 화이트리스트에 `"avif"` 한 단어 + `libdav1d` 시스템 의존성 안내. 매우 작은 PR.
- [ ] **`--threads` CLI 옵션** — 현재는 `RAYON_NUM_THREADS` 환경변수만. 작은 추가 작업.
- [ ] **JPG/JPEG 단일 입력 명시 회귀 테스트** — 다중 포맷 PR 에서 혼합 배치로 간접 커버. 명시적 단일 케이스는 별도.
- [ ] **HEIC 입력** — `libheif` 시스템 의존성 + `libheif-rs` 등 외부 크레이트. 부담 큼.
- [ ] **대화형 모드 테스트** — `dialoguer` 입력 모킹이 까다로워 우선순위 낮음.

## 환경 메모

- WSL2 (Ubuntu) 에서 빌드 시 `nasm` 패키지 필요 (rav1e/AVIF). `sudo apt-get install -y nasm`.
- 새 장비 setup 시 `cargo build --release` 가 nasm 없이 실패하면 위 메시지로 안내됨.
