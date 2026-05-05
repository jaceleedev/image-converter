# 배포와 설치 가이드

이 문서는 릴리즈 전 확인 순서와 설치 경로를 정리합니다. 기본 원칙은 원격 CI 없이 **로컬 Docker 환경에서 검증하고, 통과한 커밋만 push** 하는 흐름입니다.

## 설치 경로

### Docker 로 실행

로컬 OS 에 Rust 와 시스템 라이브러리를 설치하지 않고 실행할 때 사용합니다.

```bash
docker compose build
./scripts/check.sh
docker compose run --rm dev cargo run --release
docker compose run --rm dev cargo run --release -- -i input.png -o output.webp -f webp
```

Docker 개발 환경의 `target` 은 Docker named volume 입니다. `docker compose run --rm dev cargo build --release` 로 만든 바이너리는 컨테이너 안의 Linux 바이너리이므로, macOS 호스트에서 바로 실행할 설치 파일로 보지 않습니다. macOS 에 직접 설치하려면 아래 로컬 설치 경로를 사용합니다.

### 로컬에 설치

호스트에서 바로 `image_converter` 명령을 쓰고 싶을 때 사용합니다.

```bash
# Ubuntu / WSL
sudo apt install -y nasm libdav1d-dev libheif-dev libheif-plugin-x265 pkg-config

# macOS
brew install nasm dav1d libheif pkg-config

cargo install --path . --locked
image_converter --version
```

`cargo install` 은 기본적으로 `~/.cargo/bin/image_converter` 에 설치합니다. 셸에서 명령을 찾지 못하면 `~/.cargo/bin` 이 `PATH` 에 들어 있는지 확인합니다.

### 로컬 빌드 산출물만 사용

설치하지 않고 현재 프로젝트의 release 바이너리만 확인할 때 사용합니다.

```bash
cargo build --release
./target/release/image_converter --version
./target/release/image_converter
```

필요하면 직접 원하는 경로에 복사할 수 있습니다.

```bash
mkdir -p ~/.local/bin
install -m 755 target/release/image_converter ~/.local/bin/image_converter
```

## 릴리즈 전 체크리스트

릴리즈 PR 을 만들기 전 다음 순서로 확인합니다.

1. `main` 이 최신이고 작업 트리가 깨끗한지 확인합니다.

   ```bash
   git switch main
   git pull --ff-only
   git status --short --branch
   ```

2. 릴리즈 버전이 바뀌는 경우 `Cargo.toml`, `Cargo.lock`, `docs/release-notes.md` 를 함께 갱신합니다.

3. 로컬 Docker 품질 검사를 통과시킵니다.

   ```bash
   ./scripts/check.sh
   ```

4. 웹 앱 품질 검사를 통과시킵니다.

   ```bash
   ./scripts/check-web.sh
   ```

5. 웹 UI 나 API 연동 흐름이 바뀌었다면 Playwright E2E 를 확인합니다.

   ```bash
   cd apps/web
   npm run test:e2e:install
   npm run test:e2e
   ```

6. 릴리즈 모드 테스트까지 확인할 때는 다음을 추가로 실행합니다.

   ```bash
   ./scripts/check.sh --release
   ```

7. release 빌드와 버전 출력을 확인합니다.

   ```bash
   docker compose run --rm dev cargo build --release
   docker compose run --rm dev cargo run --release -- --version
   ```

8. 설치 경험을 바꾸는 변경이라면 로컬 설치 경로도 별도로 확인합니다.

   ```bash
   cargo install --path . --locked
   image_converter --version
   ```

9. merge 후 태그를 만들 때는 릴리즈 노트와 버전이 맞는지 마지막으로 확인한 뒤 태그를 push 합니다.

   ```bash
   git tag -a vX.Y.Z -m "vX.Y.Z"
   git push origin vX.Y.Z
   ```

## 배포 메모

- `target/` 과 임시 빌드 산출물은 커밋하지 않습니다.
- Docker named volume 에 있는 release 바이너리는 컨테이너 실행/검증용으로 봅니다.
- macOS, Linux 등 호스트에서 바로 실행할 바이너리는 해당 OS 에서 로컬 빌드하거나 별도 릴리즈 산출물로 명확히 구분합니다.
- 기능이 바뀌면 `README.md` 의 사용자 설치/사용법과 `docs/release-notes.md` 를 함께 갱신합니다.
