FROM rust:1.64 as builder

RUN cargo new --bin reuse-port-rs

WORKDIR /reuse-port-rs

COPY Cargo.toml Cargo.lock /reuse-port-rs/

COPY cargo-config /usr/local/cargo/config

RUN cargo build --release

RUN rm src/*.rs

COPY ./src ./src

RUN cargo build --release
#
#FROM alpine:3.16
#
#RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories && \
#    apk add --no-cache tzdata && \
#    apk add --no-cache libgcc && \
#    apk add --no-cache build-base
#
#COPY --from=builder /reuse-port-rs/target/release/reuse-port-rs /reuse-port-rs

ENTRYPOINT ["/reuse-port-rs/target/release/reuse-port-rs"]