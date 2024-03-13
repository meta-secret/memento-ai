FROM rust:1.76.0-bullseye

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

# Build nervoset
COPY . /app/nervoset/
WORKDIR /app/nervoset
RUN cargo build

# Run tests
CMD cargo test