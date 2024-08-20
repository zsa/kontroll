use serde::Serialize;
use std::fmt;

/// Generated code from the proto file
use keymapp::{
    ConnectAnyKeyboardRequest, ConnectKeyboardRequest, DisconnectKeyboardRequest,
    GetKeyboardsRequest,
};

#[cfg(not(target_os = "windows"))]
use tokio::net::UnixStream;

use tonic::Request;
#[cfg(not(target_os = "windows"))]
use tonic::transport::{
    Endpoint, Uri
};
#[cfg(not(target_os = "windows"))]
use tower::service_fn;

#[derive(Debug)]
pub struct ApiError {
    message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

use self::keymapp::{
    keyboard_service_client::KeyboardServiceClient, Keyboard, SetLayerRequest, SetRgbAllRequest,
    SetRgbLedRequest,
};

/// Generated data structures from the proto file
pub mod keymapp {
    tonic::include_proto!("api");
}

/// The kontroll API.
pub struct Kontroll {
    client: KeyboardServiceClient<tonic::transport::Channel>,
}

#[derive(Serialize)]
/// Data representation of a connected keyboard, used in the status response
pub struct ConnectedKeyboard {
    friendly_name: String,
    firmware_version: String,
    current_layer: i32,
}

#[derive(Serialize)]
/// Data representation of the status response, including the version of Kontroll and Keymapp and optionally the connected keyboard.
pub struct Status {
    keymapp_version: String,
    kontroll_version: String,
    keyboard: Option<ConnectedKeyboard>,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keyboard = match &self.keyboard {
            Some(k) => format!(
                "Connected keyboard:\t{}\nFirmware version:\t{}\nCurrent layer:\t\t{}",
                k.friendly_name, k.firmware_version, k.current_layer
            ),
            None => "No keyboard connected".to_string(),
        };
        write!(
            f,
            "Keymapp version:\t{}\nKontroll version:\t{}\n{}\n",
            self.keymapp_version, self.kontroll_version, keyboard
        )
    }
}

#[cfg(not(target_os = "windows"))]
/// The get client function handles the connection to Keymapp, on Unix systems it uses a Unix domain socket, on Windows it uses a TCP connection.
pub async fn get_client(
    path: Option<String>,
) -> Result<KeyboardServiceClient<tonic::transport::Channel>, ApiError> {
    // Get socket path from the supplied path provided, or environment variable or set a default

    let socket_path = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => match std::env::var("KEYMAPP_SOCKET") {
            Ok(p) => std::path::PathBuf::from(p),
            Err(_) => {
                let dirs = match directories::BaseDirs::new() {
                    Some(dirs) => dirs,
                    None => {
                        return Err(ApiError {
                            message: "Failed to get config directory".to_string(),
                        })
                    }
                };
                dirs.config_dir().join(".keymapp/").join("keymapp.sock")
            }
        },
    };

    if !socket_path.exists() {
        return Err(ApiError { message: format!("Keymapp socket not found at {}, make sure Keymapp is running and the API is started.", socket_path.to_str().unwrap()) });
    }

    let channel = Endpoint::try_from("http://[::]:50051")
        .map_err(|e| ApiError {
            message: format!("Failed to create api client: {}", e),
        })?
        .connect_with_connector(service_fn(move |_: Uri| {
            UnixStream::connect(socket_path.clone())
        }))
        .await
        .map_err(|e| ApiError {
            message: format!("Failed to connect to keymapp: {}", e),
        })?;

    let client = KeyboardServiceClient::new(channel);
    Ok(client)
}

#[cfg(target_os = "windows")]
pub async fn get_client(
    port: Option<String>,
) -> Result<KeyboardServiceClient<tonic::transport::Channel>, ApiError> {
    // Get port number from the supplied path provided, or environment variable or set a default
    let port = port.unwrap_or_else(|| std::env::var("KEYMAPP_PORT").unwrap_or("50051".to_string()));
    let addr = format!("http://localhost:{}", port);
    let timeout = std::time::Duration::from_secs(5);

    match tokio::time::timeout(timeout, KeyboardServiceClient::connect(addr)).await {
        Ok(Ok(c)) => Ok(c),
        Err(_) => Err(ApiError { message: format!("Connection to Keymapp timed out, make sure the api is running and listening to port {}", port) }),
        Ok(Err(e)) => Err(ApiError { message: format!("Connection to Keymapp failed, with error {}", e) })
    }
}

impl Kontroll {
    /// Create a new Kontroll instance, connecting to Keymapp, optionally specifying a port number on Windows or a socket path on Unix.
    pub async fn new(port: Option<String>) -> Result<Self, ApiError> {
        let client = get_client(port).await?;
        Ok(Self { client })
    }

