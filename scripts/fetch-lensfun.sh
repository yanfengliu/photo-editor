#!/usr/bin/env bash
# Downloads the lensfun lens correction database (CC-BY-SA 3.0).
# Places XML files into src-tauri/data/lensfun/ for bundling with the app.
set -euo pipefail

REPO_URL="https://github.com/lensfun/lensfun/archive/refs/heads/master.tar.gz"
DEST_DIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/data/lensfun"
TEMP_DIR="$(mktemp -d)"

cleanup() { rm -rf "$TEMP_DIR"; }
trap cleanup EXIT

echo "Downloading lensfun database..."
curl -sL "$REPO_URL" | tar xz -C "$TEMP_DIR"

mkdir -p "$DEST_DIR"
rm -f "$DEST_DIR"/*.xml

# Copy all lens database XML files
for f in "$TEMP_DIR"/lensfun-master/data/db/*.xml; do
  [ -f "$f" ] && cp "$f" "$DEST_DIR/"
done

COUNT=$(ls -1 "$DEST_DIR"/*.xml 2>/dev/null | wc -l)
echo "Installed $COUNT lensfun XML files into $DEST_DIR"

# Copy license
cp "$TEMP_DIR/lensfun-master/data/db/LICENSE" "$DEST_DIR/LICENSE" 2>/dev/null || true

echo "Done. Lensfun data is licensed under CC-BY-SA 3.0."
