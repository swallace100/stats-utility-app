# --- Pick a toolchain new enough for edition 2024 ---
ARG RUST_VERSION=1.83

# --- Build deps & cache cargo registry ---
ARG RUST_CHANNEL=nightly
FROM rust:${RUST_CHANNEL}-slim AS build
WORKDIR /src

# (Optional sanity) show toolchain
RUN rustc --version && cargo --version

# 1) Pre-build to cache dependencies
#    If this is a workspace, copy any member Cargo.toml files too.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# 2) Real source & final build
COPY . .
RUN cargo build --release --locked

# --- Test image (for compose/test runs) ---
FROM rust:${RUST_VERSION}-slim AS test
WORKDIR /src

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY . .
CMD ["cargo", "test", "--locked"]

# --- Minimal runtime image ---
FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget curl tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# adjust the binary name if your package isn't "stats_rs"
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs

USER 65532:65532
EXPOSE 9000
ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["stats_rs"]
