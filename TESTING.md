# 🧪 테스트 가이드

## 테스트 개요

이미지 변환기의 안정성을 보장하기 위한 테스트 코드가 포함되어 있습니다.

## 테스트 구조

```
src/
  lib.rs          # 테스트용 함수들
  tests/
    mod.rs        # 테스트 모듈 선언
    test_utils.rs # 테스트 헬퍼 함수 및 매크로
    unit_tests.rs # 단위 테스트
    integration_tests.rs # 통합 테스트
```

## 테스트 실행 방법

### 기본 테스트 실행
```bash
cargo test
```

### 테스트 출력과 함께 실행 (추천)
```bash
cargo test -- --nocapture
```

### 순차적으로 테스트 실행 (출력이 깔끔하게 나옴)
```bash
cargo test -- --test-threads=1 --nocapture
```

### 특정 테스트만 실행
```bash
# 파일 크기 관련 테스트만 실행
cargo test format_file_size

# WebP 관련 테스트만 실행
cargo test webp -- --nocapture
```

### 빠른 테스트 (릴리즈 모드)
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

### 🔄 통합 테스트 (Integration Tests)

3. **`test_webp_conversion`**
   - PNG → WebP 변환 기능 전체 테스트
   - 변환이 정상적으로 작동하고, 파일 크기가 감소하는지 확인

4. **`test_avif_conversion`**
   - PNG → AVIF 변환 기능 전체 테스트
   - 변환이 정상적으로 작동하고, 파일 크기가 감소하는지 확인

5. **`test_quality_parameter_bounds`**
   - 품질 파라미터 경계값 테스트
   - 최소값(1)과 최대값(100)에서 정상 작동하는지 확인

### ❌ 에러 처리 테스트

6. **`test_invalid_format`**
   - 지원하지 않는 형식으로 변환 시도 시 에러 처리 테스트
   - 올바른 에러 메시지가 반환되는지 확인

7. **`test_nonexistent_input_file`**
   - 존재하지 않는 입력 파일 처리 테스트
   - 파일이 없을 때 적절한 에러가 발생하는지 확인

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

현재 테스트는 다음 영역을 커버합니다:
- ✅ 파일 크기 포맷팅
- ✅ WebP 변환
- ✅ AVIF 변환
- ✅ 품질 파라미터 검증
- ✅ 에러 처리

향후 추가할 수 있는 테스트:
- 다양한 이미지 형식 입력 (JPG, JPEG)
- 대용량 이미지 처리
- 동시 변환 처리
- 메모리 사용량 테스트
- 대화형 모드 테스트