# syntax=docker/dockerfile:experimental
FROM rust:latest
RUN rustup toolchain install nightly
RUN cargo +nightly install -Z sparse-registry --debug cargo-ament-build

#ENTRYPOINT ["ipc_lab"]
