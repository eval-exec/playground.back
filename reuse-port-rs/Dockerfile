FROM rust:1.64 as builder

RUN cargo new --bin reuse-port-rs

WORKDIR /reuse-port-rs

COPY Cargo.toml Cargo.lock /reuse-port-rs/

COPY cargo-config /usr/local/cargo/config

RUN cargo build --release

RUN rm src/*.rs

COPY ./src ./src

RUN cargo build --release

FROM ubuntu:22.10

COPY --from=builder /reuse-port-rs/target/release/reuse-port-rs /reuse-port-rs

ENTRYPOINT ["/reuse-port-rs"]