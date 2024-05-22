# Kontroll
Kontroll demonstates how to control the Keymapp API, making it easy to control your ZSA keyboard from the command line and scripts.

## Installation
If you have the rust toolchain installed, you can install Kontroll with the following command, clone this repository and run the following command:
```bash
cargo install kontroll
```
Otherwise, you can download the latest release from the [releases page](https://github.com/zsa/kontroll/releases) and add it to your PATH.

## Prerequisites
Make sure you have a recent version of Keymapp running with a ZSA keyboard connected to your computer. In Keymapp's config page, make sure the Keymapp API is enabled. By default, the Keymapp API listens on port `50051`. If you have changed the port, you can specify it by setting the `KEYMAPP_PORT` environment variable.

## Usage
```bash
Commands:
  list                 List all available keyboards
  connect              Connect to a keyboard given the index returned by the list command
  connect-any          Connect to the first keyboard detected by keymapp
  set-layer            Set the layer of the currently connected keyboard
  set-rgb              Sets the RGB color of a LED
  set-rgb-all          Sets the RGB color of all LEDs
  set-status-led       Set / Unset a status LED
  increase-brightness  Increase the brightness of the keyboard's LEDs
  decrease-brightness  Decrease the brightness of the keyboard's LEDs
  disconnect           Disconnect from the currently connected keyboard
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Development
We developped Kontroll to showcase the Keymapp API and to provide a simple way to control your ZSA keyboard from the command line and scripts.
If you wish to build your own client, you need to implement the Keymapp API using gRPC and the protobuf file [available here](proto/keymapp.proto).
An example on how to implement each endpoint can be found in the [api.rs](src/api.rs) file.
