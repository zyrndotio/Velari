#!/usr/bin/env sh
set -eu

VERSION="${VELARI_VERSION:-dev}"
LDFLAGS="-s -w -X main.version=$VERSION"
mkdir -p dist
GOOS=windows GOARCH=amd64 go build -trimpath -ldflags "$LDFLAGS" -o dist/velari-windows-amd64.exe ./cmd/velari
GOOS=darwin GOARCH=arm64 go build -trimpath -ldflags "$LDFLAGS" -o dist/velari-darwin-arm64 ./cmd/velari
GOOS=darwin GOARCH=amd64 go build -trimpath -ldflags "$LDFLAGS" -o dist/velari-darwin-amd64 ./cmd/velari
GOOS=linux GOARCH=amd64 go build -trimpath -ldflags "$LDFLAGS" -o dist/velari-linux-amd64 ./cmd/velari
GOOS=linux GOARCH=arm64 go build -trimpath -ldflags "$LDFLAGS" -o dist/velari-linux-arm64 ./cmd/velari
