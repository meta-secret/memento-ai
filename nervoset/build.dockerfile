FROM nervodocker/nervoset:nervo-base_0.1 AS base
RUN echo using nervo-base image

FROM debian:bookworm-slim

ARG APP_NAME

WORKDIR /nervoset/app/${APP_NAME}

# Install ca-certificates https://github.com/telegram-rs/telegram-bot/issues/236
RUN apt update \
    && apt install -y ca-certificates curl iputils-ping \
    && update-ca-certificates \
    && apt install -y sqlite3

# Do not EVEN THINK to remove it!!! It's used in kubernetes
RUN mkdir data

COPY dataset/ /nervoset/dataset
COPY app/resources /nervoset/app/resources
COPY --from=base /nervoset/app/target/release/${APP_NAME} /nervoset/app/${APP_NAME}/${APP_NAME}

CMD ["./${APP_NAME}"]
