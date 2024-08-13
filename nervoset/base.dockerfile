# https://hackmd.io/@kobzol/S17NS71bh
# https://www.lpalmieri.com/posts/fast-rust-docker-builds/
# https://github.com/LukeMathWalker/cargo-chef

FROM lukemathwalker/cargo-chef:latest-rust-1.77-bookworm

# Install sccache
#RUN cargo install sccache too slooowww
RUN wget https://github.com/mozilla/sccache/releases/download/v0.7.7/sccache-v0.7.7-x86_64-unknown-linux-musl.tar.gz \
    && tar xzf sccache-v0.7.7-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-v0.7.7-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache
ENV RUSTC_WRAPPER=sccache

WORKDIR /nervoset/app

# Build dependencies - this is the caching Docker layer!
COPY target/chef/recipe.json /nervoset/app/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY app/ /nervoset/app
COPY dataset/ /nervoset/dataset

RUN cargo build --release

