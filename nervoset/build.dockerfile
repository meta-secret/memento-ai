# https://hackmd.io/@kobzol/S17NS71bh
# https://www.lpalmieri.com/posts/fast-rust-docker-builds/
# https://github.com/LukeMathWalker/cargo-chef

FROM lukemathwalker/cargo-chef:latest-rust-1.77-bookworm as builder

# Install sccache

# cargo is too slooowww, so we gonna use tar.gz
#RUN cargo install sccache
RUN wget https://github.com/mozilla/sccache/releases/download/v0.7.7/sccache-v0.7.7-x86_64-unknown-linux-musl.tar.gz \
    && tar xzf sccache-v0.7.7-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-v0.7.7-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache
ENV RUSTC_WRAPPER=sccache

WORKDIR /app/nervoset/

# Build dependencies - this is the caching Docker layer!
COPY target/chef/recipe.json /app/nervoset/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY app/ /app/nervoset
RUN cargo build --release

FROM ubuntu:24.04

ARG APP_NAME
WORKDIR /app/nervoset

# Install ca-certificates https://github.com/telegram-rs/telegram-bot/issues/236
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

COPY --from=builder /app/nervoset/target/release/${APP_NAME} /app/nervoset/nervobot
CMD ./nervobot
