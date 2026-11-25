# Kontroll [![crates.io](https://img.shields.io/crates/v/kontroll.svg)](https://crates.io/crates/kontroll)

Kontroll controls your ZSA keyboard from the command line and scripts. It talks to the keyboard directly, over USB or, for the Wireless Voyager, over Bluetooth. You can read more about it on [our blog](https://blog.zsa.io/introducing-kontroll/).

You can use it to switch layers and control the keyboard's RGB and status LEDs programmatically. Call it from a script to make your ZSA keyboard react to system events in useful ways (switching to layers when apps are activated, or changing lighting when you get an important email, etc).

Note: All features of Kontroll work on the Moonlander and the Voyager. Only some features work on the ErgoDox EZ, since that board has a different processor (Teensy). You're able to use Kontroll to switch layers (which is really the crucial bit) and to set the status LEDs, but you will not be able to control the RGB LEDs of the ErgoDox EZ.

Feel free to submit scripts that use Kontroll as pull requests via the [examples](examples/) directory.

## Installation

If you have the [rust toolchain installed](/dependencies.md), you can build Kontroll by cloning this repository and running the following command:

```bash
cargo build --release
```

Bluetooth support is on by default. To build without it, run `cargo build --release --no-default-features`.

Otherwise, you can download the latest release from the [releases page](https://github.com/zsa/kontroll/releases) and add it to your PATH.

## Prerequisites

Kontroll 2 no longer needs Keymapp. Plug in a ZSA keyboard running firmware built with Oryx, or bond a Wireless Voyager over Bluetooth in your system's Bluetooth settings.

On Linux, install ZSA's [udev rules](https://github.com/zsa/zapp/tree/main/udev) so Kontroll can open the keyboard without root:

```bash
sudo cp 50-zsa.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

The keyboard keeps one live session, on the connection (USB or Bluetooth) of the last app that opened it. Layer changes are only accepted from that session, and live key and layer events only go to it. Each Kontroll command opens that session on its own connection. If Keymapp 2.x is showing the same keyboard live over the other connection, Kontroll takes the session over from it.

## Upgrading from Kontroll 1.x

- Each command opens the keyboard, does its work and closes it. `connect`, `connect-any` and `disconnect` are kept so old scripts still run, but they do nothing. Pick a keyboard with `--keyboard <INDEX>` (from `kontroll list`) on any command; without it the first keyboard found is used, USB first.
- `--port` is ignored.
- `--sustain <ms>` behaves as it did with Keymapp: the LEDs are restored after that many milliseconds. Kontroll now waits for that time before exiting, since there is no Keymapp to do it in the background.
- Two identical USB keyboards (same model) cannot be told apart: only the first of them can be controlled.
- `status` no longer reports a Keymapp version, and now reports the transport and the firmware's protocol version.

## Usage

```cli
Usage: kontroll [OPTIONS] <COMMAND>

Commands:
  status               Get the status of the keyboard
  list                 List all available keyboards
  connect              Deprecated, does nothing: use --keyboard <INDEX> instead
  connect-any          Deprecated, does nothing: the first keyboard found is used by default
  set-layer            Set the layer of the keyboard
  set-rgb              Sets the RGB color of a LED
  set-rgb-all          Sets the RGB color of all LEDs
  restore-rgb-leds     Restores the RGB color of all LEDs to their default
  set-status-led       Set / Unset a status LED
  restore-status-leds  Restores the status of all status LEDs to their default
  increase-brightness  Increase the brightness of the keyboard's LEDs
  decrease-brightness  Decrease the brightness of the keyboard's LEDs
  disconnect           Deprecated, does nothing: Kontroll closes the keyboard after each command
  help                 Print this message or the help of the given subcommand(s)

Options:
  -k, --keyboard <INDEX>  Keyboard to control, by its index from `kontroll list` (default: the first one found)
  -h, --help              Print help
  -V, --version           Print version
```

## Examples

Under [examples](/examples/README.md) is a showcase on how you could use the API.
Run the examples by having the rust toolchain installed and run the following command:

```bash
cargo run --release --example snake
```

## Development

Kontroll talks to the keyboard's raw HID protocol (usage page 0xFF60) through [zapp](https://github.com/zsa/zapp)'s libraries: `zapp-core` for USB, `zapp-ble` for Bluetooth. The keyboard side of the protocol lives in ZSA's QMK fork (`modules/zsa/oryx/oryx.c`).

If you are using the Rust programming language, you may use the [Kontroll crate](https://crates.io/crates/kontroll). See [api.rs](src/api.rs) for the available methods.
