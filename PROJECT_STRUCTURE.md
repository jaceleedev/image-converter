# 📁 프로젝트 구조

## 현재 구조

```
image_converter/
├── .gitignore              # Git 무시 파일 설정
├── Cargo.toml              # Rust 프로젝트 설정 및 의존성
├── README.md               # 프로젝트 사용 가이드
├── TESTING.md              # 테스트 실행 가이드
├── PROJECT_STRUCTURE.md    # 이 문서
└── src/
    ├── main.rs             # 진입점 - CLI 인자 처리, 단일/일괄 분기
    ├── lib.rs              # 라이브러리 루트 - 공개 API re-export
    ├── error.rs            # ConverterError + Result 타입 (thiserror 기반)
    ├── converter.rs        # 단일 파일 변환 + 인코딩 헬퍼
    ├── batch.rs            # 디렉토리 일괄 변환 (재귀 옵션)
    ├── interactive.rs      # 대화형 모드 (단일/디렉토리)
    ├── utils.rs            # 유틸리티 함수
    └── tests/              # 테스트 모듈
        ├── mod.rs          # 테스트 모듈 선언
        ├── test_utils.rs   # 테스트 헬퍼 함수
        ├── unit_tests.rs   # 단위 테스트
        └── integration_tests.rs # 통합 테스트
```

## 모듈별 책임

### `main.rs` (진입점)

- CLI 인자 파싱 (clap 사용)
- 대화형/명령줄 모드 분기
- 입력 경로 타입(파일/디렉토리)에 따라 단일/일괄 변환 분기
- 에러 처리 및 종료 — `ConverterError` 의 `Display` 로 메시지 출력

### `error.rs` (에러 타입)

- `ConverterError` enum + `Result<T>` 별칭
- thiserror 기반, 외부 크레이트 에러(io, image, dialoguer, ravif, ParseFloat) 는 `#[from]` 으로 자동 변환
- WebP 인코더의 `&str` 에러는 별도 `Webp(String)` variant 로 매핑 (소유권 확보)
- 사용자 입력성 에러는 `UnsupportedFormat`, `InvalidPath` 등 컨텍스트가 담긴 variant
- Display 메시지는 모두 한국어 (`"입출력 오류: ..."`, `"지원하지 않는 포맷입니다: xyz"` 등)

### `converter.rs` (단일 변환 비즈니스 로직)

- `encode_to`: 메모리 이미지를 WebP/AVIF/PNG/JPG/JPEG 바이트로 인코딩 (내부 헬퍼)
  - WebP: `webp::Encoder::from_image` + `encode(quality)`
  - AVIF: `ravif::Encoder` (rav1e 기반, RGBA 추출 후 인코딩) — `with_bit_depth(BitDepth::Eight)` 로 **8-bit 강제** (image 0.24 디코더와 호환성 유지)
  - PNG: `image` 크레이트의 `write_to(&mut Cursor, ImageOutputFormat::Png)` — 무손실, quality 무시
  - JPG/JPEG: 알파 채널을 가질 수 없어 `to_rgb8()` 으로 다운샘플 후 `ImageOutputFormat::Jpeg(quality_u8)`
- `convert_image`: 진행률 + 결과 출력 포함
- `convert_image_silent`: 출력 없는 변환 (`ConvertStats` 반환). 배치 모드와 테스트에서 사용
- 두 함수 모두 동일한 내부 인코딩 헬퍼를 호출 (코드 중복 제거)

### `batch.rs` (디렉토리 일괄 변환)

- `convert_directory`: 입력 디렉토리에서 지원 포맷만 골라 **rayon 으로 병렬** 변환
  - 입력 화이트리스트: `png`/`jpg`/`jpeg`/`webp`/`avif`/`tiff`/`tif`/`bmp`/`ico`
- 각 파일은 `process_one` 헬퍼가 처리하고 `Option<ConvertStats>` 를 반환 — 한 파일이 실패해도 나머지는 그대로 진행
- 결과는 직렬 합산하여 `BatchSummary` 통계로 반환
- 진행률 바 (`indicatif::ProgressBar`) 와 `pb.println` 은 thread-safe (내부 Mutex)
- 재귀 모드에서 입력 디렉토리 구조를 출력에 미러링
- 스레드 수는 `RAYON_NUM_THREADS` 환경변수로 조절 (기본: CPU 코어 수)

### `interactive.rs` (사용자 인터페이스)

- 대화형 모드 구현
- 단일 파일 / 디렉토리 모드 선택
- 디렉토리 모드에서 재귀 옵션 질문
- 출력 포맷 선택지: WebP / AVIF / PNG / JPEG
- PNG 출력 선택 시 quality 단계는 자동으로 스킵 (무손실 포맷이라 의미 없음)
- 단계별 사용자 입력 처리

### `utils.rs` (공통 유틸리티)

- 파일 크기 포맷팅
- 기타 헬퍼 함수

### `lib.rs` (라이브러리 인터페이스)

- 공개 API re-export (`convert_image`, `convert_image_silent`, `convert_directory`, ...)
- `main.rs`도 이 라이브러리를 통해 변환 함수에 접근하여 코드 중복 컴파일을 피함

### `tests/` (테스트 코드)

- 단위 테스트와 통합 테스트 분리
- 테스트 유틸리티 및 매크로
- 깔끔한 테스트 출력

## 장점

1. **명확한 책임 분리**: 각 모듈이 단일 책임을 가짐
2. **테스트 용이성**: 비즈니스 로직이 분리되어 테스트하기 쉬움
3. **유지보수성**: 기능별로 파일이 분리되어 찾기 쉬움
4. **확장성**: 새 기능 추가 시 적절한 모듈에 추가하면 됨
5. **재사용성**: lib.rs를 통해 다른 프로젝트에서도 사용 가능

## 의존성 흐름

```
main.rs
  └── image_converter (lib)
        ├── converter.rs (단일 변환)
        │     └── utils.rs
        ├── batch.rs (일괄 변환)
        │     ├── converter.rs (단일 변환을 루프 호출)
        │     └── utils.rs
        ├── interactive.rs (대화형 모드)
        │     ├── converter.rs
        │     └── batch.rs
        └── tests/ (테스트에서만 사용)
```

## 향후 개선 제안

1. **`--threads` CLI 옵션**: 환경변수 외에 명시적 플래그
2. **`image` 0.25 업그레이드**: 10-bit AVIF 디코딩 지원. breaking change 가 있어 별도 작업
3. **HEIC 입력**: iPhone 사진 변환용. `libheif` 시스템 의존성 + 외부 크레이트
4. **설정 모듈**: 품질 프리셋, 기본값 등을 관리하는 `config.rs`
5. **다국어 지원**: 메시지를 별도 파일로 분리

이 구조는 현재 프로젝트 규모에 적합하며, 향후 확장에도 대응할 수 있습니다.
