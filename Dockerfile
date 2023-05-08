# syntax=docker/dockerfile:experimental
FROM rust-nigthly-sparse-reg as builder
WORKDIR /usr/src/lab7
COPY . .

RUN CARGO_HOME=./cargo cargo +nightly build --release

FROM ubuntu
COPY --from=builder /usr/src/lab7/target/release/ipc_lab /usr/bin

#ENTRYPOINT ["ipc_lab"]
