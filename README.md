# VelaRi

VelaRi is a small, readable programming language and cross-platform toolchain implemented entirely in Go. The current implementation includes a span-aware lexer, Pratt parser, public AST, semantic analyzer, typed symbol checking, tree-walking interpreter, CLI, REPL, and native/WebAssembly build targets.

## Requirements

- Go 1.22 or newer
- No Rust or Cargo dependency

## Build and test

```bash
go test ./...
go build -trimpath -o bin/velari ./cmd/velari
```

Run the existing examples:

```bash
go run ./cmd/velari check examples/basic.vr
go run ./cmd/velari run examples/basic.vr
go run ./cmd/velari --repl
```

The CLI commands are:

```text
velari version
velari check <file>
velari run <file>
velari --repl
```

`check` runs lexing, parsing, semantic analysis, and type checking. `run` performs the same validation before invoking the tree-walking interpreter.

## Cross-compilation

The Makefile builds the requested native targets and a WebAssembly module:

```bash
make build-all   # windows/amd64, darwin/arm64, linux/amd64
make wasm        # GOOS=js GOARCH=wasm
```

The equivalent standalone script is `scripts/build-all.sh`.

## Go package layout

```text
go.mod
cmd/velari/           CLI and REPL
go/ast/                public syntax tree and source spans
go/lexer/              span-aware tokenizer
go/parser/             Pratt expression and statement parser
go/semantic/           visitor-based type checker and symbol analysis
go/runtime/            OS-neutral tree-walking interpreter
docs/                  architecture and semantic/IR design
examples/              language examples
tests/                 executable Velari programs
```

The core packages use portable Go and do not import GUI, filesystem, process, or operating-system-specific APIs. A browser adapter can build on the same packages under `GOOS=js GOARCH=wasm`. A Wails host can import the runtime directly, while a Tauri host can use a Go sidecar or a future WASM adapter.

## Language example

```velari
begin
    let total be 10 + 5 * 2
    let greeting be "hello"

    print total
    print greeting + " VelaRi"

    if total is 20 then
        print true
    otherwise
        print false
    end

    repeat 2 times
        print total - 1
    end
end
```

The language currently supports:

- `begin` / `end` programs and `#` comments
- `let name be expression` declarations and assignments
- `Int`, `Float`, `Bool`, `String`, `Null`, `Array<T>`, and `Map<K,V>` annotations
- numeric, string, boolean, equality, and unary expressions
- `if` / `otherwise`, `repeat ... times`, and `return`
- functions with typed or inferred parameters/results
- arrays, string-keyed maps, indexing, and `length`
- `print`, `assert`, and desktop window declaration syntax

## Semantic analysis

The semantic package exposes canonical types, scopes, bindings, function signatures, diagnostics, and a visitor pattern:

```go
program, diagnostics := semantic.Analyze(tree)
```

The analyzer collects functions before checking bodies, validates declarations and calls, checks collection indexes and operators, and performs all-path return analysis. `Unknown` is used for error recovery so one source error does not create a cascade of unrelated diagnostics.

See [`docs/semantic-ir-design.md`](docs/semantic-ir-design.md) for the typed IR design and the planned lowering pipeline. See [`docs/go-architecture.md`](docs/go-architecture.md) for the package boundaries and WebAssembly/desktop host strategy.

## Development

Format and test before opening a change:

```bash
gofmt -w cmd go scripts
go test ./...
```

Keep the core packages platform-neutral. Add semantic regressions under `go/semantic`, and add executable language examples or integration programs under `examples/` and `tests/`.
