# Maintainer: Taha Moosavi <taha.moosavi.taha@gmail.com>
pkgname=autopilot-rs
pkgver=0.1.0
pkgrel=1
pkgdesc="AutoPilot-rs runs automation jobs based on conditions like WiFi, Bluetooth, battery, CPU usage, and more."
arch=('x86_64')
url="https://github.com/streamtechteam/auto_pilot_rs"
license=('MIT') 
depends=('systemd-libs')
makedepends=('rust' 'cargo')

build() {
  cargo build --release --locked
}

package() {
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}