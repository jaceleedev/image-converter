# 🖼️ Image Converter

웹 개발에 필요한 이미지를 대화형 안내로 변환하는 Rust CLI 도구. WebP/AVIF 압축뿐 아니라 PNG/JPEG 로의 역변환도 지원합니다.

## ✨ 주요 기능

- **양방향 포맷 변환**: PNG/JPG/JPEG/WebP/AVIF/TIFF/BMP/ICO → PNG/JPG/WebP/AVIF
- **단일 파일 + 디렉토리 일괄 변환**: 입력이 디렉토리이면 자동으로 일괄 모드 (rayon 으로 멀티코어 병렬 처리)
- **재귀 변환**: `--recursive` 옵션으로 하위 폴더까지 한 번에 변환 (구조 그대로 미러링)
- **대화형 기본 실행**: 인자 없이 실행하면 단일/디렉토리 변환을 단계별로 안내
- **용량 비교**: 변환 전후 파일 크기 및 감소율 표시 (배치 모드는 합계까지)
- **진행 상황 표시**: 실시간 진행률 표시
- **품질 설정**: 1-100% 품질 조정 가능 (PNG 는 무손실이라 자동 무시)
- **덮어쓰기 방지**: 기존 출력 파일을 보존 (단일 변환은 중단, 일괄 변환은 해당 파일 건너뜀)
- **아름다운 UI**: 이모티콘과 색상으로 보기 좋은 출력

## 📚 문서

- [에이전트 가이드](AGENTS.md): Codex/Claude Code 등 에이전트 공통 작업 규칙
- [개발 문서 인덱스](docs/README.md): 개발 문서 목록
- [프로젝트 구조](docs/architecture.md): 모듈별 책임과 의존성 흐름
- [테스트 가이드](docs/testing.md): 테스트 실행법과 테스트 목록
- [작업 메모리](docs/memory.md): 최근 작업 로그와 결정 기록

## 🎯 지원 포맷 매트릭스

| 포맷       | 입력 | 출력 | 비고                                                                                                |
| ---------- | :--: | :--: | --------------------------------------------------------------------------------------------------- |
| PNG        |  ✅  |  ✅  | 무손실. 출력 시 `--quality` 무시                                                                    |
| JPG / JPEG |  ✅  |  ✅  | 알파 채널 미지원 — 자동으로 RGB 변환 후 인코딩                                                      |
| WebP       |  ✅  |  ✅  |                                                                                                     |
| AVIF       |  ✅  |  ✅  | 인코딩은 8-bit 로 고정 (라운드트립 호환성). 외부 도구로 만든 10-bit AVIF 입력은 `image` 0.24 한계로 미지원 |
| TIFF       |  ✅  |  ❌  | 입력만 지원                                                                                         |
| BMP        |  ✅  |  ❌  | 입력만 지원                                                                                         |
| ICO        |  ✅  |  ❌  | 입력만 지원                                                                                         |

## 📦 설치

### Docker 개발 환경 (추천)

로컬 OS 에 Rust / `nasm` / `dav1d` 를 직접 설치하지 않고, 컨테이너 안에서 빌드와 테스트를 실행할 수 있습니다. WSL, macOS, 새 MacBook 으로 옮겨도 같은 Debian 기반 환경을 사용합니다.

```bash
# 개발 이미지 빌드
docker compose build

# 전체 품질 검사 (포맷팅 + Clippy + 테스트)
./scripts/check.sh

# 테스트 실행
docker compose run --rm dev cargo test

# 포맷팅
docker compose run --rm dev cargo fmt --check

# 린트
docker compose run --rm dev cargo clippy --all-targets --all-features -- -D warnings

# 릴리즈 빌드
docker compose run --rm dev cargo build --release

# 대화형 모드 실행
docker compose run --rm dev cargo run --release

# 자동화용 명령줄 모드 실행
docker compose run --rm dev cargo run --release -- -i input.png -o output.webp -f webp

# 컨테이너 안에서 셸 열기
docker compose run --rm dev
```

