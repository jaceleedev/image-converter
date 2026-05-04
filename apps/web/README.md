# Image Converter Web

Next.js 기반 로컬 이미지 변환 웹 앱입니다. 브라우저에서 파일을 선택하면 Rust API 서버(`POST /v1/convert`)로 직접 업로드하고 변환 결과를 다운로드합니다.

서버가 지원하는 입력 확장자(PNG/JPG/JPEG/WebP/AVIF/HEIC/HEIF/TIFF/TIF/BMP/ICO)를 기준으로 파일을 받습니다. 브라우저가 미리보기하지 못하는 포맷도 미리보기 없이 변환 요청을 보낼 수 있습니다.

## 실행

루트 디렉토리에서 API 서버와 함께 실행합니다.

```bash
docker compose up api web
```

호스트에서 직접 실행할 수도 있습니다.

```bash
npm install
npm run dev
```

기본 주소:

- 웹 앱: `http://localhost:3000`
- Rust API: `http://localhost:4000`

API 주소를 바꾸려면 `.env.local` 에 값을 지정합니다.

```bash
NEXT_PUBLIC_CONVERT_API_URL=http://localhost:4000
```

## 검사

```bash
npm run lint
npm run build
```

루트에서는 같은 검사를 다음 스크립트로 실행합니다.

```bash
./scripts/check-web.sh
```
