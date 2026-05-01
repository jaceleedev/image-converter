# 🧪 테스트 가이드

## 테스트 개요

이미지 변환기의 안정성을 보장하기 위한 테스트 코드가 포함되어 있습니다.

## 테스트 구조

```
src/
  main.rs         # CLI 인자 파서 단위 테스트 (#[cfg(test)] mod tests)
  interactive.rs  # 대화형 모드 검증/경로 빌더 단위 테스트 (#[cfg(test)] mod tests)
  lib.rs          # 테스트용 함수들
  tests/
    mod.rs        # 테스트 모듈 선언
    test_utils.rs # 테스트 헬퍼 함수 및 매크로
    unit_tests.rs # 단위 테스트
    integration_tests.rs # 통합 테스트
```

## 테스트 실행 방법

### Docker 로 실행 (추천)
```bash
docker compose build
./scripts/check.sh
```

`scripts/check.sh` 는 Docker 개발 컨테이너에서 다음 검사를 순서대로 실행합니다.

```bash
docker compose run --rm dev cargo fmt --check
docker compose run --rm dev cargo clippy --all-targets --all-features -- -D warnings
docker compose run --rm dev cargo test
```

테스트 출력까지 보고 싶으면 다음처럼 실행합니다.

```bash
docker compose run --rm dev cargo test -- --nocapture
```

릴리즈 모드 테스트도 컨테이너 안에서 실행할 수 있습니다.

```bash
docker compose run --rm dev cargo test --release
```

컨테이너의 `target` 과 Cargo 캐시는 Docker named volume 에 저장됩니다. 캐시까지 초기화하려면 `docker compose down -v` 를 실행합니다.

### 로컬 Rust 로 실행

호스트에 Rust 와 시스템 의존성(`nasm`, `dav1d`, `pkg-config`) 을 직접 설치한 경우에는 같은 검사를 로컬 Cargo 로 실행할 수 있습니다.

```bash
./scripts/check.sh --local
```

#### 기본 테스트 실행
```bash
cargo test
```

#### 테스트 출력과 함께 실행 (추천)
```bash
cargo test -- --nocapture
```

#### 순차적으로 테스트 실행 (출력이 깔끔하게 나옴)
```bash
cargo test -- --test-threads=1 --nocapture
```

#### 특정 테스트만 실행
```bash
# 파일 크기 관련 테스트만 실행
cargo test format_file_size

# WebP 관련 테스트만 실행
cargo test webp -- --nocapture
```

#### 빠른 테스트 (릴리즈 모드)
```bash
cargo test --release
```

## 테스트 출력 예시

```
🧪 파일 크기 포맷팅 함수 테스트
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  → 다양한 크기를 사람이 읽기 쉬운 형식으로 변환하는지 확인
  ✓ 0 바이트 처리 완료
  ✓ 512 바이트 → 512.00 B
  ✓ 1024 바이트 → 1.00 KB
  ✓ 1536 바이트 → 1.50 KB
  ✓ 1048576 바이트 → 1.00 MB
  ✓ 1073741824 바이트 → 1.00 GB
```

## 테스트 구성

### 📏 단위 테스트 (Unit Tests)

1. **`test_format_file_size`**
   - 파일 크기를 사람이 읽기 쉬운 형식으로 변환하는 함수 테스트
   - 예: 1024 bytes → "1.00 KB"
   - 0B, 512B, 1KB, 1.5KB, 1MB, 1GB 테스트

2. **`test_format_file_size_edge_cases`**
   - 파일 크기 변환 함수의 경계값 테스트
   - 단위가 바뀌는 경계에서 올바르게 동작하는지 확인
   - 1023B (1KB 직전), 1MB 직전, 1GB 직전 테스트

3. **`test_format_quality_label_*`** (2개)
   - 손실 포맷(WebP/JPEG/AVIF)은 품질 값을 표시하는지 확인
   - PNG 는 품질 값 대신 "무손실" 로 표시하는지 확인

4. **`test_output_format_extension_matching`**
   - 출력 포맷별 허용 확장자 매칭 규칙 확인
   - JPEG 포맷은 `.jpg` 와 `.jpeg` 를 모두 허용하고, 대소문자를 무시하는지 검증

### 🔄 통합 테스트 (Integration Tests)

