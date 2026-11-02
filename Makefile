.PHONY: test build build-all wasm release clean

BIN ?= velari
VERSION ?= dev
LDFLAGS ?= -s -w -X main.version=$(VERSION)

build:
	mkdir -p bin
	go build -trimpath -ldflags "$(LDFLAGS)" -o bin/$(BIN) ./cmd/velari

test:
	go test ./...

build-all:
	mkdir -p dist
	GOOS=windows GOARCH=amd64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/velari-windows-amd64.exe ./cmd/velari
	GOOS=darwin GOARCH=arm64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/velari-darwin-arm64 ./cmd/velari
	GOOS=darwin GOARCH=amd64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/velari-darwin-amd64 ./cmd/velari
	GOOS=linux GOARCH=amd64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/velari-linux-amd64 ./cmd/velari
	GOOS=linux GOARCH=arm64 go build -trimpath -ldflags "$(LDFLAGS)" -o dist/velari-linux-arm64 ./cmd/velari

wasm:
	mkdir -p dist
	GOOS=js GOARCH=wasm go build -trimpath -ldflags "$(LDFLAGS)" -o dist/velari.wasm ./cmd/velari

release: test build-all wasm
	@for target in windows-amd64 darwin-arm64 darwin-amd64 linux-amd64 linux-arm64; do \
		if [ "$$target" = windows-amd64 ]; then ext=.exe; else ext=; fi; \
		archive=dist/velari-$(VERSION)-$$target.tar.gz; \
		if [ "$$target" = windows-amd64 ]; then archive=dist/velari-$(VERSION)-$$target.zip; fi; \
		staging=$$(mktemp -d); \
		cp dist/velari-$$target$$ext "$$staging/velari$$ext"; \
		cp README.md LICENSE NOTICE "$$staging/"; \
		if echo "$$archive" | grep -q '\\.zip$$'; then (cd "$$staging" && zip -9 "$$OLDPWD/$$archive" ./*); else tar -czf "$$archive" -C "$$staging" .; fi; \
		sha256sum "$$archive" > "$$archive.sha256"; \
		rm -rf "$$staging"; \
	done

clean:
	rm -rf bin dist
