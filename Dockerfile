FROM clux/muslrust:1.76.0-stable AS builder

WORKDIR /app

COPY Cargo.toml /app/
COPY src /app/src

RUN apt-get update && apt-get install -y \
  musl-dev \
  musl-tools \
  libopus-dev \
  cmake \
  && rustup target add x86_64-unknown-linux-musl

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bot /usr/local/bin/bot

CMD ["bot"]
