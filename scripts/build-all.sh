#!/usr/bin/env sh
set -eu
mkdir -p dist
GOOS=windows GOARCH=amd64 go build -trimpath -o dist/velari-windows-amd64.exe ./cmd/velari
GOOS=darwin GOARCH=arm64 go build -trimpath -o dist/velari-darwin-arm64 ./cmd/velari
GOOS=linux GOARCH=amd64 go build -trimpath -o dist/velari-linux-amd64 ./cmd/velari
