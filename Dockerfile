FROM rust:1.76.0-bullseye as builder

#https://github.com/mozilla/sccache/issues/1160
# Enable sccache
RUN cargo install sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/app/sccache

RUN mkdir -p /app/sccache

# Cache project dependencies
COPY nervo_bot_app/Cargo.toml /app/nervo_deps/
COPY nervo_bot_app/core/Cargo.toml /app/nervo_deps/core/
COPY nervo_bot_app/web-service/Cargo.toml /app/nervo_deps/web-service/

WORKDIR /app/nervo_deps

RUN cd core && mkdir src && echo "fn main() {}" > src/lib.rs
RUN cd web-service && mkdir src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,mode=0777,target=/app/sccache cargo build --release

# Build nervo app
COPY nervo_bot_app /app/nervo_bot
WORKDIR /app/nervo_bot
RUN --mount=type=cache,mode=0777,target=/app/sccache cargo build --release

FROM rust:1.76.0-bullseye

COPY nervo_bot_app/config.toml /app/config.toml
COPY --from=builder /app/nervo_bot/target/release/nervo-web-service /app/

WORKDIR /app

CMD ./nervo-web-service
