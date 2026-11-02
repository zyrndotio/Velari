# VelaRi product goals

## Positioning

VelaRi is intended to combine Rust-like performance and security with Go-like readability and approachability. The project should make the safe path the easy path: readable source, a small language core, fast startup, deterministic builds, and a toolchain that can be installed by ordinary developers and enterprise IT teams.

The current implementation is a Go compiler/interpreter foundation. Go is the implementation language for the toolchain; it is not automatically the complete safety model for programs written in VelaRi. VelaRi’s compiler must enforce its own type, memory, concurrency, resource, and capability rules rather than relying on a marketing claim that the implementation language makes every generated program safe.

## Product priorities

### 1. Performance

- Keep the language core small and compilation/startup fast.
- Lower checked programs to a typed IR before native or WebAssembly backends.
- Make WebAssembly a first-class web target.
- Keep allocation, I/O, GUI, and concurrency operations explicit in the runtime model.
- Benchmark lexer, parser, semantic analysis, interpreter startup, and generated workloads continuously.
- Avoid adding garbage-heavy abstractions to hot compiler paths without measurement.

The initial tree-walking interpreter is a development/runtime compatibility target, not the final performance target. Native and WebAssembly backends should consume the same validated typed IR.

### 2. Security and safety

- Reject type-invalid programs before execution.
- Use explicit bounds/type checks for arrays, maps, and external values.
- Keep filesystem, process, network, GUI, and environment access behind explicit host capabilities.
- Provide a sandboxable WebAssembly runtime boundary.
- Make unsafe/native escape hatches explicit, audited, and opt-in.
- Preserve source spans in diagnostics and build metadata.
- Add fuzzing and negative tests for the lexer, parser, semantic analyzer, and runtime.

VelaRi should aim for memory-safe default semantics and capability-oriented application APIs. “Rust-like security” is a design target that must be demonstrated with enforcement and tests, not just a slogan.

### 3. Readability

- Prefer a small number of obvious constructs over syntax-heavy features.
- Keep declarations, conditionals, loops, functions, and diagnostics easy to scan.
- Use clear compiler errors with source locations and actionable explanations.
- Keep the standard library discoverable and avoid implicit global behavior.
- Preserve a stable, readable textual IR for backend and debugging work.

### 4. Web applications

The web target is `GOOS=js GOARCH=wasm` for the Go host/runtime bridge and a future VelaRi-to-WASM backend. Browser-facing code should communicate through explicit host bindings rather than importing OS-specific packages into the language core.

### 5. Desktop applications

Desktop applications should use HTML/CSS/JavaScript frontends with a small Go host through Wails, or another supported webview host. The core language packages must remain independent of Wails/Tauri so they remain importable, testable, and cross-compilable.

## Supported installation targets

| Platform | Primary path | Intended audience |
|---|---|---|
| macOS | Homebrew formula or signed release archive | Developer machines and UI application development |
| Ubuntu/Debian | APT package or `.deb` release artifact | Cloud infrastructure, servers, containers |
| RHEL/CentOS/Fedora | DNF/YUM repository or `.rpm` release artifact | Enterprise infrastructure |
| Windows | WinGet manifest or Chocolatey package | Corporate fleets and DevOps automation |
| Any supported OS | Verified release archive and installer script | Early adopters and CI environments |

Package-manager publication requires repository ownership, signing keys, and release credentials. The repository should generate deterministic artifacts first; publishing to Homebrew, WinGet, Chocolatey, and Linux repositories is a distribution operation layered on top of those artifacts.

## Definition of installable

A tagged release is installable when:

1. It publishes checksummed archives for supported OS/architecture pairs.
2. It publishes a WebAssembly artifact separately from native binaries.
3. `scripts/install.sh` installs the correct macOS/Linux binary without Go or Rust.
4. `scripts/install.ps1` installs the correct Windows binary without Go or Rust.
5. A downloaded `.deb` installs with `sudo apt install ./velari-*.deb`.
6. A downloaded `.rpm` installs with `sudo dnf install ./velari-*.rpm` or `sudo yum install ./velari-*.rpm`.
7. Package-manager manifests are versioned from the same release tag and checksums.
8. CI tests the installed binary with `velari version` and `velari check examples/basic.vr`.

## Release order

1. GitHub release archives plus SHA-256 checksums.
2. Native shell and PowerShell installers.
3. `.deb` and `.rpm` assets for direct package-manager installation.
4. Homebrew tap/formula.
5. WinGet manifest submitted to `microsoft/winget-pkgs`.
6. Chocolatey package published to the community feed.
7. Signed APT and DNF repositories when stable release cadence and signing infrastructure exist.