    /// Gets Keymapp's version, Kontroll's version and the connected keyboard's information.
    pub async fn get_status(&self) -> Result<Status, ApiError> {
        let req = Request::new(keymapp::GetStatusRequest {});
        // Tonic internals require a mutable reference to the client, so we clone it here.
        // https://github.com/hyperium/tonic/issues/33#issuecomment-538154015
        match self.client.clone().get_status(req).await {
            Ok(r) => {
                let res = r.into_inner();
                let keyboard = match res.connected_keyboard {
                    Some(k) => Some(ConnectedKeyboard {
                        friendly_name: k.friendly_name,
                        firmware_version: k.firmware_version,
                        current_layer: k.current_layer,
                    }),
                    None => None,
                };
                Ok(Status {
                    keymapp_version: res.keymapp_version,
                    kontroll_version: env!("CARGO_PKG_VERSION").to_string(),
                    keyboard,
                })
            }
            Err(e) => Err(ApiError {
                message: format!("Failed to get status: {}", e.message()),
            }),
        }
    }

    /// Gets a list of available keyboards.
    pub async fn list_keyboards(&self) -> Result<Vec<Keyboard>, ApiError> {
        let req = Request::new(GetKeyboardsRequest {});
        let res = match self.client.clone().get_keyboards(req).await {
            Ok(r) => r.into_inner().keyboards,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to get keyboards: {}", e.message()),
                })
            }
        };
        Ok(res)
    }

    /// Connects to a keyboard by index.
    pub async fn connect(&self, index: usize) -> Result<bool, ApiError> {
        let req = Request::new(ConnectKeyboardRequest { id: index as i32 });
        let res = match self.client.clone().connect_keyboard(req).await {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to connect: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Connects to the first entry in the list of available keyboards.
    pub async fn connect_any(&self) -> Result<bool, ApiError> {
        let req = Request::new(ConnectAnyKeyboardRequest {});
        let res = match self.client.clone().connect_any_keyboard(req).await {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to connect: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Sets a layer by index on the connected keyboard.
    pub async fn set_layer(&self, index: usize) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .set_layer(SetLayerRequest {
                layer: index as i32,
            })
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to set layer: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Sets an RGB LED by index on the connected keyboard.
    pub async fn set_rgb_led(
        &self,
        index: usize,
        r: u8,
        g: u8,
        b: u8,
        sustain: i32,
    ) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .set_rgb_led(SetRgbLedRequest {
                led: index as i32,
                red: r as i32,
                green: g as i32,
                blue: b as i32,
                sustain,
            })
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to set rgb: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Sets all RGB LEDs on the connected keyboard.
    pub async fn set_rgb_all(&self, r: u8, g: u8, b: u8, sustain: i32) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .set_rgb_all(SetRgbAllRequest {
                red: r as i32,
                green: g as i32,
                blue: b as i32,
                sustain,
            })
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to set rgb: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Restores all RGB LEDs on the connected keyboard.
    pub async fn restore_rgb_leds(&self) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .set_rgb_all(SetRgbAllRequest {
                red: 0,
                green: 0,
                blue: 0,
                sustain: 1,
            })
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to set rgb: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Sets a status LED by index on the connected keyboard.
    pub async fn set_status_led(
        &self,
        led: usize,
        on: bool,
        sustain: i32,
    ) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .set_status_led(keymapp::SetStatusLedRequest {
                led: led as i32,
                on,
                sustain,
            })
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to set status led: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Restores all status LEDs on the connected keyboard.
    pub async fn restore_status_leds(&self) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .set_status_led(keymapp::SetStatusLedRequest {
                led: 0,
                on: false,
                sustain: 1,
            })
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to set status led: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    /// Sets the brightness of the connected keyboard. Several steps can be taken.
    pub async fn update_brightness(&self, increase: bool, steps: i32) -> Result<bool, ApiError> {
        let mut res = false;
        if !(1..=255).contains(&steps) {
            return Err(ApiError {
                message: "Brightness steps must be between 1 and 255".to_string(),
            });
        }
        if increase {
            for _ in 0..steps {
                res = match self
                    .client
                    .clone()
                    .increase_brightness(keymapp::IncreaseBrightnessRequest {})
                    .await
                {
                    Ok(r) => r.into_inner().success,
                    Err(e) => {
                        return Err(ApiError {
                            message: format!("Failed to increase brightness: {}", e.message()),
                        })
                    }
                };
                if !res {
                    break;
                }
            }
        } else {
            for _ in 0..steps {
                res = match self
                    .client
                    .clone()
                    .decrease_brightness(keymapp::DecreaseBrightnessRequest {})
                    .await
                {
                    Ok(r) => r.into_inner().success,
                    Err(e) => {
                        return Err(ApiError {
                            message: format!("Failed to decrease brightness: {}", e.message()),
                        })
                    }
                };
                if !res {
                    break;
                }
            }
        }
        Ok(res)
    }

    /// Disconnects the connected keyboard.
    pub async fn disconnect(&self) -> Result<bool, ApiError> {
        let res = match self
            .client
            .clone()
            .disconnect_keyboard(DisconnectKeyboardRequest {})
            .await
        {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to disconnect: {}", e.message()),
                })
            }
        };

        Ok(res)
    }
}
