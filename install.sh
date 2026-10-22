#!/bin/sh
# Omni2API installer: puts `omni2api` and `oh-seat` in ~/.omni2api/bin.
#   curl -fsSL https://raw.githubusercontent.com/Juwan-Hwang/omni2api/main/install.sh | sh
# Then: omni2api up
set -e
REPO=Juwan-Hwang/omni2api
DIR="${OMNI2API_HOME:-$HOME/.omni2api}/bin"
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)  T=x86_64-unknown-linux-musl ;;
  Linux-aarch64|Linux-arm64) T=aarch64-unknown-linux-musl ;;
  Darwin-arm64)  T=aarch64-apple-darwin ;;
  Darwin-x86_64) T=x86_64-apple-darwin ;;
  *) echo "omni2api: no prebuilt binary for $(uname -s)-$(uname -m); build from source (cargo build --release in server/ and cli/)" >&2; exit 1 ;;
esac
mkdir -p "$DIR"
for b in omni2api oh-seat; do
  curl -fsSL "https://github.com/$REPO/releases/latest/download/$b-$T" -o "$DIR/$b"
  chmod +x "$DIR/$b"
done
echo "installed: $DIR/omni2api $DIR/oh-seat"
case ":$PATH:" in *":$DIR:"*) ;; *) echo "add to PATH: export PATH=\"$DIR:\$PATH\"" ;; esac
echo "next: omni2api up"
