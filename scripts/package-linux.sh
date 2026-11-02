#!/usr/bin/env sh
set -eu

VERSION="${1:-${VELARI_VERSION:-dev}}"
ARCH="${2:-amd64}"
case "$ARCH" in
  amd64) GOARCH=amd64; DEB_ARCH=amd64; RPM_ARCH=x86_64 ;;
  arm64) GOARCH=arm64; DEB_ARCH=arm64; RPM_ARCH=aarch64 ;;
  *) echo "unsupported Linux architecture: $ARCH" >&2; exit 2 ;;
esac
VERSION="$(printf '%s' "$VERSION" | sed 's/^v//; s/-go$//; s/[^0-9A-Za-z.+~]/./g')"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT
mkdir -p dist "$ROOT/deb/DEBIAN" "$ROOT/deb/usr/bin"
go build -trimpath -ldflags "-s -w -X main.version=$VERSION" -o "$ROOT/deb/usr/bin/velari" ./cmd/velari
chmod 0755 "$ROOT/deb/usr/bin/velari"
cat >"$ROOT/deb/DEBIAN/control" <<EOF
Package: velari
Version: $VERSION
Section: devel
Priority: optional
Architecture: $DEB_ARCH
Maintainer: VelaRi contributors <opensource@zyrndotio.dev>
Description: VelaRi programming language toolchain
 A readable, cross-platform compiler and interpreter with WebAssembly support.
EOF
dpkg-deb --build "$ROOT/deb" "dist/velari_${VERSION}_${DEB_ARCH}.deb"

if command -v rpmbuild >/dev/null 2>&1; then
  mkdir -p "$ROOT/rpm/BUILD" "$ROOT/rpm/BUILDROOT" "$ROOT/rpm/RPMS/$RPM_ARCH" "$ROOT/rpm/SOURCES" "$ROOT/rpm/SPECS" "$ROOT/rpm/SRPMS"
  cat >"$ROOT/rpm/SPECS/velari.spec" <<EOF
Name: velari
Version: $VERSION
Release: 1
Summary: VelaRi programming language toolchain
License: Apache-2.0
BuildArch: $RPM_ARCH

%description
A readable, cross-platform compiler and interpreter with WebAssembly support.

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 $ROOT/deb/usr/bin/velari %{buildroot}/usr/bin/velari

%files
/usr/bin/velari
EOF
  if [ "$RPM_ARCH" = "$(uname -m)" ]; then
    rpmbuild --define "_topdir $ROOT/rpm" -bb "$ROOT/rpm/SPECS/velari.spec" >/dev/null
  else
    rpmbuild --target "$RPM_ARCH" --define "_topdir $ROOT/rpm" -bb "$ROOT/rpm/SPECS/velari.spec" >/dev/null
  fi
  cp "$ROOT/rpm/RPMS/$RPM_ARCH/velari-$VERSION-1.$RPM_ARCH.rpm" "dist/velari-$VERSION-1.$RPM_ARCH.rpm"
fi
sha256sum dist/velari_${VERSION}_${DEB_ARCH}.deb > "dist/velari_${VERSION}_${DEB_ARCH}.deb.sha256"
if [ -f "dist/velari-$VERSION-1.$RPM_ARCH.rpm" ]; then sha256sum "dist/velari-$VERSION-1.$RPM_ARCH.rpm" > "dist/velari-$VERSION-1.$RPM_ARCH.rpm.sha256"; fi