호스트에 Rust 와 시스템 의존성을 직접 설치한 경우에는 `./scripts/check.sh --local` 로 같은 검사를 로컬 Cargo 로 실행할 수 있습니다.

`target` 과 Cargo registry/git 캐시는 Docker named volume 에 저장되어 호스트 프로젝트 디렉토리를 빌드 산출물로 어지럽히지 않습니다. 그래서 기본 흐름은 `cargo run` 도 컨테이너 안에서 실행하는 방식입니다. 완전히 새로 받고 싶으면 다음처럼 볼륨까지 지웁니다.

```bash
docker compose down -v
```

기본 Rust 이미지는 `rust:1-trixie` 입니다. `dav1d-sys` 가 `dav1d >= 1.3.0` 을 요구하므로 Debian bookworm 대신 trixie 를 사용합니다. 특정 버전으로 고정하고 싶으면 `.env` 파일이나 명령 앞 환경변수로 바꿀 수 있습니다.

```bash
RUST_IMAGE=rust:1.94-trixie docker compose build
```

### 로컬 설치

1. Rust가 설치되어 있어야 합니다. [Rust 설치 가이드](https://www.rust-lang.org/tools/install)를 참고하세요.
2. **시스템 라이브러리 설치** — AVIF 인코딩(`rav1e`)에는 `nasm`, AVIF 디코딩(`dav1d`)에는 `libdav1d` 가 필요합니다.

   ```bash
   # Ubuntu / WSL
   sudo apt install -y nasm libdav1d-dev

   # macOS
   brew install nasm dav1d
   ```

3. 이 프로젝트를 클론하거나 다운로드합니다:

   ```bash
   git clone <repository-url>
   cd image-converter
   ```

4. 프로젝트 디렉토리에서 빌드합니다:

   ```bash
   cargo build --release
   ```

## 🚀 사용법

### 기본 사용: 대화형 모드 🌟

```bash
./target/release/image_converter
```

명시적으로 대화형 모드를 지정할 수도 있습니다.

```bash
./target/release/image_converter -I
```

단계별로 안내를 따라 쉽게 이미지를 변환할 수 있습니다:

1. 변환 모드 선택 (단일 파일 / 디렉토리 일괄)
2. 입력 경로 입력 (모드에 따라 파일 또는 디렉토리)
3. (디렉토리 모드만) 하위 폴더 재귀 여부
4. 출력 형식 선택 (WebP/AVIF/PNG/JPEG)
5. 품질 선택 (최고/높음/보통/낮음/사용자 지정 — PNG 출력은 무손실이라 자동 스킵)
6. (디렉토리 모드만) 스레드 수 (1 이상, 비워두면 모든 코어 사용)
7. 출력 경로 확인

### 자동화용 명령줄 모드

반복 작업이나 스크립트에 넣어야 할 때만 `-i`, `-o`, `-f` 옵션을 사용합니다. 일반 사용은 대화형 모드를 권장합니다.

#### 단일 파일

```bash
# WebP로 변환 (품질 90%)
./target/release/image_converter -i input.png -o output.webp -f webp -q 90

# AVIF로 변환 (품질 80%)
./target/release/image_converter -i photo.jpg -o photo.avif -f avif -q 80

# 역변환: WebP → PNG (무손실, quality 무시됨)
./target/release/image_converter -i photo.webp -o photo.png -f png

# 역변환: AVIF → PNG (8-bit AVIF 만 지원)
./target/release/image_converter -i photo.avif -o photo.png -f png

# JPEG 로 변환 (알파 채널이 있으면 자동 RGB 변환)
./target/release/image_converter -i icon.png -o icon.jpg -f jpeg -q 85

# TIFF/BMP 입력도 그대로 지원
./target/release/image_converter -i scan.tiff -o scan.webp -f webp -q 85
```

#### 디렉토리 일괄 변환

입력 경로가 디렉토리이면 자동으로 일괄 변환 모드로 동작합니다. 변환은 **rayon** 을 통해 멀티코어로 병렬 처리됩니다 (큰 폴더에서 큰 속도 향상).

```bash
# photos/ 안의 모든 PNG/JPG/JPEG 를 WebP로 변환 (현재 폴더만)
./target/release/image_converter -i photos -o photos_webp -f webp -q 80

# 하위 폴더까지 재귀적으로 변환 (입력 디렉토리 구조 그대로 미러링)
./target/release/image_converter -i photos -o photos_webp -f webp -q 80 -r

# 사용할 스레드 수 제한 (CLI 플래그)
./target/release/image_converter -i photos -o photos_webp -f webp -r -t 4

# 환경변수로도 가능 (CLI 플래그가 우선)
RAYON_NUM_THREADS=4 ./target/release/image_converter -i photos -o photos_webp -f webp -r
```

## 📊 옵션

- `-I, --interactive`: 대화형 모드 명시 실행 (인자 없이 실행해도 대화형 모드로 시작)
- `-i, --input <PATH>`: 입력 이미지 파일 또는 디렉토리 경로
- `-o, --output <PATH>`: 출력 파일 또는 디렉토리 경로
- `-f, --format <FORMAT>`: 출력 형식 (`png`, `jpg`, `jpeg`, `webp`, `avif`)
- `-q, --quality <QUALITY>`: 변환 품질 1-100 (기본값: 90, **PNG 출력 시 무손실이라 무시됨**)
- `-r, --recursive`: 디렉토리 입력 시 하위 폴더까지 재귀 변환
- `-t, --threads <N>`: 디렉토리 모드에서 사용할 스레드 수 (1 이상, 미지정 시 `RAYON_NUM_THREADS` 또는 CPU 코어 수). 단일 파일 변환에는 영향 없음

## 💡 예제 출력

### 단일 파일 변환

```
🚀 이미지 변환을 시작합니다...
📁 원본: 5.32 MB (1920x1080 px)
[00:00:02] ████████████████████████████████ 100% 변환 완료!
💾 변환 후: 1.23 MB (품질: 90%)
🎉 용량 감소: 76.9% ↓
✨ 변환 완료: input.png → output.webp
```

### 디렉토리 일괄 변환

```
🚀 디렉토리 일괄 변환을 시작합니다...
  📂 입력: photos (재귀)
  📁 출력: photos_webp (WEBP)
[00:00:05] ████████████████████████████████ 12/12 last.png

📊 일괄 변환 결과:
  🗂️ 처리 대상: 12개
  ✅ 성공: 12개  ⏭️ 건너뜀: 0개  ❌ 실패: 0개
  📁 원본 합계: 38.21 MB
  💾 변환 합계: 8.74 MB (품질: 80%)
  🎉 평균 용량 감소: 77.1% ↓
```

## 🎯 Alias 설정

자주 사용하는 명령어를 alias로 설정하여 편리하게 사용할 수 있습니다. 예를 들어, zsh를 사용하는 경우 `~/.zshrc` 파일에 다음을 추가하세요:

```bash
alias imgconv='~/image_converter/target/release/image_converter'
# 명령줄 모드를 따로 쓰고 싶을 때만 옵션을 붙이면 됨
```

이후에는 다음과 같이 사용할 수 있습니다:

```bash
# 대화형 모드
imgconv

# 명령줄 모드
imgconv -i input.png -o output.webp -f webp -q 80
```

## 🛠️ 기술 스택

- **Rust**: 시스템 프로그래밍 언어
- **image**: 이미지 처리
- **webp**: WebP 인코딩
- **ravif**: AVIF 인코딩
- **clap**: CLI 인자 처리
- **dialoguer**: 대화형 인터페이스
- **colored**: 색상 출력
- **indicatif**: 진행률 표시
- **walkdir**: 디렉토리 재귀 순회
- **rayon**: 일괄 변환 멀티코어 병렬 처리
- **thiserror**: 명시적 커스텀 에러 타입

## 📈 성능

- WebP: 일반적으로 PNG 대비 25-35% 작은 크기
- AVIF: 일반적으로 JPEG 대비 50% 작은 크기 (더 나은 품질)
- 변환 속도: 이미지 크기와 선택한 품질에 따라 다름