4. **`test_webp_conversion`**
   - PNG → WebP 변환 기능 전체 테스트
   - 변환이 정상적으로 작동하고, 파일 크기가 감소하는지 확인

5. **`test_avif_conversion`**
   - PNG → AVIF 변환 기능 전체 테스트
   - 변환이 정상적으로 작동하고, 파일 크기가 감소하는지 확인

6. **`test_quality_parameter_bounds`**
   - 품질 파라미터 경계값 테스트
   - 최소값(1)과 최대값(100)에서 정상 작동하는지 확인

### 📂 디렉토리 일괄 변환 테스트

7. **`test_batch_directory_conversion`**
   - 디렉토리 안의 PNG 3개를 일괄 변환
   - 모든 파일이 성공적으로 변환되고 출력 디렉토리에 생성되는지 확인

8. **`test_batch_skips_existing_output_files`**
   - 이미 존재하는 출력 파일은 덮어쓰지 않고 건너뛰는지 확인
   - `BatchSummary.skipped` 카운트와 기존 파일 내용 보존을 검증

9. **`test_batch_skips_non_image_files`**
   - `.txt`, `.md` 같은 비이미지 파일은 자동으로 스킵되는지 확인

10. **`test_batch_recursive_conversion`**
   - 재귀 옵션 사용 시 하위 디렉토리도 처리되고, 출력에 구조가 미러링되는지 확인

11. **`test_batch_non_recursive_skips_subdirs`**
   - 재귀 옵션 없이 실행 시 하위 디렉토리는 무시되는지 확인

12. **`test_batch_empty_directory`**
    - 이미지가 없는 디렉토리에서도 에러 없이 0개 처리로 끝나는지 확인

### ❌ 에러 처리 테스트

13. **`test_invalid_format`**
    - 지원하지 않는 형식으로 변환 시도 시 에러 처리 테스트
    - 올바른 에러 메시지가 반환되는지 확인

14. **`test_nonexistent_input_file`**
    - 존재하지 않는 입력 파일 처리 테스트
    - 파일이 없을 때 적절한 에러가 발생하는지 확인

15. **`test_single_conversion_rejects_existing_output`**
    - 단일 변환에서 출력 파일이 이미 있으면 `OutputExists` 에러를 반환하는지 확인
    - 기존 출력 파일 내용이 보존되는지 검증

16. **`test_single_conversion_rejects_mismatched_output_extension`**
    - 단일 변환에서 선택 포맷과 출력 파일 확장자가 다르면 `OutputExtensionMismatch` 에러를 반환하는지 확인
    - 확장자가 맞지 않을 때 출력 파일을 만들지 않는지 검증

### 🔁 다중 포맷 입출력 테스트

16. **`test_png_output_from_webp_input`**
    - WebP → PNG 역변환이 동작하는지 확인 (`encode_to` 의 PNG 분기 + WebP 디코딩)
    - 출력 파일이 PNG 매직 바이트(`89 50 4E 47`) 로 시작하는지 검증

17. **`test_jpeg_output_from_png_input`**
    - PNG → JPEG 변환이 동작하는지 확인
    - 출력 파일이 JPEG 매직 바이트(`FF D8 FF`) 로 시작하는지 검증

18. **`test_jpg_alias_for_jpeg`**
    - 포맷 인자 `"jpg"` 와 `"jpeg"` 가 같은 JPEG 인코딩 분기로 매핑되는지 확인

19. **`test_tiff_input_to_webp`**
    - TIFF 입력이 디코딩되어 WebP 로 변환되는지 확인 (`is_supported_image` 화이트리스트 + `image::open` TIFF 디코더)

20. **`test_bmp_input_to_jpeg`**
    - BMP 입력이 디코딩되어 JPEG 로 변환되는지 확인

21. **`test_batch_mixed_input_formats`**
    - PNG + WebP + AVIF + TIFF + BMP 5종이 한 디렉토리에 섞여 있을 때 모두 PNG 로 일괄 변환되는지 확인 (배치 모드의 입력 화이트리스트 + 다중 디코더 결합 검증)

