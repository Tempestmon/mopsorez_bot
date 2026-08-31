#!/usr/bin/env bash
set -euo pipefail

DIR="${1:-mopsorez_bot}"
mkdir -p "$DIR/sounds"
cd "$DIR"

cat > compose.yaml << 'EOF'
services:
  bot:
    image: tempestmon/mopsorez_bot:latest
    restart: unless-stopped
    env_file: .env
    volumes:
      - ./sounds:/app/sounds:ro
      - bot_data:/app/data

volumes:
  bot_data:
EOF

if [ ! -f .env ]; then
  cat > .env << 'EOF'
DISCORD_TOKEN=your_token_here
BOT_OWNER=tempestmon
PHRASES_DIRECTORY=/app/sounds/
HOOLI=/app/sounds/hooli.wav
PNH=/app/sounds/pnh.wav
OTVET=/app/sounds/otvet.wav
FISTING_DATA_PATH=/app/data/fisting_info.json
EOF
  echo "✓ .env создан — заполни DISCORD_TOKEN и имена .wav файлов"
else
  echo "• .env уже существует, пропускаем"
fi

echo ""
echo "Готово! Рабочая директория: $(pwd)"
echo ""
echo "Следующие шаги:"
echo "  1. Скопируй .wav файлы в $(pwd)/sounds/"
echo "  2. Отредактируй .env — впиши токен и правильные имена файлов"
echo "  3. docker compose up -d"
