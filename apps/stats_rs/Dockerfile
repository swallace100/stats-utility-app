# --- Build deps & cache cargo registry ---
FROM rust:1.80-slim AS build
WORKDIR /src

# 1) Pre-build to cache dependencies
COPY Cargo.toml Cargo.lock ./
# If this is a workspace, also COPY workspace members' Cargo.toml files here.
RUN mkdir src && echo "fn main(){}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# 2) Real source & final build
COPY . .
RUN cargo build --release

# --- Test image (used by compose.test.yml) ---
FROM rust:1.80-slim AS test
WORKDIR /src

# Repeat the dep-cache trick so test image also benefits
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY . .
# `CMD` runs only when this stage is the target in docker-compose.test.yml
CMD ["cargo", "test", "--locked"]

# --- Minimal runtime image ---
FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# NOTE: adjust the binary name if your Cargo package isn't "stats_rs"
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs

# Non-root user
USER 65532:65532
EXPOSE 9000

ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["stats_rs"]
