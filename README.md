# VelaRi

VelaRi is a Rust-based programming language and compiler foundation for safe, fast, cross-platform native desktop applications. The project is designed to grow from a small, readable language core into a production-quality toolchain targeting Windows, Linux, and macOS on x86_64 and ARM64.

> **Project status:** early development. The current release contains a working lexer, recoverable parser, structured diagnostics, typed semantic-analysis pass, scoped interpreter, functions, and CLI. Native LLVM code generation, package management, and the modular desktop runtime are planned roadmap work.

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
- Lexical scopes, assignments, functions, function calls, and return statements
- Array and map literals, indexing, and the built-in `length` function
- Human-readable runtime values and executable `assert` test statements
- Explicit type annotations for primitives and generic collection types
- Typed intermediate representation inspection with `velari ir`
- Windows release automation and Rust-native installation tooling
- Desktop project syntax preview and build validation groundwork
- A cross-platform Rust CLI for checking, running, testing, creating projects, and reporting the version

## Install from source

Install a current stable Rust toolchain with [rustup](https://rustup.rs/), then clone and build the repository:

```bash
git clone https://github.com/zyrndotio/Velari.git
cd Velari
cargo test
cargo build --release
```

The compiled binary is placed at `target/release/velari` on Unix-like systems and `target/release/velari.exe` on Windows.

On Windows, a locally built executable is not automatically installed as a global command. Run it from the project directory with:

```text
.\target\release\velari.exe version
.\target\release\velari.exe check examples\basic.vr
.\target\release\velari.exe run examples\basic.vr
```

To use `velari` directly from any Windows terminal, copy `target\release\velari.exe` into a directory on your `PATH`, or add the release directory to your user `PATH`, then open a new terminal. Rust's `cargo install --path .` is another option; Cargo installs the binary into `%USERPROFILE%\.cargo\bin`, which rustup normally adds to `PATH`.

## Download a release

Prebuilt installers and portable release assets are published on the [GitHub Releases page](https://github.com/zyrndotio/Velari/releases). Stable and beta releases provide a Windows `Velari-Setup.exe` installer, Linux `.deb` and `.AppImage` packages, and a macOS `.dmg` package when the corresponding platform workflow succeeds. Portable Linux and Windows archives remain available as fallback downloads, and every release asset includes a SHA-256 checksum.


## Windows installation

Download `Velari-Setup.exe` from the [latest GitHub release](https://github.com/zyrndotio/Velari/releases) to install VelaRi on Windows. The setup program installs the compiler for the user and creates the application entry points provided by the installer. Verify the installation with:

```text
velari version
velari system
```

Windows, Linux, and macOS release installers are produced by GitHub Actions for version tags. The current `build` command validates source and project configuration. The `package` command creates a validated desktop package manifest; standalone native executable generation remains future backend work.

## Command-line usage

Run the compiler through Cargo while developing:

```bash
cargo run -- version
cargo run -- check examples/basic.vr
cargo run -- run examples/basic.vr
cargo run -- new hello-world
cargo run -- check hello-world/src/main.vr
cargo run -- test hello-world
cargo run -- build hello-world
cargo run -- package hello-world
cargo run -- system
cargo run -- ir examples/typed.vr
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
| `velari new <directory>` | Create a starter project containing `vela.toml`, `src/main.vr`, and a `tests/` directory. |
| `velari test [project]` | Run every `.vr` file under the project `tests/` directory. |
| `velari build [project]` | Validate a project for future native compilation; native `.exe` generation is not available yet. |
| `velari package [project]` | Validate desktop metadata and write a package manifest under `target/package`. |
| `velari system` | Report the VelaRi version and host platform information. |
| `velari install` | Atomically install or replace the current binary in the user VelaRi installation directory. |
| `velari where` | Print the expected user installation path. |
| `velari uninstall` | Remove the user-installed VelaRi binary if it exists. |

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
20
hello VelaRi
true
19
19
```

## Syntax overview

A source file is enclosed by `begin` and `end`. Variables use the readable `let ... be ...` form:

```velari
begin
    let enabled be true
    let answer be 42
    let message be "VelaRi"
    print message
    assert enabled
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

## Explicit types and IR

VelaRi 0.4.5-beta.2 accepts explicit primitive and collection annotations:

```velari
begin
    let count: Int be 3
    let label: String be "VelaRi"
    let values: Array<Int> be [1, 2, 3]
end
```

Function parameters and return values can also be annotated. The compiler validates declarations before execution and rejects incompatible initializers. Use `velari ir <file>` to inspect the current typed intermediate representation, which is the foundation for future native backends.

## Collections

Arrays and maps are available in the interpreter:

```velari
begin
    let numbers be [1, 2, 3, 4]
    let user be {"name": "VelaRi", "age": 3}

    print numbers[2]
    print user["name"]
    print length(numbers)
end
```

Arrays use numeric indexes and maps use string keys. Out-of-bounds indexes and missing keys produce runtime errors. `length` accepts strings, arrays, and maps.

## Project layout

```text
.
├── Cargo.toml          # Rust package metadata
├── LICENSE             # Apache License 2.0
├── NOTICE              # Copyright and license attribution
├── README.md           # Project documentation
├── examples/
│   ├── desktop/main.vr  # Desktop syntax preview
│   ├── basic.vr        # End-to-end language example
│   ├── typed.vr        # Explicit type annotations
│   └── typed-functions.vr # Typed function signatures
│   └── functions.vr    # Functions and return values
└── src/
    ├── ast.rs          # Typed abstract syntax tree
    ├── diagnostics.rs  # Structured source diagnostics
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

## Desktop projects

Desktop projects now receive an explicit manifest section:

```toml
[package]
name = "desktop-app"
version = "0.1.0"
entry = "src/main.vr"

[desktop]
backend = "auto"
```

The `new` command creates `src`, `tests`, and `assets` directories. The `package` command validates the manifest and writes `target/package/manifest.txt`. This is packaging groundwork, not native executable generation; the future desktop runtime and native backend will consume this metadata.

## Project discovery and tests

When `check`, `run`, or `test` receives a directory, VelaRi searches that directory and its parents for `vela.toml`. The manifest entry defaults to `src/main.vr` when no `entry` value is present. A generated project includes an empty `tests/` directory so project-level tests can be added immediately.

Place executable VelaRi test programs under `tests/`:

```text
my-project/
├── vela.toml
├── src/main.vr
└── tests/
    └── smoke.vr
```

Run them with `velari test` from the project directory or with `velari test path/to/project`. Each `.vr` file must parse, pass semantic analysis, and execute successfully. Test files can use `assert condition`; a false assertion fails the test with a non-zero exit status.

## VelaRi 0.4.2 release

VelaRi 0.4.2 is a distribution and tooling milestone. It adds safer replacement during `velari install`, plus `velari where` and `velari uninstall` so local installations can be inspected and removed without manually editing the installation directory. Release archives are built by GitHub Actions for Linux x86_64 and Windows x86_64 tags.

The installer copies the running binary to `~/.velari/bin/velari` on Unix-like systems and `%USERPROFILE%\\.velari\\velari.exe` on Windows. The destination directory must be on `PATH` for the `velari` command to be available globally. The command reports the path after installation rather than changing shell startup files automatically.

## VelaRi 0.4.5-beta.2 stable release

VelaRi 0.4.5-beta.2 is the packaging and release-readiness milestone built on the 0.4.3 semantic typing foundation. It includes packager metadata for the VelaRi product identifier, installer-oriented release configuration, a square application icon for AppImage generation, and the audited issue record in `ISSUES.md`. The release is not yet the native application backend; the compiler still executes programs through the validated tree-walking interpreter.

The stable release provides a Windows `Velari-Setup.exe`, Linux `.deb` and `.AppImage` packages, and a macOS `.dmg`, alongside portable archives and SHA-256 checksums.

## Next release: VelaRi 0.4.5

The 0.4.5 milestone will focus on developer workflow and runtime reliability before native code generation. Planned work includes a first-class `velari fmt` command, richer diagnostics with source excerpts and actionable suggestions, broader project manifest validation, more complete collection operations and mutation rules, and expanded control-flow and function regression coverage. The release will also improve installer smoke tests and checksum verification in CI.

Native executable generation remains planned for the 0.5.x line rather than being added directly to the current interpreter-oriented package command.

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

- Arrays, maps, indexing, iteration, and mutation rules
- Explicit type annotations and broader type inference
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
- Add installer discovery, uninstall support, and signed release artifacts
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

## Release status

VelaRi 0.4.5-beta.2 is the stable packaging and release-readiness milestone. VelaRi 0.4.3 remains the semantic typing foundation: collection element and map value types propagate through semantic analysis, function calls validate annotated parameters, and return values are checked against annotated function results. The typed interpreter and backend-neutral IR remain the current execution foundation; native executable generation is planned for the 0.5.x line. The next planned release is 0.4.5, focused on formatting, diagnostics, project validation, collection operations, and release-quality CI.

## License

VelaRi is licensed under the **Apache License, Version 2.0**. You may use, modify, distribute, and commercially use the project under the terms of that license. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright 2026 zyrndotio.
