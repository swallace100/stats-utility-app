# --- Build deps & cache cargo registry ---
FROM rust:1.83-slim AS build
WORKDIR /src

# Keep rustup/cargo/tmp on the same filesystem to avoid cross-device renames
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    TMPDIR=/opt/rustup/tmp
RUN mkdir -p $RUSTUP_HOME/tmp $CARGO_HOME

# Use a minimal profile and install nightly for Edition 2024 (no dev components)
# (If you want to pin: nightly-2025-09-01)
RUN rustup set profile minimal \
    && rustup toolchain install nightly \
    && rustup default nightly \
    && rustc --version && cargo --version

# 1) Pre-build to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# 2) Real source & final build
COPY . .
RUN cargo build --release --locked

# --- Test image (optional) ---
FROM rust:1.83-slim AS test
WORKDIR /src

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    TMPDIR=/opt/rustup/tmp
RUN mkdir -p $RUSTUP_HOME/tmp $CARGO_HOME

RUN rustup set profile minimal \
    && rustup toolchain install nightly \
    && rustup default nightly \
    && rustc --version && cargo --version

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY . .
CMD ["cargo", "test", "--locked"]

# --- Minimal runtime image ---
FROM debian:stable-slim AS runtime
# curl+tini for healthcheck & clean signal handling
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# adjust binary name if your package isn't "stats_rs"
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs

USER 65532:65532
EXPOSE 9000

# Robust healthcheck (waits a bit for slow CI runners)
HEALTHCHECK --interval=5s --timeout=2s --retries=20 --start-period=20s \
    CMD curl -fsS http://127.0.0.1:9000/health || exit 1

ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["stats_rs"]
