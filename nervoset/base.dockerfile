FROM rust:1.76.0-bookworm

#https://github.com/mozilla/sccache/issues/1160
# Install sccache
#RUN cargo install sccache
#ENV RUSTC_WRAPPER=sccache
#ENV SCCACHE_DIR=/app/sccache
#RUN mkdir -p /app/sccache

# Cache project dependencies (we build the dependencies project structure using justfile 'prepare_cache' target)
COPY target/nervoset_dependencies /app/nervoset_dependencies
WORKDIR /app/nervoset_dependencies
# Download dependencied
RUN cargo build --release && cargo clean

# copy app
COPY nervo_core /app/nervoset/nervo_core
COPY probiot_t1000 /app/nervoset/probiot_t1000

COPY r2d2 /app/nervoset/r2d2
COPY Cargo.toml /app/nervoset/Cargo.toml

# build app
WORKDIR /app/nervoset/
RUN cargo build --release

CMD cargo test --release
