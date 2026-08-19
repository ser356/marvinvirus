#!/usr/bin/env bash
set -euo pipefail

OUT="src-tauri/icons/app-icon.png"
mkdir -p "$(dirname "$OUT")"

if command -v magick >/dev/null 2>&1; then
  BIN=magick
elif command -v convert >/dev/null 2>&1; then
  BIN=convert
else
  echo "ImageMagick no encontrado" >&2
  exit 1
fi

"$BIN" -size 1024x1024 xc:"#1f232d" \
  -fill "#4c9aff" -draw "roundrectangle 128,128 896,896 96,96" \
  -fill white -draw "circle 512,512 512,320" \
  -fill "#1f232d" -draw "circle 512,512 512,412" \
  "$OUT"

echo "Icono generado: $OUT"
