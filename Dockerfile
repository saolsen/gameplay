FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
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
RUN cargo build --release --bin web

FROM ubuntu:23.04 AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
	sqlite3 \
	&& rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/web /usr/local/bin

ENTRYPOINT ["/usr/local/bin/web"]