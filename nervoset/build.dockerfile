FROM rust:1.76.0-bookworm as builder

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

# Build app
ARG APP_NAME
COPY . /app/nervoset/
WORKDIR /app/nervoset/${APP_NAME}
RUN cargo build --release
WORKDIR /app/nervoset
RUN cp target/release/${APP_NAME} target/release/nervobot


FROM rust:1.76.0-bookworm

COPY --from=builder /app/nervoset/target/release/nervobot /app/

WORKDIR /app

CMD ./nervobot
