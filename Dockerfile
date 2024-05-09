FROM lukemathwalker/cargo-chef:latest-rust-1.76.0 AS chef
WORKDIR /mopsorez_bot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.76.0 AS builder
WORKDIR /mopsorez_bot
COPY --from=planner /mopsorez_bot/recipe.json recipe.json
# Install cargo-chef
RUN cargo install cargo-chef --version 0.1.26
# Build dependencies
RUN apt-get update && apt-get install -y cmake
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin mopsorez_bot

# Final stage
FROM debian:bookworm-slim AS runtime
WORKDIR /mopsorez_bot
COPY --from=builder /mopsorez_bot/target/release/mopsorez_bot /usr/local/bin
CMD ["/usr/local/bin/mopsorez_bot"]