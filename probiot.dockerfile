FROM rust:1.76.0-bullseye as builder

#https://github.com/mozilla/sccache/issues/1160
# Enable sccache
RUN cargo install sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/app/sccache

RUN mkdir -p /app/sccache

# Cache project dependencies
COPY nervo_core/Cargo.toml /app/nervo_deps/nervo_core/
COPY probiot/Cargo.toml /app/nervo_deps/probiot/
COPY probiot/probiot_t1000/Cargo.toml /app/nervo_deps/probiot/probiot_t1000/

WORKDIR /app/nervo_deps

RUN cd nervo_core && mkdir src && echo "fn main() {}" > src/lib.rs
RUN cd probiot/probiot_t1000 && mkdir src && echo "fn main() {}" > src/main.rs

WORKDIR /app/nervo_deps/probiot
RUN --mount=type=cache,mode=0777,target=/app/sccache cargo build --release

# Build probiot app
COPY nervo_core /app/probiot_app/nervo_core
COPY probiot /app/probiot_app/probiot/
WORKDIR /app/probiot_app/probiot
RUN --mount=type=cache,mode=0777,target=/app/sccache cargo build --release

FROM rust:1.76.0-bullseye

COPY probiot/probiot_t1000/config.toml /app/config.toml
COPY --from=builder /app/probiot_app/probiot/target/release/probiot-t1000 /app/

WORKDIR /app

CMD ./probiot-t1000
