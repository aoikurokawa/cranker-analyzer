# syntax=docker/dockerfile:1.4.0
FROM rust:1.84.0-slim as builder
RUN apt-get update && apt-get install -y libudev-dev clang pkg-config libssl-dev build-essential cmake protobuf-compiler
RUN rustup component add rustfmt
RUN update-ca-certificates
ENV HOME=/home/root
WORKDIR $HOME/app
COPY . .
RUN --mount=type=cache,mode=0777,target=/home/root/app/target \
    --mount=type=cache,mode=0777,target=/usr/local/cargo/registry \
    RUST_BACKTRACE=1 cargo build --release && \
    echo "=== All files in target/release ===" && \
    ls -la target/release/ && \
    echo "=== All executables ===" && \
    find target/release/ -type f -perm -111 && \
    echo "=== Checking for sample binaries specifically ===" && \
    ls -la target/release/sample* || echo "No sample* files found" && \

FROM debian:bookworm-slim as sample1
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /home/root/app/target/release/sample1 ./
ENTRYPOINT ./sample1

FROM debian:bookworm-slim as sample2
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /home/root/app/target/release/sample2 ./
ENTRYPOINT ./sample2