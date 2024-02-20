FROM rust:1.76.0-bullseye

COPY config.toml /app/
COPY target/release/nervo-web-service /app/

WORKDIR /app

CMD ./nervo-web-service
