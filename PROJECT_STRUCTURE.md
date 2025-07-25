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
    ├── main.rs             # 진입점 - CLI 인자 처리
    ├── lib.rs              # 라이브러리 루트 - 테스트용 함수
    ├── converter.rs        # 핵심 변환 로직
    ├── interactive.rs      # 대화형 모드
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
- 에러 처리 및 종료

### `converter.rs` (핵심 비즈니스 로직)

- 이미지 변환 함수
- 진행률 표시
- 결과 출력

### `interactive.rs` (사용자 인터페이스)

- 대화형 모드 구현
- 사용자 입력 처리
- 메뉴 시스템

### `utils.rs` (공통 유틸리티)

- 파일 크기 포맷팅
- 기타 헬퍼 함수

### `lib.rs` (라이브러리 인터페이스)

- 공개 API 정의
- 테스트용 변환 함수
- 모듈 re-export

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
  ├── converter.rs (변환 기능)
  │     └── utils.rs (헬퍼 함수)
  └── interactive.rs (대화형 모드)
        └── converter.rs (변환 실행)

lib.rs
  ├── converter.rs (re-export)
  ├── utils.rs (re-export)
  └── tests/ (테스트에서만 사용)
```

## 향후 개선 제안

1. **에러 타입 정의**: `Result<T, Box<dyn Error>>`를 커스텀 에러 타입으로
2. **설정 모듈**: 품질 프리셋, 기본값 등을 관리하는 `config.rs`
3. **다국어 지원**: 메시지를 별도 파일로 분리
4. **플러그인 시스템**: 새로운 이미지 형식 지원을 쉽게 추가

이 구조는 현재 프로젝트 규모에 적합하며, 향후 확장에도 대응할 수 있습니다.
