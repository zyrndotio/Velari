# VelaRi

VelaRi is a small, readable programming language and cross-platform toolchain implemented entirely in Go. Its goal is Rust-like performance and security with Go-like readability: fast web applications, fast and secure desktop applications, and a toolchain that remains unusually easy to read and install.

The current implementation includes a span-aware lexer, Pratt parser, public AST, semantic analyzer, typed symbol checking, tree-walking interpreter, CLI, REPL, and native/WebAssembly build targets. The tree-walking interpreter is the compatibility foundation; native and WebAssembly code generation will consume the validated typed IR described in [`docs/semantic-ir-design.md`](docs/semantic-ir-design.md).

## Installation

Released binaries do not require Go or Rust. The repository currently publishes verified release archives; the installer scripts select the correct architecture and verify SHA-256 checksums.

### macOS and Linux

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/zyrndotio/Velari/main/scripts/install.sh | sh
velari version
```

Use `VELARI_PREFIX=/usr/local` with `sudo` when a system-wide installation is preferred:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/zyrndotio/Velari/main/scripts/install.sh | sudo VELARI_PREFIX=/usr/local sh
```

The same release artifacts can be installed directly with Linux package managers:

```bash
sudo apt install ./velari_<version>_amd64.deb
sudo dnf install ./velari-<version>-1.x86_64.rpm
sudo yum install ./velari-<version>-1.x86_64.rpm
```

Homebrew and signed APT/DNF repositories are distribution-layer integrations planned for the stable release channel. See [`packaging/README.md`](packaging/README.md) for the package-manager release contract and [`docs/product-goals.md`](docs/product-goals.md) for the packaging order and security requirements.

### Windows

Run PowerShell as the current user:

```powershell
irm https://raw.githubusercontent.com/zyrndotio/Velari/main/scripts/install.ps1 | iex
velari version
```

The installer places `velari.exe` in `%LOCALAPPDATA%\VelaRi\bin`, adds that directory to the user `PATH`, and registers VelaRi under the current user's Windows **Installed apps** list. The Installed apps entry includes an uninstaller. WinGet and Chocolatey manifests are planned from the same signed release assets; the direct PowerShell installer works before those catalog submissions are approved.

### Build from source

Building from source requires Go 1.22 or newer. Rust and Cargo are not required.

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
  velari help
  velari check <file>
  velari run <file>
  velari new <directory>
  velari init [directory]
  velari test [path]
  velari doctor
  velari --repl
```

`check` runs lexing, parsing, semantic analysis, and type checking. `run` performs the same validation before invoking the tree-walking interpreter. `new` and `init` create a minimal `velari.toml` project with a `main.vr` entry point. `test` executes every `.vr` program below a path and reports pass/fail results. `doctor` prints the installation, platform, and backend status.

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
See [`docs/product-goals.md`](docs/product-goals.md) for the performance, safety, readability, web, desktop, and distribution goals.

## Development

Format and test before opening a change:

```bash
gofmt -w cmd go scripts
go test ./...
```

Keep the core packages platform-neutral. Add semantic regressions under `go/semantic`, and add executable language examples or integration programs under `examples/` and `tests/`.
