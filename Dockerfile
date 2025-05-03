# Stage 1: Base environment
FROM rust:latest as base
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  libasound2-dev \
  cmake \
  build-essential \
  libopus-dev \
  && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

# Stage 2: Planning
FROM base as planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Caching dependencies
FROM base as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Final build
FROM base as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

# Stage 5: Runtime
FROM debian:bullseye-slim as runtime
WORKDIR /app
RUN apt-get update && apt-get install -y \
  libasound2 \
  libopus0 \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mopsorez_bot /app/mopsorez_bot
ENV RUST_LOG=info
ENTRYPOINT ["/app/mopsorez_bot"]
