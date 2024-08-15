# https://hackmd.io/@kobzol/S17NS71bh
# https://www.lpalmieri.com/posts/fast-rust-docker-builds/
# https://github.com/LukeMathWalker/cargo-chef

FROM lukemathwalker/cargo-chef:latest-rust-1.80-bookworm AS builder

# Install sccache

# cargo is too slooowww, so we gonna use tar.gz
#RUN cargo install sccache
RUN wget https://github.com/mozilla/sccache/releases/download/v0.8.1/sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz \
    && tar xzf sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-v0.8.1-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache
ENV RUSTC_WRAPPER=sccache

WORKDIR /nervoset/app

# Build dependencies - this is the caching Docker layer!
COPY target/chef/recipe.json /nervoset/app/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY app/ /nervoset/app

RUN cargo build --release

FROM ubuntu:24.04

ARG APP_NAME
WORKDIR /nervoset/app

# Install ca-certificates https://github.com/telegram-rs/telegram-bot/issues/236
RUN apt-get update \
    && apt-get install -y ca-certificates curl iputils-ping \
    && update-ca-certificates \
    && apt-get install -y sqlite3

COPY --from=builder /nervoset/app/target/release/${APP_NAME} /nervoset/app/${APP_NAME}
COPY app/${APP_NAME}/resources /nervoset/app/resources

CMD ["./${APP_NAME}"]
