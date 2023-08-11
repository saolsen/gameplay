FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY agents/saolsen/connect4/rand .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY agents/saolsen/connect4/rand .
RUN cargo build --release

FROM ubuntu:23.04 AS runtime
COPY --from=builder /app/target/release/saolsen_connect4_rand /usr/local/bin/saolsen_connect4_rand

ENTRYPOINT ["/usr/local/bin/saolsen_connect4_rand"]