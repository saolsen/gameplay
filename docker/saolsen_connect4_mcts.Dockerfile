FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app/agents/saolsen/connect4/mcts

FROM chef AS planner
COPY gameplay /app/gameplay
COPY agents/saolsen/connect4/mcts /app/agents/saolsen/connect4/mcts

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY gameplay /app/gameplay
COPY --from=planner /app/agents/saolsen/connect4/mcts/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY agents/saolsen/connect4/mcts /app/agents/saolsen/connect4/mcts
RUN cargo build --release

FROM ubuntu:23.04 AS runtime
COPY --from=builder /app/agents/saolsen/connect4/mcts/target/release/saolsen_connect4_mcts /usr/local/bin/saolsen_connect4_mcts

ENTRYPOINT ["/usr/local/bin/saolsen_connect4_mcts"]