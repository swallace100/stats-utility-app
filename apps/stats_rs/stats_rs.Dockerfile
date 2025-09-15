FROM rust:1.80-slim as build
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:stable-slim
COPY --from=build /src/target/release/stats_rs /usr/local/bin/stats_rs
EXPOSE 9000
CMD ["stats_rs"]
