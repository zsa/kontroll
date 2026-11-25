use clap::{Parser, Subcommand};
use std::process::exit;
use std::thread;
use std::time::Duration;

use kontroll::{utils, ApiError, Kontroll};

#[derive(Debug, Parser)]
#[command(name = "Kontroll", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Kontroll controls your ZSA keyboard from the command line and scripts: layers, RGB and status LEDs, and brightness, over USB or Bluetooth.", long_about = None)]
pub struct Cli {
    /// Keyboard to control, by its index from `kontroll list` (default: the first one found)
    #[arg(short, long, global = true, value_name = "INDEX")]
    keyboard: Option<usize>,
    /// Deprecated and ignored: Kontroll no longer talks to Keymapp
    #[arg(short, long, global = true, hide = true)]
    port: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(about = "Get the status of the keyboard")]
    Status {
        #[arg(short, long, default_value = "false")]
        json: bool,
    },
    #[command(about = "List all available keyboards")]
    List,
    #[command(about = "Deprecated, does nothing: use --keyboard <INDEX> instead")]
    Connect {
        #[arg(short, long, name = "keyboard index", required = true)]
        index: usize,
    },
    #[command(about = "Deprecated, does nothing: the first keyboard found is used by default")]
    ConnectAny,
    #[command(about = "Set the layer of the keyboard")]
    SetLayer {
        #[arg(short, long, required = true)]
        index: u8,
    },
    #[command(about = "Sets the RGB color of a LED")]
    SetRGB {
        #[arg(short, long, required = true)]
        led: usize,
        #[arg(short, long, required = true)]
        color: String,
        /// Restore the LEDs after this many milliseconds, waiting until then (0: keep the color)
        #[arg(short, long, default_value = "0")]
        sustain: i32,
    },
    #[command(about = "Sets the RGB color of all LEDs")]
    SetRGBAll {
        #[arg(short, long, required = true)]
        color: String,
        /// Restore the LEDs after this many milliseconds, waiting until then (0: keep the color)
        #[arg(short, long, default_value = "0")]
        sustain: i32,
    },
    #[command(about = "Restores the RGB color of all LEDs to their default")]
    RestoreRGBLeds {},
    #[command(about = "Set / Unset a status LED")]
    SetStatusLed {
        #[arg(short, long, required = true)]
        led: u8,
        #[arg(short, long)]
        off: bool,
        /// Restore the status LEDs after this many milliseconds, waiting until then (0: keep the state)
        #[arg(short, long, default_value = "0")]
        sustain: i32,
    },
    #[command(about = "Restores the status of all status LEDs to their default")]
    RestoreStatusLeds {},
    #[command(about = "Increase the brightness of the keyboard's LEDs")]
    IncreaseBrightness {
        #[arg(short, long, default_value = "1")]
        steps: i32,
    },
    #[command(about = "Decrease the brightness of the keyboard's LEDs")]
    DecreaseBrightness {
        #[arg(short, long, default_value = "1")]
        steps: i32,
    },
    #[command(about = "Deprecated, does nothing: Kontroll closes the keyboard after each command")]
    Disconnect,
}

pub fn run() {
    let cli = Cli::parse();

    if cli.port.is_some() {
        eprintln!("Warning: --port is deprecated and ignored, Kontroll no longer talks to Keymapp.");
    }

    if let Err(e) = execute(cli.command, cli.keyboard) {
        eprintln!("{e}");
        exit(1);
    }
}

/// Runs one command. The keyboard is closed before this returns.
fn execute(command: Commands, keyboard: Option<usize>) -> Result<(), ApiError> {
    match command {
        Commands::Connect { .. } | Commands::ConnectAny | Commands::Disconnect => {
            eprintln!(
                "Warning: connect, connect-any and disconnect are deprecated and do nothing. \
                 Each command opens the keyboard itself; pick one with --keyboard <INDEX> \
                 (see `kontroll list`)."
            );
        }
        Commands::List => {
            let keyboards = Kontroll::list()?;
            if keyboards.is_empty() {
                return Err(ApiError::NoKeyboard);
            }
            for keyboard in keyboards {
                println!("{keyboard}");
            }
        }
        Commands::Status { json } => {
            let status = Kontroll::open(keyboard)?.status();
            if json {
                println!("{}", serde_json::to_string_pretty(&status).unwrap());
            } else {
                print!("{status}");
            }
        }
        Commands::SetLayer { index } => {
            Kontroll::open(keyboard)?.set_layer(index)?;
            println!("Layer set to {index}");
        }
        Commands::SetRGB {
            led,
            color,
            sustain,
        } => {
            let (r, g, b) = parse_color(&color)?;
            let mut api = Kontroll::open(keyboard)?;
            api.set_rgb_led(led, r, g, b)?;
            println!("LED {led} set to color {color}");
            if hold(sustain) {
                api.restore_rgb_leds()?;
            }
        }
        Commands::SetRGBAll { color, sustain } => {
            let (r, g, b) = parse_color(&color)?;
            let mut api = Kontroll::open(keyboard)?;
            api.set_rgb_all(r, g, b)?;
            println!("All LEDs set to color {color}");
            if hold(sustain) {
                api.restore_rgb_leds()?;
            }
        }
        Commands::RestoreRGBLeds {} => {
            Kontroll::open(keyboard)?.restore_rgb_leds()?;
            println!("All LEDs restored to their default color");
        }
        Commands::SetStatusLed { led, off, sustain } => {
            let on = !off;
            let mut api = Kontroll::open(keyboard)?;
            api.set_status_led(led, on)?;
            let state = if on { "on" } else { "off" };
            println!("Status LED {led} turned {state}");
            if hold(sustain) {
                api.restore_status_leds()?;
            }
        }
        Commands::RestoreStatusLeds {} => {
            Kontroll::open(keyboard)?.restore_status_leds()?;
            println!("All status LEDs restored to their default state");
        }
        Commands::IncreaseBrightness { steps } => {
            Kontroll::open(keyboard)?.update_brightness(true, steps)?;
            println!("Brightness increased");
        }
        Commands::DecreaseBrightness { steps } => {
            Kontroll::open(keyboard)?.update_brightness(false, steps)?;
            println!("Brightness decreased");
        }
    }
    Ok(())
}

fn parse_color(color: &str) -> Result<(u8, u8, u8), ApiError> {
    utils::hex_to_rgb(color)
        .map_err(|_| ApiError::InvalidArgument(format!("{color} is not a valid hex color")))
}

/// Sleeps for `sustain` milliseconds; true when the caller should restore.
fn hold(sustain: i32) -> bool {
    if sustain <= 0 {
        return false;
    }
    thread::sleep(Duration::from_millis(sustain as u64));
    true
}
