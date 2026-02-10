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

# We leave 'source' empty because we are building from the local checked-out directory
source=()
sha256sums=()

build() {
  # We assume we are already in the root of the repo
  cargo build --release --locked
}

package() {
  # Install the binary to /usr/bin/
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  
  # Optional: If you have a systemd service file, uncomment the line below:
  # install -Dm644 "autopilot.service" "$pkgdir/usr/lib/systemd/system/autopilot.service"
}