22. **`test_avif_input_to_png`**
    - AVIF → PNG 라운드트립이 동작하는지 확인 (`avif-decoder` feature + `dav1d` 디코딩, 8-bit AVIF 인코딩)
    - 출력 파일이 PNG 매직 바이트로 시작하는지 검증

23. **`test_batch_with_explicit_threads`**
    - `convert_directory()` 의 `threads: Option<usize>` 인자 검증
    - `None` (기본) 과 `Some(1)` (단일 스레드) 두 모드에서 같은 입력에 대해 같은 성공 개수가 나오는지 확인 (스레드 수에 결과가 영향받지 않아야 함)

24. **`test_jpeg_input_to_webp`**
    - JPEG 입력을 WebP 로 변환하는 명시적 단일 케이스
    - 출력 파일이 RIFF/WEBP 시그니처로 시작하는지 검증

25. **`test_jpeg_input_to_png`**
    - JPEG 입력을 PNG 로 변환 (JPEG 디코더 + PNG 인코더 결합)
    - 출력이 PNG 매직 바이트(`89 50 4E 47`) 로 시작하는지 검증

26. **`test_jpg_extension_input`**
    - 입력 측 `.jpg` 확장자도 JPEG 디코더로 정상 인식되는지 확인 (`.jpg`/`.jpeg` 양쪽 별칭 회귀 방지)

### 🛠️ CLI 인자 파서 단위 테스트 (`src/main.rs`)

27. **`parse_quality_accepts_valid_range`**
    - `--quality` 가 1.0 / 50.5 / 100.0 같은 정상 범위 값을 통과시키는지 확인

28. **`parse_quality_rejects_out_of_range`**
    - 0, 0.99, 100.01, -10, 200 같은 범위 외 값이 거부되는지 확인

29. **`parse_quality_rejects_non_numeric`**
    - `"abc"` 같은 비숫자 입력이 한국어 에러 메시지("유효한 숫자가 아닙니다") 와 함께 거부되는지 확인

30. **`parse_threads_accepts_positive`**
    - `--threads` 가 1, 16 같은 양의 정수를 통과시키는지 확인

31. **`parse_threads_rejects_zero`**
    - 0 이 거부되고 한국어 에러 메시지("1 이상") 가 포함되는지 확인 (rayon 풀 빌드 단계 panic 방지)

32. **`parse_threads_rejects_non_numeric`**
    - 비숫자(`"abc"`) 와 음수(`"-1"`) 입력 거부 확인

33. **`parse_format_accepts_valid_values_case_insensitive`**
    - `--format WEBP` 처럼 대문자로 입력해도 `OutputFormat::Webp` 로 파싱되는지 확인

34. **`parse_format_rejects_invalid_value`**
    - `--format xyz` 같은 미지원 출력 포맷을 clap 단계에서 거부하는지 확인

35. **대화형 기본 실행 / 비대화형 필수 인자 테스트** (4개)
    - 인자 없이 실행하는 파싱 결과가 대화형 기본 실행으로 분기되는지 확인
    - `-I` 플래그가 입력/출력/포맷 없이도 대화형 모드로 분기되는지 확인
    - 비대화형 모드에서 `-i`/`-o`/`-f` 누락 목록을 올바르게 계산하는지 확인
    - `-i`/`-o`/`-f` 가 모두 있으면 누락 인자가 없는지 확인

### 🎯 대화형 모드 검증 단위 테스트 (`src/interactive.rs`)

36. **`validate_input_path_*`** (4개)
    - 존재하지 않는 경로 거부, 단일 모드에 디렉토리 입력 거부, 배치 모드에 파일 입력 거부, 정상 케이스(파일/디렉토리) 통과

37. **`validate_quality_input_*`** (2개)
    - 1.0/50.5/100.0 정상 범위 통과, 0 / 0.99 / 100.01 / -10 / `"abc"` 거부

38. **`validate_threads_input_*`** (2개)
    - 1, 16 통과, 0 / -1 / `"abc"` / 빈 입력 거부

39. **`validate_output_file_path_*`** (3개)
    - 선택 포맷과 출력 파일 확장자가 일치하는지 검증
    - JPEG 는 `.jpg`/`.jpeg` 별칭을 모두 허용하고, 불일치/확장자 없음은 거부

