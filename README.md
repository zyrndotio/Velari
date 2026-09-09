# VelaRi

VelaRi is a small, safety-oriented programming language and compiler foundation for fast, cross-platform native desktop applications. The compiler is written in Rust and is being developed around a clear pipeline: **lexer → parser → AST → semantic analysis → runtime/backend**.

> VelaRi is an early-stage language project. The current release provides a working typed front end and interpreter; native LLVM code generation and the broader desktop runtime are planned next.

## Current capabilities

- Span-aware lexer with recoverable diagnostics
- Comments and escaped string literals
- Numbers, booleans, strings, and a null type foundation
- Pratt-style expression parsing with operator precedence
- Arithmetic, equality, logical operators, unary negation, and parentheses
- Static semantic checks for variable initialization and type compatibility
- Fallible interpreter with conditionals, repeat loops, printing, and window declarations
- Cross-platform Rust CLI commands for checking, running, creating projects, and displaying the version

## Quick start

Install Rust through [rustup](https://rustup.rs/), then build and test:

```bash
cargo test
cargo build --release
```

Check or run a VelaRi source file:

```bash
cargo run -- check examples/basic.vr
cargo run -- run examples/basic.vr
```

Create a new project:

```bash
cargo run -- new hello-world
cargo run -- run hello-world/src/main.vr
```

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

The parser respects normal precedence, so multiplication is evaluated before addition.

## CLI

| Command | Purpose |
| --- | --- |
| `velari check <file>` | Lex, parse, and semantically validate a source file |
| `velari run <file>` | Validate and execute a source file with the interpreter |
| `velari new <directory>` | Create a starter project with `vela.toml` and `src/main.vr` |
| `velari version` | Print the compiler version |

## Project layout

```text
src/
├── ast.rs          # Typed abstract syntax tree
├── interpreter.rs  # Fallible tree-walking runtime
├── lexer.rs        # Tokens, spans, and lexical diagnostics
├── main.rs         # CLI and compiler pipeline
├── parser.rs       # Recoverable Pratt parser
└── semantic.rs     # Static type and symbol checks
examples/
└── basic.vr       # End-to-end language example
```

## Roadmap

The next development phases are native LLVM code generation with explicit target mappings, functions and local scopes, arrays and maps, a structured diagnostic renderer, manifest loading, and modular cross-platform runtime services. The project will preserve the current front-end architecture while adding those capabilities incrementally.

## Contributing

Contributions are welcome. Please keep changes focused, add regression tests for language behavior, and run `cargo test` before opening a pull request.

## License

VelaRi is available under the MIT License. See [LICENSE](LICENSE).
