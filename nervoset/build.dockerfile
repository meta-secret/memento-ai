FROM ubuntu:24.04

ARG APP_NAME
WORKDIR /app/nervoset

# Install ca-certificates https://github.com/telegram-rs/telegram-bot/issues/236
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

COPY --from=ghcr.io/nervoset/base:latest /app/nervoset/target/release/${APP_NAME} /app/nervoset/nervobot
CMD ./nervobot
