# 웹 서비스 확장 계획

현재 Rust CLI 는 유지하면서, 같은 변환 코어를 웹 서비스에서도 재사용하기 위한 설계 메모입니다. 목표는 처음부터 큰 SaaS 를 만드는 것이 아니라, 실제로 쓸 수 있는 고품질 단일 이미지 변환 웹 도구를 먼저 만들고 이후 로그인, 사용량 제한, 결제, 배치 변환으로 확장할 수 있는 길을 열어두는 것입니다.

## 기본 판단

- CLI 는 계속 유지한다.
- 변환 로직은 Rust 라이브러리 API 를 중심으로 재사용한다.
- 웹 UI 는 Next.js + TypeScript 로 만든다.
- 이미지 변환 처리는 Next.js 서버가 아니라 Rust API 서버가 맡는다.
- 초기 트래픽은 적을 가능성이 높으므로, Rust API 는 사용하지 않을 때 0 인스턴스까지 줄어드는 Cloud Run 배포를 우선 검토한다.
- UI 시스템은 하나로 통일한다. 현재 웹 앱은 Tailwind CSS v4 + HeroUI v3 를 기본 UI 시스템으로 사용한다.

## 목표 아키텍처

```text
image-converter/
├── apps/
│   └── web/              # Next.js + TypeScript + Tailwind CSS v4 + HeroUI v3
├── src/
│   ├── main.rs           # 기존 CLI 진입점 유지
│   ├── lib.rs            # 변환 코어 공개 API
│   └── ...
└── src/bin/
    └── server.rs         # Rust HTTP API 서버 후보
```

초기에는 큰 workspace 분리보다 `src/bin/server.rs` 를 추가하는 방식이 작고 안전합니다. 웹 API 의 책임이 커지면 그때 `crates/core`, `crates/cli`, `crates/api` 또는 `apps/api` 형태로 나눕니다.

## Next.js 와 Cloud Run 역할 분리

Next.js 는 사용자가 보는 화면과 프론트엔드 상태를 담당합니다. Cloud Run 은 Google Cloud Platform 안에서 Docker 컨테이너를 실행하는 관리형 서비스이고, 여기에는 Rust 변환 API 서버를 올립니다.

```text
사용자 브라우저
  ├─ GET https://example.com
  │    └─ Next.js 가 변환 화면 제공
  │
  └─ POST https://api.example.com/v1/convert
       └─ Cloud Run 의 Rust API 가 이미지 변환 후 결과 반환
```

중요한 점은 이미지 파일을 Vercel/Next.js 서버 함수로 프록시하지 않는 것입니다. Vercel Functions 는 request/response payload 제한이 작아서 이미지 변환 주 경로로 쓰기 어렵습니다. 따라서 브라우저가 Rust API 로 직접 업로드하고, Next.js 는 파일을 직접 처리하지 않습니다.

나중에 로그인과 결제가 들어가도 흐름은 비슷합니다.

```text
1. 사용자가 Next.js 앱에서 로그인
2. Next.js 또는 인증 제공자가 access token 발급
3. 브라우저가 이미지와 token 을 Rust API 로 전송
4. Rust API 가 token/요금제/사용량 제한을 확인
5. 변환 실행 후 결과 반환
```

초기 MVP 에서는 인증 없이 공개 변환 API 로 시작하되, 파일 크기와 동시성 제한을 반드시 둡니다.

## API 서버 설계

Rust API 서버는 `axum` 기반을 우선 검토합니다.

초기 엔드포인트:

- `GET /healthz`: 배포와 모니터링용 상태 확인
- `POST /v1/convert`: multipart 업로드 + 변환 옵션 처리

`POST /v1/convert` 입력:

- 파일 1개
- 출력 포맷: `png`, `jpg`, `jpeg`, `webp`, `avif`
- 품질: 1-100
- 최대 가로 크기: 선택
- JPEG 배경색: JPG/JPEG 출력일 때만 선택

처리 흐름:

```text
multipart 요청 수신
  -> 업로드 크기 제한 확인
  -> 임시 디렉토리에 원본 저장
  -> 확장자/디코딩 가능 여부 확인
  -> 기존 convert_image_silent_with_conversion_options 호출
  -> 결과 파일 stream/download 응답
  -> 임시 파일 삭제
```

