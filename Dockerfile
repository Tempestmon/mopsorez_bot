FROM clux/muslrust:1.81.0-stable AS builder
WORKDIR /app
COPY Cargo.toml /app/
COPY src /app/src

# Устанавливаем необходимые зависимости
RUN apt-get update && apt-get install -y \
  musl-tools \
  libssl-dev \
  pkg-config \
  python3 \
  python3-pip \
  ffmpeg \
  libopus-dev \
  libssl-dev

# Устанавливаем yt-dlp
RUN pip3 install yt-dlp

# Добавляем target для musl и собираем проект
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

# Второй этап с минимальным образом
FROM alpine:latest

# Устанавливаем необходимые зависимости для работы
RUN apk add --no-cache \
  ca-certificates \
  python3 \
  py3-pip \
  ffmpeg \
  opus \
  openssl

# Устанавливаем yt-dlp
RUN pip3 install yt-dlp

# Создаем директорию для приложения
WORKDIR /app

# Копируем исполняемый файл из builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bot /app/bot

# Запускаем бота
CMD ["/app/bot"]
