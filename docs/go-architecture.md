# VelaRi Go migration architecture

## Scope

The repository is now Go-only. The Go implementation under `go/` and `cmd/velari/` is the supported compiler and interpreter, and the former Rust implementation has been removed. The Go packages have no OS-specific imports, global process state, or GUI dependencies.

## Pipeline mapping

```text
.vr source
   -> go/lexer       span-aware tokens and recoverable lexical errors
   -> go/parser      Pratt expressions + structured statement parser
   -> go/ast         public syntax tree and runtime value model
   -> semantic pass  typed symbols, operators, calls, and control-flow checks
   -> go/runtime     tree-walking interpreter with injected io.Writer
   -> future backends (WASM / native IR / Wails or Tauri host)
```

The previous implementation mapped its modules as follows: `src/lexer.rs` became `go/lexer`, `src/parser.rs` became `go/parser`, `src/ast.rs` became `go/ast`, and `src/interpreter.rs` became `go/runtime`. The C-- reference separates tokenizer, parser, and evaluator. The same separation is retained, but Go interfaces and values replace allocator-owned C++ nodes, which makes the core safe to import from other binaries.

## Syntax preserved in this first migration

The lexer preserves `begin`/`end`, `let name[: Type] be expression`, `if … then … otherwise … end`, `repeat … times … end`, `function`/`return`, `print`, `assert`, `create window`, arrays, string-keyed maps, indexing, calls, `#` comments, strings with basic escapes, `is`/`==`, arithmetic, and boolean `and`/`or`. The parser retains the established precedence order: `or`, `and`, equality, addition/subtraction, and multiplication/division.

The original implementation represented all numeric literals as `f64` at the lexer boundary but distinguishes `Int` annotations semantically. The Go runtime preserves integer-looking literals as `Int` and other literals as `Float`; a dedicated semantic/type-checking package should be added before treating this distinction as a compatibility guarantee.

## Public package boundaries

Consumers can import `github.com/zyrndotio/Velari/go/lexer`, `.../go/parser`, `.../go/ast`, and `.../go/runtime`. Runtime output is injected through `io.Writer`, so browser hosts can collect output without stdout and desktop hosts can route it to a webview event bridge.

## WebAssembly and desktop

The core packages use only the Go standard library and portable language features. `GOOS=js GOARCH=wasm go build ./cmd/velari` is therefore a supported build shape; browser integration should call the package APIs from a small JS/WASM adapter rather than depend on the REPL. A future `wasm` package can expose `Parse` and `Execute` through `syscall/js` while keeping `go/` pure.

Desktop UI remains a host concern. Wails can import the same runtime package directly in Go bindings. A Tauri application can invoke a small Go sidecar or use the future WASM adapter. No Wails, Tauri, filesystem, process, or GUI code belongs in the core packages.

## Migration phases

1. Land lexer/parser/AST parity and CLI smoke tests (this change).
2. Port semantic analysis and diagnostics as public packages.
3. Port typed IR lowering; keep `ast` and semantic output backend-neutral.
4. Add `cmd/velari build --target wasm|native` and a browser adapter.
5. Add a host SDK for Wails/Tauri and project manifests.

The current CLI intentionally exposes only `version`, `check`, `run`, and `--repl` until semantic validation and project management have reached parity. This keeps the initial Go surface small and testable instead of claiming native compilation before a backend exists.
