# Maintainer: Kumar Manisk <mathewmanisk699@gmail.com>
pkgname=rogtui-git
pkgver=0.1.0.r0.g$(git rev-parse --short HEAD 2>/dev/null || echo "0000000")
pkgrel=1
pkgdesc="Minimal, zero-overhead hardware control center for ASUS ROG and TUF gaming laptops"
arch=('x86_64')
url="https://github.com/kmanisk/rogtui"
license=('MIT' 'Apache-2.0')
depends=('gcc-libs' 'glibc')
makedepends=('cargo' 'git')
optdepends=(
    'asusd: ASUS system daemon for fan curves, profiles, and Aura RGB'
    'supergfxd: Dedicated GPU switching daemon'
)
provides=('rogtui')
conflicts=('rogtui')
source=("git+file://$(pwd)")
sha256sums=('SKIP')

pkgver() {
    cd "$srcdir/$pkgname" 2>/dev/null || cd "$startdir"
    printf "0.1.0.r%s.g%s" "$(git rev-list --count HEAD 2>/dev/null || echo 1)" "$(git rev-parse --short HEAD 2>/dev/null || echo "init")"
}

build() {
    cd "$srcdir/$pkgname" 2>/dev/null || cd "$startdir"
    export CARGO_TARGET_DIR="target"
    cargo build --release --locked
}

check() {
    cd "$srcdir/$pkgname" 2>/dev/null || cd "$startdir"
    cargo test --release --locked
}

package() {
    cd "$srcdir/$pkgname" 2>/dev/null || cd "$startdir"
    install -Dm755 target/release/rogtui "$pkgdir/usr/bin/rogtui"
    install -Dm644 README.md "$pkgdir/usr/share/doc/rogtui/README.md"
    install -Dm644 docs/endpoints.md "$pkgdir/usr/share/doc/rogtui/endpoints.md"
}
