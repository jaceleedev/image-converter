# 🖼️ Image Converter

여러 이미지 포맷을 양방향으로 변환하는 고성능 Rust CLI 도구. WebP/AVIF 압축뿐 아니라 PNG/JPEG 로의 역변환도 지원합니다.

## ✨ 주요 기능

- **양방향 포맷 변환**: PNG/JPG/JPEG/WebP/TIFF/BMP/ICO → PNG/JPG/WebP/AVIF
- **단일 파일 + 디렉토리 일괄 변환**: 입력이 디렉토리이면 자동으로 일괄 모드 (rayon 으로 멀티코어 병렬 처리)
- **재귀 변환**: `--recursive` 옵션으로 하위 폴더까지 한 번에 변환 (구조 그대로 미러링)
- **대화형 모드**: 단일/디렉토리 모드를 단계별로 안내
- **용량 비교**: 변환 전후 파일 크기 및 감소율 표시 (배치 모드는 합계까지)
- **진행 상황 표시**: 실시간 진행률 표시
- **품질 설정**: 1-100% 품질 조정 가능 (PNG 는 무손실이라 자동 무시)
- **아름다운 UI**: 이모티콘과 색상으로 보기 좋은 출력

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
   cd image_converter
   ```

4. 프로젝트 디렉토리에서 빌드합니다:

   ```bash
   cargo build --release
   ```

## 🚀 사용법

### 대화형 모드 (추천) 🌟

```bash
./target/release/image_converter -I
```

또는

```bash
./target/release/image_converter --interactive
```

단계별로 안내를 따라 쉽게 이미지를 변환할 수 있습니다:

1. 변환할 이미지 파일 경로 입력
2. 출력 형식 선택 (WebP/AVIF)
3. 품질 선택 (최고/높음/보통/낮음/사용자 지정)
4. 출력 파일 경로 확인

### 명령줄 모드 - 단일 파일

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

### 명령줄 모드 - 디렉토리 일괄 변환

입력 경로가 디렉토리이면 자동으로 일괄 변환 모드로 동작합니다. 변환은 **rayon** 을 통해 멀티코어로 병렬 처리됩니다 (큰 폴더에서 큰 속도 향상).

```bash
# photos/ 안의 모든 PNG/JPG/JPEG 를 WebP로 변환 (현재 폴더만)
./target/release/image_converter -i photos -o photos_webp -f webp -q 80

# 하위 폴더까지 재귀적으로 변환 (입력 디렉토리 구조 그대로 미러링)
./target/release/image_converter -i photos -o photos_webp -f webp -q 80 -r

# 사용할 스레드 수 제한 (기본: CPU 코어 수)
RAYON_NUM_THREADS=4 ./target/release/image_converter -i photos -o photos_webp -f webp -r
```

## 📊 옵션

- `-I, --interactive`: 대화형 모드 실행
- `-i, --input <PATH>`: 입력 이미지 파일 또는 디렉토리 경로
- `-o, --output <PATH>`: 출력 파일 또는 디렉토리 경로
- `-f, --format <FORMAT>`: 출력 형식 (`png`, `jpg`, `jpeg`, `webp`, `avif`)
- `-q, --quality <QUALITY>`: 변환 품질 1-100 (기본값: 90, **PNG 출력 시 무손실이라 무시됨**)
- `-r, --recursive`: 디렉토리 입력 시 하위 폴더까지 재귀 변환

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
  ✅ 성공: 12개  ❌ 실패: 0개
  📁 원본 합계: 38.21 MB
  💾 변환 합계: 8.74 MB (품질: 80%)
  🎉 평균 용량 감소: 77.1% ↓
```

## 🎯 Alias 설정

자주 사용하는 명령어를 alias로 설정하여 편리하게 사용할 수 있습니다. 예를 들어, zsh를 사용하는 경우 `~/.zshrc` 파일에 다음을 추가하세요:

```bash
alias imgconv='~/image_converter/target/release/image_converter'
# 대화형 모드로 바로 실행
alias imgconvi='~/image_converter/target/release/image_converter -I'
```

이후에는 다음과 같이 사용할 수 있습니다:

```bash
# 일반 모드
imgconv -i input.png -o output.webp -f webp -q 80

# 대화형 모드
imgconvi
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
