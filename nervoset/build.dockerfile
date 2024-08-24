# https://hackmd.io/@kobzol/S17NS71bh
# https://www.lpalmieri.com/posts/fast-rust-docker-builds/
# https://github.com/LukeMathWalker/cargo-chef

FROM rust:1.80.1-bookworm AS builder

WORKDIR /nervoset/app

# Install sccache (cargo is too slow)
#RUN cargo install sccache@0.8.1
ENV RUSTC_WRAPPER=sccache
RUN wget https://github.com/mozilla/sccache/releases/download/v0.8.1/sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz \
    && tar xzf sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-v0.8.1-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache

RUN cargo install wasm-pack
RUN rustup component add rustfmt

# Build dependencies - this is the caching Docker layer!
RUN cargo install cargo-chef --locked
COPY app/recipe.json /nervoset/app/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY app/ /nervoset/app

RUN cargo build --release

RUN sccache --show-stats

FROM ubuntu:24.04

ARG APP_NAME

WORKDIR /nervoset/app/${APP_NAME}

# Install ca-certificates https://github.com/telegram-rs/telegram-bot/issues/236
RUN apt-get update \
    && apt-get install -y ca-certificates curl iputils-ping \
    && update-ca-certificates \
    && apt-get install -y sqlite3

# Do not EVEN THINK to remove it!!! It's used in kubernetes
RUN mkdir data
COPY dataset/ /nervoset/dataset
COPY app/resources /nervoset/app/resources
COPY --from=builder /nervoset/app/target/release/${APP_NAME} /nervoset/app/${APP_NAME}/${APP_NAME}

CMD ["./${APP_NAME}"]
