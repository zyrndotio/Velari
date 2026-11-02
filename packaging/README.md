# Package-manager distribution

The release workflow is the source of truth for binaries and SHA-256 checksums. It publishes:

- `velari-<os>-<arch>.tar.gz` for macOS and Linux;
- `velari-windows-amd64.zip` for Windows;
- `velari_<version>_<arch>.deb` for APT-compatible systems;
- `velari-<version>-1.<arch>.rpm` for DNF/YUM-compatible systems;
- a separate `velari.wasm` artifact can be added when the browser runtime entry point is ready.

## Direct package installation

These work without Go or Rust after downloading a release asset:

```bash
sudo apt install ./velari_<version>_amd64.deb
sudo dnf install ./velari-<version>-1.x86_64.rpm
sudo yum install ./velari-<version>-1.x86_64.rpm
```

Direct package installation is intentionally supported before a public repository is created. The package files and checksums are uploaded to every tagged GitHub release.

## Homebrew

Publish a formula in a tap such as `zyrndotio/homebrew-velari` using the macOS release archives and their checksums. The formula should select `darwin-arm64` on Apple Silicon and `darwin-amd64` on Intel. The intended user command is:

```bash
brew tap zyrndotio/velari
brew install velari
```

Do not publish a formula until the tagged release URL and SHA-256 values are fixed; Homebrew refuses mutable or unverifiable source archives.

## WinGet

Submit a versioned manifest to `microsoft/winget-pkgs` using the Windows ZIP URL and its SHA-256 checksum. The intended user command is:

```powershell
winget install zyrndotio.VelaRi
```

The manifest should install `velari.exe` and expose it on the machine or user `PATH`.

## Chocolatey

Publish a package that downloads the Windows ZIP from the tagged GitHub release, verifies its checksum, and installs `velari.exe` into the Chocolatey tools directory. The intended user command is:

```powershell
choco install velari
```

Chocolatey and WinGet are separate catalogs; publishing one does not publish the other.

## Repository-backed APT/DNF channels

A stable corporate channel needs package signing keys, a hosted repository, and a key-rotation policy. Until that infrastructure exists, the signed/checksummed `.deb` and `.rpm` release assets provide a transparent `apt install ./...` and `dnf install ./...` path without pretending that GitHub Releases is an APT/DNF repository.
