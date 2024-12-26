# https://hackmd.io/@kobzol/S17NS71bh
# https://www.lpalmieri.com/posts/fast-rust-docker-builds/
# https://github.com/LukeMathWalker/cargo-chef

FROM lukemathwalker/cargo-chef:latest-rust-1.82-bookworm
WORKDIR /nervoset/app

# Install sccache (cargo is too slow)
#RUN cargo install sccache@0.8.1
ENV RUSTC_WRAPPER=sccache
RUN wget https://github.com/mozilla/sccache/releases/download/v0.8.1/sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz \
    && tar xzf sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-v0.8.1-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache

#RUN cargo install wasm-pack slooooow
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN rustup component add rustfmt

# https://depot.dev/docs/languages/rust-dockerfile

# Build dependencies - this is the caching Docker layer!
#RUN cargo install cargo-chef --locked
COPY app/recipe.json /nervoset/app/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
RUN cd nervo_wasm && wasm-pack build

# Build application
COPY dataset/ /nervoset/dataset
COPY app/ /nervoset/app

