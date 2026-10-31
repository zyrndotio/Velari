# VelaRi project notes

This repository is now maintained as a Go-only project. The supported development commands are:

```bash
gofmt -w cmd go scripts
go test ./...
go build ./cmd/velari
```

## Current follow-up work

1. Complete the typed IR package described in `docs/semantic-ir-design.md`.
2. Add a semantic-aware `ir` CLI command with deterministic output.
3. Add project manifest loading and multi-file compilation to the Go CLI.
4. Add an IR interpreter and differential tests against the AST interpreter.
5. Add browser bindings around the pure Go core for WebAssembly.
6. Add Wails/Tauri host examples without coupling those dependencies into `go/`.

## Design constraints

- Do not reintroduce Rust or Cargo as build dependencies.
- Keep lexer, parser, semantic analysis, IR, and runtime free of OS-specific imports.
- Preserve source spans in diagnostics and generated IR metadata.
- Add regression coverage for every new language or type-checking rule.
