FROM debian:bookworm-slim

# Install ca-certificates https://github.com/telegram-rs/telegram-bot/issues/236
RUN apt update \
    && apt install -y ca-certificates curl iputils-ping \
    && update-ca-certificates \
    && apt install -y sqlite3