40. **`default_output_path_for_file_*`** (2개)
    - `{stem}_converted.{format}` 패턴 (예: `photo.png` + webp → `photo_converted.webp`), 확장자 없는 입력 (`no_ext` + png → `no_ext_converted.png`) 처리

41. **`default_output_path_for_dir_*`** (2개)
    - `{dirname}_converted_{format}` 패턴 (예: `photos` + webp → `photos_converted_webp`), trailing slash (`/tmp/photos/`) 정상 처리

## 테스트 매크로

### `test_description!`
테스트의 목적을 설명하는 제목을 출력합니다.
```rust
test_description!("파일 크기 포맷팅 함수 테스트");
```

### `test_step!`
테스트의 각 단계를 표시합니다.
```rust
test_step!("100x100 테스트 이미지 생성 중...");
```

### `test_success!`
테스트 단계의 성공을 표시합니다.
```rust
test_success!("변환 완료");
```

## 테스트 유틸리티

### `create_test_image` 함수
- 테스트용 체커보드 패턴 이미지를 생성
- 흑백 체크 패턴으로 압축 테스트에 적합
- 지정한 크기의 PNG 이미지 생성

### `convert_image_silent` 함수
- 테스트용 이미지 변환 함수 (진행률 표시 없음)
- 테스트에서 출력이 섞이지 않도록 조용히 변환

## 테스트 환경

- **tempfile**: 임시 디렉토리에서 안전하게 테스트 실행
- 테스트 후 자동으로 임시 파일 정리
- 각 테스트는 독립적으로 실행됨

## 새로운 테스트 추가하기

1. 적절한 파일에 테스트 추가:
   - 단위 테스트: `src/tests/unit_tests.rs`
   - 통합 테스트: `src/tests/integration_tests.rs`

2. 테스트 매크로 사용:
```rust
#[test]
fn test_new_feature() {
    test_description!("새 기능 테스트");
    test_step!("테스트 준비 중...");
    
    // 테스트 코드
    
    test_success!("테스트 완료");
}
```

## 테스트 커버리지

현재 테스트는 다음 영역을 커버합니다 (총 56개):
- ✅ 파일 크기 포맷팅
- ✅ 출력 요약 라벨 (PNG 무손실 / 손실 포맷 품질 표시)
- ✅ 출력 포맷별 허용 확장자 매칭
- ✅ WebP / AVIF 단일 변환
- ✅ 품질 파라미터 검증
- ✅ 디렉토리 일괄 변환 (재귀 / 비재귀)
- ✅ 출력 덮어쓰기 방지 (단일 변환 에러 / 일괄 변환 건너뜀)
- ✅ 출력 확장자 불일치 방지 (단일 변환 에러 / 대화형 입력 검증)
- ✅ 비이미지 파일 자동 스킵
- ✅ 빈 디렉토리 처리
- ✅ 에러 처리 (지원하지 않는 형식, 존재하지 않는 파일, 기존 출력 파일)
- ✅ PNG / JPEG 출력 (WebP → PNG 역변환, AVIF → PNG 역변환, PNG → JPEG, jpg 별칭)
- ✅ TIFF / BMP / AVIF 입력 디코딩
- ✅ 혼합 입력 포맷 일괄 변환 (PNG + WebP + AVIF + TIFF + BMP → PNG)
- ✅ 명시적 스레드 수 옵션 (`threads=None` vs `threads=Some(1)` 결과 일관성)
- ✅ JPG/JPEG 단일 입력 (jpeg→webp, jpeg→png, .jpg 확장자 별칭)
- ✅ CLI 인자 파서 (`--quality` 1.0~100.0 범위, `--threads` ≥ 1 검증, 출력 포맷 허용값 검증, 대화형 기본 실행, 비대화형 필수 인자 검증)
- ✅ 대화형 모드 검증 클로저 + 디폴트 출력 경로 빌더 (순수 함수로 분리하여 단위 테스트)

향후 추가할 수 있는 테스트:
- 10-bit AVIF 입력 디코딩 (`image` 0.25 업그레이드 후)
- 일괄 변환 중 일부 파일이 손상되어 실패할 때의 동작
- 대용량 이미지 처리
- 메모리 사용량 테스트
- 대화형 모드 통합 테스트 (`dialoguer::Select` 는 PTY 필요 — `rexpect` 등 도입 필요)
