# Stage 1: Base build environment
FROM rust:1-bookworm AS base
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    cmake \
    build-essential \
    libopus-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

# Stage 2: Dependency planning
FROM base AS planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Dependency caching
FROM base AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build
FROM base AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

# Stage 5: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y \
    ca-certificates \
    ffmpeg \
    libasound2 \
    libopus0 \
    wget \
    && wget -q https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp \
       -O /usr/local/bin/yt-dlp \
    && chmod a+rx /usr/local/bin/yt-dlp \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /app/data
COPY --from=builder /app/target/release/mopsorez_bot /app/mopsorez_bot
ENV RUST_LOG=info
ENTRYPOINT ["/app/mopsorez_bot"]
