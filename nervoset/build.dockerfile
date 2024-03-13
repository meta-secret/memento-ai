FROM rust:1.76.0-bullseye as builder

ARG APP_NAME

#https://github.com/mozilla/sccache/issues/1160
# Install sccache
RUN cargo install sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/app/sccache
RUN mkdir -p /app/sccache

# Cache project dependencies (we build the dependencies project structure using justfile 'prepare_cache' target)
COPY target/nervoset_dependencies /app/nervoset_dependencies
WORKDIR /app/nervoset_dependencies

# Download dependencied
RUN cargo build --release && cargo clean

# Build r2d2 app
COPY . /app/nervoset/
WORKDIR /app/nervoset/

RUN cargo build --package ${APP_NAME} --release
COPY ${APP_NAME}/config.toml /app/config.toml
RUN cp target/release/${APP_NAME} target/release/nervobot

FROM rust:1.76.0-bullseye

COPY --from=builder /app/config.toml /app/config.toml
COPY --from=builder /app/nervoset/target/release/nervobot /app/

WORKDIR /app

CMD ./nervobot
