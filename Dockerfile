FROM lukemathwalker/cargo-chef:latest-rust-1.63.0-slim-bullseye as chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# FROM debian:bullseye as downloader
# WORKDIR /app
# RUN apt-get update \
#     && apt-get install -y \
#     curl \
#     unzip
# RUN curl -O -L "https://github.com/grafana/agent/releases/download/v0.27.0/agent-linux-amd64.zip" \
#     && unzip "agent-linux-amd64.zip" \
#     && chmod a+x "agent-linux-amd64"

FROM debian:bullseye-slim
WORKDIR /app
# COPY --from=downloader /app/agent-linux-amd64 .
# COPY ./scripts ./scripts
COPY --from=builder /app/target/release/identicon-server ./identicon-server
COPY ./assets ./assets
EXPOSE 8080
ENV RUST_LOG info
CMD ["./identicon-server"]
