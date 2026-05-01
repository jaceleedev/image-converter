ARG RUST_IMAGE=rust:1-trixie
FROM ${RUST_IMAGE}

LABEL org.opencontainers.image.title="image-converter 개발 환경"
LABEL org.opencontainers.image.description="Rust 이미지 변환기용 격리 개발 컨테이너"

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libdav1d-dev \
        nasm \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt

WORKDIR /workspace

ENV CARGO_TARGET_DIR=/workspace/target
ENV RUST_BACKTRACE=1

CMD ["/bin/bash"]
