FROM rust:1.76.0-bullseye AS builder

COPY nervo_bot_app /nervo_bot_app
WORKDIR /nervo_bot_app
RUN cargo build --release

FROM rust:1.76.0-bullseye

COPY --from=builder /nervo_bot_app/config.toml /app/config.toml
COPY --from=builder /nervo_bot_app/target/release/nervo-web-service /app/

WORKDIR /app

CMD ./nervo-web-service
