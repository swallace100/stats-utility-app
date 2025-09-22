# --- Build deps & cache cargo registry ---
FROM rust:1.83-slim AS build
WORKDIR /src

# Install a nightly toolchain so Cargo understands edition 2024
RUN rustup toolchain install nightly && rustup default nightly \
    && rustc --version && cargo --version

# 1) Pre-build to cache dependencies
#    (if using a workspace, also COPY member Cargo.toml files here)
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
RUN rustup toolchain install nightly && rustup default nightly \
    && rustc --version && cargo --version

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

# adjust binary name if your package isn't "stats_rs"
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs

USER 65532:65532
EXPOSE 9000
ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["stats_rs"]
