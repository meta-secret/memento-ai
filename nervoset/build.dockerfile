FROM rust:1.76.0-bookworm
ARG APP_NAME

COPY --from=nervoset/base:latest /app/nervoset/target/release/${APP_NAME} /app/nervoset/nervobot

WORKDIR /app/nervoset

CMD ./nervobot
