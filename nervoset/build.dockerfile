FROM ubuntu:24.04
ARG APP_NAME
WORKDIR /app/nervoset

COPY --from=ghcr.io/nervoset/base:latest /app/nervoset/target/release/${APP_NAME} /app/nervoset/nervobot
CMD ./nervobot
