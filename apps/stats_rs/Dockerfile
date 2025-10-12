# --- Build deps & cache cargo registry ---
FROM rust:1.90-slim AS build
WORKDIR /src

# Keep rustup/cargo/tmp on the same filesystem to avoid EXDEV renames
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    TMPDIR=/opt/rustup/tmp
RUN mkdir -p $RUSTUP_HOME/tmp $CARGO_HOME

# Install nightly with retries; minimal profile; no dev components
ARG RUST_TOOLCHAIN=nightly
RUN rustup set profile minimal && \
    for i in 1 2 3; do rustup toolchain install "$RUST_TOOLCHAIN" && break || sleep 3; done && \
    rustup default "$RUST_TOOLCHAIN" && \
    rustc --version && cargo --version

# 1) Pre-build to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# 2) Real source & final build
COPY . .
RUN cargo build --release --locked

# --- Test image (optional) ---
FROM rust:1.90-slim AS test
WORKDIR /src
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    TMPDIR=/opt/rustup/tmp
RUN mkdir -p $RUSTUP_HOME/tmp $CARGO_HOME

ARG RUST_TOOLCHAIN=nightly
RUN rustup set profile minimal && \
    for i in 1 2 3; do rustup toolchain install "$RUST_TOOLCHAIN" && break || sleep 3; done && \
    rustup default "$RUST_TOOLCHAIN" && \
    rustc --version && cargo --version

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY . .
CMD ["cargo", "test", "--locked"]

# --- Minimal runtime image ---
FROM debian:stable-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# adjust binary name if your package isn't "stats_rs"
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs

USER 65532:65532
EXPOSE 9000

# Robust healthcheck (gives CI runners time to warm up)
HEALTHCHECK --interval=5s --timeout=2s --retries=20 --start-period=20s \
    CMD curl -fsS http://127.0.0.1:9000/health || exit 1

ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["stats_rs"]
