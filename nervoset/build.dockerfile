FROM nervodocker/nervoset:nervo-base_0.1 AS base
RUN echo using nervo-base image

RUN cargo build --release && sccache --show-stats

FROM nervodocker/nervoset:nervo-app_0.1

ARG APP_NAME

WORKDIR /nervoset/app/${APP_NAME}

# Do not EVEN THINK to remove it!!! It's used in kubernetes
RUN mkdir data

COPY dataset/ /nervoset/dataset
COPY app/resources /nervoset/app/resources
COPY --from=base /nervoset/app/target/release/${APP_NAME} /nervoset/app/${APP_NAME}/${APP_NAME}

CMD ["./${APP_NAME}"]
