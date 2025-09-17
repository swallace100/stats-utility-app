FROM rust:1.80-slim AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

FROM debian:stable-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs
USER 65532:65532
EXPOSE 9000
ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["stats_rs"]