성능과 안정성을 위해 이미지 변환은 async request task 안에서 직접 오래 붙잡지 않고 `spawn_blocking` 또는 제한된 worker pool 로 넘깁니다. AVIF/HEIC 변환은 CPU 와 메모리를 많이 쓸 수 있으므로 `Semaphore` 로 동시 변환 수를 제한합니다.

## UI 시스템 결정

여러 UI 라이브러리를 동시에 설치해서 섞지 않습니다. Tailwind 기반이라도 각 라이브러리의 theme token, spacing, radius, animation, focus style 이 달라지면 제품이 금방 흐트러집니다.

현재 기준 권장안:

- Tailwind CSS
- HeroUI v3 (`@heroui/react`, `@heroui/styles`)
- lucide-react

초기에는 shadcn/ui 와 Aceternity UI 계열 로컬 컴포넌트를 검토했지만, 제품 UI 기준은 HeroUI v3 로 전환합니다. HeroUI v3 는 Tailwind CSS v4 와 React 19 조건에 맞고, 기본 접근성/상태 처리/컴포넌트 품질이 갖춰져 있어 변환 작업대 UI 를 빠르게 안정화하기 좋습니다.

HeroUI v3 로 전환하는 이유:

- Next.js, TypeScript, Tailwind 와 궁합이 좋다.
- React Aria 기반 컴포넌트로 접근성과 키보드 상호작용을 기본 확보하기 좋다.
- 무료로 사용할 수 있는 공개 컴포넌트 라이브러리이며, Pro 템플릿 영역에 의존하지 않아도 MVP 를 구성할 수 있다.
- HeroUI v3 는 Provider 없이 `@import "@heroui/styles";` 와 compound component 패턴으로 쓸 수 있어 설정이 단순하다.

다른 UI 후보의 사용 원칙:

- daisyUI: 빠른 프로토타입 후보로만 둔다.
- shadcn/ui: 현재 웹 앱에서는 제거된 상태이며, 제품 UI 기준으로 삼지 않는다.
- Aceternity UI / Magic UI / TailAdmin: 기본 UI 시스템이 아니라 참고용 또는 향후 별도 검토 후보로 둔다.
- TailAdmin: 공개 변환 도구 화면에는 쓰지 않는다. 나중에 관리자/사용량/결제 대시보드가 필요할 때 별도 검토한다.

## 초기 화면 UX

첫 화면은 랜딩 페이지가 아니라 실제 변환 도구여야 합니다.

핵심 화면 구성:

- 드래그 앤 드롭 업로드 영역
- 원본 정보: 파일명, 포맷, 용량, 크기
- 출력 포맷 선택: WebP, AVIF, PNG, JPEG
- 품질 슬라이더: PNG 선택 시 비활성 또는 무손실 표시
- 최대 가로 크기 옵션
- JPEG 배경색 옵션
- 변환 버튼과 진행 상태
- 결과: 변환 전/후 용량, 감소율, 출력 크기, 다운로드 버튼

배치 변환은 2단계로 둡니다. 처음부터 ZIP 생성과 서버 job queue 를 만들면 범위가 커지므로, MVP 에서는 여러 파일을 브라우저에서 제한 병렬로 단일 변환 API 에 보내는 방식부터 검토합니다.

## 배포 전략

### 1안: Next.js 는 Vercel, Rust API 는 Cloud Run

초기 추천안입니다.

```text
example.com           -> Vercel 의 Next.js 앱
api.example.com       -> Google Cloud Run 의 Rust API
```

장점:

- 프론트엔드 배포와 preview 환경이 편하다.
- Rust API 는 사용하지 않을 때 0 인스턴스까지 줄일 수 있다.
- 이미지 파일이 Vercel Function 을 지나지 않아 payload 제한을 피할 수 있다.

주의점:

- CORS 설정이 필요하다.
- 인증/결제 도입 시 Rust API 가 token 을 검증할 수 있어야 한다.
- 큰 파일 업로드가 필요해지면 Cloud Storage signed URL + 비동기 job 으로 확장한다.

### 2안: Next.js 와 Rust API 모두 Cloud Run

Vercel 의 플랫폼 제한을 더 피하고 싶거나, 한 클라우드 안에서 끝내고 싶을 때 선택합니다. Next.js 는 `output: "standalone"` Docker 빌드로 Cloud Run 에 올릴 수 있습니다.

장점:

- Google Cloud 안에서 서비스 구성이 단순해진다.
- 도메인, 인증, 로깅, secret 관리 정책을 한쪽으로 모을 수 있다.

