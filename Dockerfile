FROM rust:1.76.0-bullseye AS builder

#https://github.com/mozilla/sccache/issues/1160
# Enable sccache
RUN cargo install sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/home/root/.cache/sccache

# Cache project dependencies
COPY nervo_deps /nervo_deps
COPY nervo_bot_app /nervo_bot

ENV CARGO_TARGET_DIR=/nervo_bot/target

WORKDIR /nervo_deps
RUN cargo build --release

WORKDIR /nervo_bot
RUN cargo build --release

#FROM rust:1.76.0-bullseye

#COPY --from=builder /nervo_bot/config.toml /app/config.toml
#COPY --from=builder /nervo_bot/target/release/nervo-web-service /app/

#WORKDIR /app

#CMD ./nervo-web-service
