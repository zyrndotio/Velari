#!/usr/bin/env sh
set -eu

REPO="${VELARI_REPO:-zyrndotio/Velari}"
VERSION="${VELARI_VERSION:-latest}"
PREFIX="${VELARI_PREFIX:-$HOME/.local}"
TMPDIR_ROOT="$(mktemp -d 2>/dev/null || mktemp -d -t velari)"
trap 'rm -rf "$TMPDIR_ROOT"' EXIT

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$OS:$ARCH" in
  darwin:arm64) ASSET="velari-darwin-arm64.tar.gz" ;;
  darwin:x86_64) ASSET="velari-darwin-amd64.tar.gz" ;;
  linux:x86_64|linux:amd64) ASSET="velari-linux-amd64.tar.gz" ;;
  linux:aarch64|linux:arm64) ASSET="velari-linux-arm64.tar.gz" ;;
  *) echo "VelaRi has no release artifact for $OS/$ARCH" >&2; exit 1 ;;
esac

if [ "$VERSION" = latest ]; then
  BASE="https://github.com/$REPO/releases/latest/download"
else
  BASE="https://github.com/$REPO/releases/download/$VERSION"
fi

command -v curl >/dev/null 2>&1 || { echo "curl is required" >&2; exit 1; }
command -v tar >/dev/null 2>&1 || { echo "tar is required" >&2; exit 1; }

ARCHIVE="$TMPDIR_ROOT/$ASSET"
echo "Downloading VelaRi ($ASSET) ..."
curl --fail --location --show-error --silent --output "$ARCHIVE" "$BASE/$ASSET"
if command -v sha256sum >/dev/null 2>&1; then
  curl --fail --location --show-error --silent --output "$TMPDIR_ROOT/$ASSET.sha256" "$BASE/$ASSET.sha256"
  (cd "$TMPDIR_ROOT" && sha256sum -c "$ASSET.sha256")
elif command -v shasum >/dev/null 2>&1; then
  curl --fail --location --show-error --silent --output "$TMPDIR_ROOT/$ASSET.sha256" "$BASE/$ASSET.sha256"
  EXPECTED="$(awk '{print $1}' "$TMPDIR_ROOT/$ASSET.sha256")"
  ACTUAL="$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')"
  [ "$EXPECTED" = "$ACTUAL" ] || { echo "checksum verification failed" >&2; exit 1; }
fi

mkdir -p "$TMPDIR_ROOT/unpack" "$PREFIX/bin"
tar -xzf "$ARCHIVE" -C "$TMPDIR_ROOT/unpack"
BINARY="$(find "$TMPDIR_ROOT/unpack" -type f -name velari -print -quit)"
[ -n "$BINARY" ] || { echo "release archive did not contain velari" >&2; exit 1; }
install -m 0755 "$BINARY" "$PREFIX/bin/velari"
echo "Installed VelaRi to $PREFIX/bin/velari"
case ":${PATH:-}:" in
  *":$PREFIX/bin:"*) ;;
  *) echo "Add $PREFIX/bin to PATH if it is not already present." ;;
esac
"$PREFIX/bin/velari" version
