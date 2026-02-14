# Maintainer: Taha Moosavi <taha.moosavi.taha@gmail.com>
pkgname=autopilot-rs
pkgver=0.1.0
pkgrel=1
pkgdesc="AutoPilot-rs runs automation jobs based on conditions like WiFi, Bluetooth, battery, CPU usage, and more."
arch=('x86_64')
url="https://github.com/streamtechteam/auto_pilot_rs"
license=('MIT')
depends=('systemd-libs', 'libxcb', 'libxrandr', 'libx11', 'libxau')
makedepends=('rust' 'cargo' 'libxcb' 'libxrandr' 'libxau' 'libx11')

source=()
sha256sums=()

build() {
  # Build from the directory where PKGBUILD is located
  cd "$startdir"
  cargo build --release --locked
}

package() {
  # Reference the binary using the absolute start directory
  install -Dm755 "$startdir/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

  # If you have a license file, it's good practice to include it:
  install -Dm644 "$startdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
