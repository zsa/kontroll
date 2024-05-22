# Kontroll
Kontroll demonstates how to control the Keymapp API, making it easy to control your ZSA keyboard from the command line and scripts.

You can use it to switch layers and control the keyboard's RGB and status LEDs programmatically via Keymapp. Call it from a script to make your ZSA keyboard react to system events in useful ways (switching to layers when apps are activated, or changing lighting when you get an important email, etc).

Feel free to submit scripts that use Kontroll as pull requests via the [examples](examples/) directory.

## Installation
If you have the rust toolchain installed, you can build Kontroll by cloning this repository and running the following command:
```bash
cargo build --release
```
Otherwise, you can download the latest release from the [releases page](https://github.com/zsa/kontroll/releases) and add it to your PATH.

## Prerequisites
Make sure you have a recent version of Keymapp running with a ZSA keyboard connected to your computer. In Keymapp's config page, make sure the the API is enabled. By default, the API listens on port `50051`. If you have changed the port in the UI, you can specify the port to Kontroll by setting the `KEYMAPP_PORT` environment variable.

## Usage
```
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
We developed Kontroll to showcase the Keymapp API and to provide a simple way to control your ZSA keyboard from the command line and scripts.

If you wish to build your own client, you need to implement the Keymapp API using gRPC. The protobuf file [available here](proto/keymapp.proto) describes all the remote call procedures and messages available.

An example on how to implement each procedure call can be found in the [api.rs](src/api.rs) file.
