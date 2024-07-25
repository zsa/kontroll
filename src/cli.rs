use clap::{Parser, Subcommand};
use std::process::exit;

use crate::{api, utils};

#[derive(Debug, Parser)]
#[command(name = "Kontroll", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Kontroll demonstates how to control the Keymapp API, making it easy to control your ZSA keyboard from the command line and scripts.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(about = "Get the status of the currently connected keyboard")]
    Status {
        #[arg(short, long, default_value = "false")]
        json: bool,
    },
    #[command(about = "List all available keyboards")]
    List,
    #[command(about = "Connect to a keyboard given the index returned by the list command")]
    Connect {
        #[arg(short, long, name = "keyboard index", required = true)]
        index: usize,
    },
    #[command(about = "Connect to the first keyboard detected by keymapp")]
    ConnectAny,
    #[command(about = "Set the layer of the currently connected keyboard")]
    SetLayer {
        #[arg(short, long, required = true)]
        index: usize,
    },
    #[command(about = "Sets the RGB color of a LED")]
    SetRGB {
        #[arg(short, long, required = true)]
        led: usize,
        #[arg(short, long, required = true)]
        color: String,
        #[arg(short, long, default_value = "0")]
        sustain: i32,
    },
    #[command(about = "Sets the RGB color of all LEDs")]
    SetRGBAll {
        #[arg(short, long, required = true)]
        color: String,
        #[arg(short, long, default_value = "0")]
        sustain: i32,
    },
    #[command(about = "Restores the RGB color of all LEDs to their default")]
    RestoreRGBLeds {},
    #[command(about = "Set / Unset a status LED")]
    SetStatusLed {
        #[arg(short, long, required = true)]
        led: usize,
        #[arg(short, long)]
        off: bool,
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
    #[command(about = "Disconnect from the currently connected keyboard")]
    Disconnect,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { json } => match api::get_status().await {
            Ok(status) => {
                if json {
                    println!("{}", serde_json::to_string_pretty(&status).unwrap());
                } else {
                    println!("{}", status)
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::List => match api::list_keyboards().await {
            Ok(keyboards) => {
                for (_i, keyboard) in keyboards.iter().enumerate() {
                    let connected = if keyboard.is_connected {
                        "(connected)"
                    } else {
                        ""
                    };
                    println!("{}: {} {}", keyboard.id, keyboard.friendly_name, connected);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::Connect { index } => match api::connect(index).await {
            Ok(_) => {
                println!("Connected to keyboard {}", index);
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::ConnectAny => match api::connect_any().await {
            Ok(_) => {
                println!("Connected to the first keyboard detected by keymapp");
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::Disconnect => match api::disconnect().await {
            Ok(_) => {
                println!("Disconnected from the currently connected keyboard");
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::SetLayer { index } => match api::set_layer(index).await {
            Ok(_) => {
                println!("Layer set to {}", index);
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::SetRGB {
            led,
            color,
            sustain,
        } => {
            let (r, g, b) = match utils::hex_to_rgb(&color) {
                Ok(rgb) => rgb,
                Err(_) => {
                    eprintln!("{} is not a valid hex color", color);
                    exit(1);
                }
            };

            match api::set_rgb_led(led, r, g, b, sustain).await {
                Ok(_) => {
                    println!("LED {} set to color {}", led, color);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            }
        }
        Commands::SetRGBAll { color, sustain } => {
            let (r, g, b) = match utils::hex_to_rgb(&color) {
                Ok(rgb) => rgb,
                Err(_) => {
                    eprintln!("{} is not a valid hex color", color);
                    exit(1);
                }
            };

            match api::set_rgb_all(r, g, b, sustain).await {
                Ok(_) => {
                    println!("All LEDs set to color {}", color);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            }
        }
        Commands::RestoreRGBLeds {} => match api::restore_rgb_leds().await {
            Ok(_) => {
                println!("All LEDs restored to their default color");
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::SetStatusLed { led, off, sustain } => {
            let on = !off;
            match api::set_status_led(led, on, sustain).await {
                Ok(_) => {
                    let state = if on { "on" } else { "off" };
                    println!("Status LED {} turned {}", led, state);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            }
        }
        Commands::RestoreStatusLeds {} => match api::restore_status_leds().await {
            Ok(_) => {
                println!("All status LEDs restored to their default state");
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::IncreaseBrightness { steps } => match api::update_brightness(true, steps).await {
            Ok(_) => {
                println!("Brightness increased");
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
        Commands::DecreaseBrightness { steps } => {
            match api::update_brightness(false, steps).await {
                Ok(_) => {
                    println!("Brightness decreased");
                }
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            }
        }
    }
}
