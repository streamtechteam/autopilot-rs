# Maintainer: Taha Moosavi <taha.moosavi.taha@gmail.com>
pkgname=autopilot-rs
pkgver=0.1.0.beta
pkgrel=1
pkgdesc="AutoPilot-rs runs automation jobs based on conditions like WiFi, Bluetooth, battery, CPU usage, and more."
arch=('x86_64')
url="https://github.com/streamtechteam/autopilot-rs"
license=('GPL-3.0-or-later')
provides=("$pkgname")
conflicts=("$pkgname")
depends=('systemd-libs' 'libxcb' 'libxrandr' 'libx11' 'libxau')
makedepends=('rust' 'cargo' 'libxcb' 'libxrandr' 'libxau' 'libx11' 'git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/streamtechteam/autopilot-rs/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=("SKIP")

prepare() {
  cd "$srcdir/$pkgname-$pkgver"
  # Make the source tree self-contained so cargo does not escape into an
  # outer workspace (e.g. a CI checkout of the repository itself).
  if ! grep -q '^\[workspace\]' Cargo.toml; then
    printf '\n[workspace]\n' >> Cargo.toml
  fi
}

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  install -Dm755 "$srcdir/$pkgname-$pkgver/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

  install -Dm644 "$srcdir/$pkgname-$pkgver/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
