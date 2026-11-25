# Dependencies

## Cargo

To install the rust toolchain for building the project go to
[rust-lang.org](https://www.rust-lang.org/tools/install) and follow the instructions.
Kontroll needs Rust 1.88 or newer.

## Linux

The USB side builds hidapi's C code against libudev, so it needs a C compiler,
`pkg-config` and the libudev development files:

- Debian / Ubuntu: `sudo apt install build-essential pkg-config libudev-dev`
- Fedora: `sudo dnf install gcc pkgconf-pkg-config systemd-devel`
- Arch: `sudo pacman -S base-devel pkgconf systemd-libs`

Bluetooth goes through BlueZ over D-Bus in pure Rust and needs nothing extra
to build; BlueZ must be running to use it.

To open the keyboard without root, install ZSA's udev rules from
[zapp](https://github.com/zsa/zapp/tree/main/udev):

```bash
sudo cp 50-zsa.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## macOS & Windows

Nothing beyond the Rust toolchain (and the platform's C compiler, which comes
with Xcode's command line tools on macOS and with Visual Studio's build tools on
Windows).
