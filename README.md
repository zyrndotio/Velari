# VelaRi

VelaRi is a Rust-based programming language and compiler foundation for safe, fast, cross-platform native desktop applications. The project is designed to grow from a small, readable language core into a production-quality toolchain targeting Windows, Linux, and macOS on x86_64 and ARM64.

> **Project status:** early development. The current release contains a working lexer, recoverable parser, typed semantic-analysis pass, interpreter, CLI, and example program. Native LLVM code generation, package management, and the modular desktop runtime are planned roadmap work.

[![Rust](https://img.shields.io/badge/built_with-Rust-dea584.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Why VelaRi?

VelaRi is being developed around a few core goals:

- **Native performance:** compile to efficient native applications rather than relying on a large virtual machine.
- **Compile-time safety:** catch invalid operations and uninitialized variables before execution.
- **Memory safety:** use Rust for the compiler and design the runtime around explicit, auditable ownership boundaries.
- **Developer experience:** provide readable source syntax, useful diagnostics, and a simple project workflow.
- **Cross-platform applications:** keep target-specific behavior behind platform abstractions instead of hardcoding one operating system.
- **Secure capabilities:** make filesystem, networking, process, and other privileged services explicit runtime capabilities.

## Current release capabilities

The current codebase provides a coherent front-end and interpreter foundation:

- Span-aware lexical tokens with line, column, and byte offsets
- Recoverable lexer diagnostics instead of panics for malformed input
- Comments beginning with `#`
- String literals with basic `\\n`, `\\t`, quote, and backslash escapes
- Number, boolean, string, and null type foundations
- Pratt-style expression parsing with conventional precedence
- Unary negation, arithmetic, equality, logical `and`, and logical `or`
- Parenthesized expressions
- Static checks for variable initialization and type-compatible operations
- String concatenation
- `if` / `otherwise` conditionals
- `repeat ... times` loops
- Printing and window-declaration syntax in the interpreter
- A cross-platform Rust CLI for checking, running, creating projects, and reporting the version

## Install from source

Install a current stable Rust toolchain with [rustup](https://rustup.rs/), then clone and build the repository:

```bash
git clone https://github.com/zyrndotio/Velari.git
cd Velari
cargo test
cargo build --release
```

The compiled binary is placed at `target/release/velari` on Unix-like systems and `target/release/velari.exe` on Windows.

On Windows PowerShell, a locally built executable is not automatically installed as a global command. Run it from the project directory with:

```powershell
.\target\release\velari.exe version
.\target\release\velari.exe check examples\basic.vr
.\target\release\velari.exe run examples\basic.vr
```

To use `velari` directly from any PowerShell window, copy `target\release\velari.exe` into a directory on your `PATH`, or add the release directory to your user `PATH`, then open a new terminal. Rust's `cargo install --path .` is another option; Cargo installs the binary into `%USERPROFILE%\.cargo\bin`, which rustup normally adds to `PATH`.

## Download a release

Prebuilt source archives and release assets are published on the [GitHub Releases page](https://github.com/zyrndotio/Velari/releases). The source archive can also be built on any supported Rust host with the commands above.

## Command-line usage

Run the compiler through Cargo while developing:

```bash
cargo run -- version
cargo run -- check examples/basic.vr
cargo run -- run examples/basic.vr
cargo run -- new hello-world
cargo run -- check hello-world/src/main.vr
```

After installing the release binary, use the same commands without `cargo run --`:

```bash
velari version
velari check examples/basic.vr
velari run examples/basic.vr
velari new hello-world
```

| Command | Description |
| --- | --- |
| `velari version` | Print the compiler version. |
| `velari check <file>` | Lex, parse, and semantically validate a VelaRi source file without executing it. |
| `velari run <file>` | Validate and execute a source file with the tree-walking interpreter. |
| `velari new <directory>` | Create a starter project containing `vela.toml` and `src/main.vr`. |

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

The parser applies normal precedence, so `10 + 5 * 2` evaluates as `10 + (5 * 2)`. The example prints values equivalent to:

```text
Number(20.0)
String("hello VelaRi")
Boolean(true)
Number(19.0)
Number(19.0)
```

## Syntax overview

A source file is enclosed by `begin` and `end`. Variables use the readable `let ... be ...` form:

```velari
begin
    let enabled be true
    let answer be 42
    let message be "VelaRi"
    print message
end
```

Supported expression operators, from lower to higher precedence, are:

| Precedence | Operators | Meaning |
| ---: | --- | --- |
| 1 | `or` | Boolean disjunction |
| 2 | `and` | Boolean conjunction |
| 3 | `is`, `==` | Equality comparison |
| 4 | `+`, `-` | Addition and subtraction; string concatenation for two strings |
| 5 | `*`, `/` | Multiplication and division |
| unary | `-` | Numeric negation |

## Project layout

```text
.
├── Cargo.toml          # Rust package metadata
├── LICENSE             # Apache License 2.0
├── NOTICE              # Copyright and license attribution
├── README.md           # Project documentation
├── examples/
│   └── basic.vr        # End-to-end language example
└── src/
    ├── ast.rs          # Typed abstract syntax tree
    ├── interpreter.rs  # Fallible tree-walking runtime
    ├── lexer.rs        # Tokens, spans, and lexical diagnostics
    ├── main.rs         # CLI and compiler pipeline
    ├── parser.rs       # Recoverable Pratt parser
    └── semantic.rs     # Static type and symbol checks
```

## Compiler architecture

The current pipeline is intentionally small and modular:

```text
VelaRi source
    │
    ▼
Lexer ──► tokens with source spans
    │
    ▼
Parser ──► AST with operator precedence
    │
    ▼
Semantic analysis ──► type and symbol validation
    │
    ▼
Interpreter ──► executable behavior during early development
```

The interpreter is a development backend, not the final deployment model. The planned native backend will consume the same validated front end and lower VelaRi types to target-specific native representations.

## Development workflow

Run the complete test suite:

```bash
cargo test
```

Run the example:

```bash
cargo run -- run examples/basic.vr
```

Build an optimized compiler binary:

```bash
cargo build --release
```

When adding language behavior, include a focused regression test and update the example or documentation when syntax changes. Keep diagnostics user-facing and avoid `panic!` for invalid source programs.

## Roadmap

### Front end

- Structured diagnostic rendering with error codes and source snippets
- Explicit type annotations and broader type inference
- Functions, parameters, return values, local scopes, and recursion
- Arrays, maps, indexing, iteration, and mutation rules
- Structs, enums, option/result types, generics, and traits

### Native compilation

- Reintroduce LLVM code generation behind a stable backend interface
- Map VelaRi types to explicit native types instead of representing everything as floating point
- Add target abstractions for Windows x86_64/ARM64, Linux x86_64/ARM64, and macOS x86_64/ARM64
- Add native linking and release/debug optimization controls

### Tooling and projects

- Fully load and validate `vela.toml` manifests
- Add `build`, `test`, `fmt`, `clean`, and dependency commands
- Add package and standard-library management
- Provide editor integration, formatting, and language-server support

### Runtime and desktop platform

- Modular memory, strings, collections, filesystem, networking, process, threading, and timer services
- Explicit permission and capability configuration
- Platform-neutral windowing and event APIs with Windows, macOS, and Linux backends
- Native graphics, audio, and desktop application integrations

## Contributing

Contributions are welcome. Before opening a pull request:

1. Keep the change focused and explain the user-visible behavior.
2. Add or update regression coverage.
3. Run `cargo test` locally.
4. Update the README when commands or language syntax change.
5. Do not commit generated `target/` artifacts or secrets.

Please use GitHub Issues for bugs and feature proposals, and include a minimal VelaRi source example when reporting parser, semantic, or runtime behavior.

## License

VelaRi is licensed under the **Apache License, Version 2.0**. You may use, modify, distribute, and commercially use the project under the terms of that license. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright 2026 zyrndotio.
