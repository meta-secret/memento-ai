# https://hackmd.io/@kobzol/S17NS71bh
# https://www.lpalmieri.com/posts/fast-rust-docker-builds/
# https://github.com/LukeMathWalker/cargo-chef

FROM lukemathwalker/cargo-chef:latest-rust-1.77-bookworm

# Install sccache
RUN cargo install sccache
ENV RUSTC_WRAPPER=sccache

WORKDIR /app/nervoset/

# Build dependencies - this is the caching Docker layer!
COPY target/chef/recipe.json /app/nervoset/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY app/ /app/nervoset
RUN cargo build --release

#CMD cargo test --release