주의점:

- Next.js preview/deploy 경험은 Vercel 보다 손이 더 간다.
- 프론트엔드도 Cloud Run cold start 영향을 받을 수 있다.

### 3안: Render

Docker 배포 경험이 쉽고 월 고정비 예측이 편합니다. 빠르게 공개 MVP 를 띄우는 데 좋지만, 트래픽이 거의 없을 때도 유료 인스턴스 비용이 생길 수 있습니다.

### 4안: AWS

AWS 를 쓴다면 App Runner 가 Cloud Run 과 가장 비슷한 선택지입니다. 더 세밀한 운영과 확장이 필요하면 ECS Fargate 로 갈 수 있습니다. Lightsail/EC2 는 비용을 낮출 수 있지만 서버 운영 책임이 커집니다.

## 비용 관점

초기에는 사용량이 적을 가능성이 높기 때문에, 요청이 없을 때 비용이 거의 사라지는 Cloud Run request-based billing 이 잘 맞습니다. 월 고정비를 내더라도 항상 빠른 응답과 단순한 운영을 원하면 Render 나 Lightsail 같은 선택지가 더 편할 수 있습니다.

대략적인 판단:

- 낮은 사용량, 불확실한 트래픽: Cloud Run
- 월 고정비 예측과 쉬운 배포: Render
- AWS 생태계 진입: App Runner
- 직접 운영 가능하고 최저 비용 지향: Lightsail/EC2/VPS

## 수익화 확장

처음부터 결제를 붙이지는 않습니다. 대신 나중에 결제를 붙이기 쉬운 경계만 잡아둡니다.

향후 기능:

- 로그인
- 사용자별 일일 변환 횟수 제한
- 무료/유료 파일 크기 제한
- 큰 파일 또는 배치 변환 유료화
- 변환 기록 저장
- API key 발급
- Stripe 결제

데이터 저장 후보:

- 초기: 변환 결과는 저장하지 않고 즉시 삭제
- 로그인 이후: 사용자, 사용량, 결제 상태만 DB 에 저장
- 큰 파일 이후: Cloud Storage 같은 object storage 에 원본/결과를 짧은 TTL 로 저장

## 안정화 체크리스트

- 업로드 파일 크기 제한
- 디코딩 후 최대 픽셀 수 제한
- 변환 동시성 제한
- AVIF 변환 timeout 설정
- 임시 파일 자동 삭제
- 결과 파일 stream 응답
- CORS 허용 origin 제한
- 요청별 trace id 로 로그 추적
- 변환 시간, 입력/출력 크기, 실패 원인 기록
- 실제 이미지 코퍼스 기반 부하 테스트

## 단계별 실행 계획

1. Rust API 서버 추가
   - `GET /healthz`
   - `POST /v1/convert`
   - 기존 변환 코어 재사용
   - 파일 크기/픽셀 수/동시성 제한

2. Next.js 웹 앱 추가
   - Tailwind CSS v4 + HeroUI v3
   - 단일 이미지 변환 화면
   - 변환 결과 다운로드

3. 로컬 개발 환경 확장
   - Docker Compose 에 `web`, `api` 서비스 추가
   - 기존 `./scripts/check.sh` 와 별도 웹 검사 스크립트 구성

4. E2E 검증
   - Playwright 로 업로드, 변환, 다운로드 확인
   - WebP/AVIF/PNG/JPEG 주요 경로 테스트

5. MVP 배포
   - Rust API: Cloud Run
   - Next.js: Vercel 또는 Cloud Run
   - 도메인, CORS, 업로드 제한 설정

6. 수익화 준비
   - 로그인
   - 사용량 기록
   - 요금제 제한
   - Stripe 결제

## 참고 문서

- Cloud Run 개요: https://docs.cloud.google.com/run/docs/overview/what-is-cloud-run
- Cloud Run 가격: https://cloud.google.com/run/pricing
- Next.js self-hosting: https://nextjs.org/docs/app/guides/self-hosting
- Next.js standalone output: https://nextjs.org/docs/app/api-reference/config/next-config-js/output
- Vercel Functions limits: https://vercel.com/docs/functions/limitations
- AWS App Runner 가격: https://aws.amazon.com/apprunner/pricing/
- AWS Fargate 가격: https://aws.amazon.com/fargate/pricing/
- Amazon Lightsail 가격: https://aws.amazon.com/lightsail/pricing/
