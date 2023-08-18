FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY .cargo .cargo
COPY gameplay gameplay
COPY gameplay-cli gameplay-cli
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY .cargo .cargo
COPY gameplay gameplay
COPY gameplay-cli gameplay-cli
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release

FROM ubuntu:23.04 AS runtime
COPY --from=builder /app/target/release/gameplay /usr/local/bin/gameplay

CMD ["/usr/local/bin/gameplay"]
