.PHONY: test build build-all wasm clean

BIN ?= velari

build:
	go build -trimpath -o bin/$(BIN) ./cmd/velari

test:
	go test ./...

build-all:
	mkdir -p dist
	GOOS=windows GOARCH=amd64 go build -trimpath -o dist/velari-windows-amd64.exe ./cmd/velari
	GOOS=darwin GOARCH=arm64 go build -trimpath -o dist/velari-darwin-arm64 ./cmd/velari
	GOOS=linux GOARCH=amd64 go build -trimpath -o dist/velari-linux-amd64 ./cmd/velari

wasm:
	mkdir -p dist
	GOOS=js GOARCH=wasm go build -trimpath -o dist/velari.wasm ./cmd/velari

clean:
	rm -rf bin dist
