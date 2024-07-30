use serde::Serialize;
use std::fmt;

use keymapp::{
    ConnectAnyKeyboardRequest, ConnectKeyboardRequest, DisconnectKeyboardRequest,
    GetKeyboardsRequest,
};

#[cfg(not(target_os = "windows"))]
use tokio::net::UnixStream;

use tonic::{
    transport::{Endpoint, Uri},
    Request,
};
use tower::service_fn;

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

pub mod keymapp {
    tonic::include_proto!("api");
}

pub struct Kontroll {
    client: KeyboardServiceClient<tonic::transport::Channel>,
}

#[derive(Serialize)]
pub struct ConnectedKeyboard {
    friendly_name: String,
    firmware_version: String,
    current_layer: i32,
}

#[derive(Serialize)]
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
    let client = match tokio::time::timeout(timeout, KeyboardServiceClient::connect(addr)).await {
        Ok(Ok(c)) => Ok(c),
        Err(_) => Err(ApiError { message: format!("Connection to Keymapp timed out, make sure the api is running and listening to port {}", port) }),
        Ok(Err(e)) => Err(ApiError { message: format!("Connection to Keymapp failed, with error {}", e.to_string()) })
    };

    Ok(client?)
}

impl Kontroll {
    pub async fn new(port: Option<String>) -> Result<Self, ApiError> {
        let client = get_client(port).await?;
        Ok(Self { client })
    }

    pub async fn get_status(&mut self) -> Result<Status, ApiError> {
        let req = Request::new(keymapp::GetStatusRequest {});
        match self.client.get_status(req).await {
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
                return Ok(Status {
                    keymapp_version: res.keymapp_version,
                    kontroll_version: env!("CARGO_PKG_VERSION").to_string(),
                    keyboard,
                });
            }
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to get status: {}", e.message()),
                })
            }
        };
    }

    pub async fn list_keyboards(&mut self) -> Result<Vec<Keyboard>, ApiError> {
        println!("Getting keyboards");
        let req = Request::new(GetKeyboardsRequest {});
        let res = match self.client.get_keyboards(req).await {
            Ok(r) => r.into_inner().keyboards,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to get keyboards: {}", e.message()),
                })
            }
        };
        Ok(res)
    }

    pub async fn connect(&mut self, index: usize) -> Result<bool, ApiError> {
        let req = Request::new(ConnectKeyboardRequest { id: index as i32 });
        let res = match self.client.connect_keyboard(req).await {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to connect: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    pub async fn connect_any(&mut self) -> Result<bool, ApiError> {
        let req = Request::new(ConnectAnyKeyboardRequest {});
        let res = match self.client.connect_any_keyboard(req).await {
            Ok(r) => r.into_inner().success,
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to connect: {}", e.message()),
                })
            }
        };

        Ok(res)
    }

    pub async fn set_layer(&mut self, index: usize) -> Result<bool, ApiError> {
        let res = match self
            .client
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

    pub async fn set_rgb_led(
        &mut self,
        index: usize,
        r: u8,
        g: u8,
        b: u8,
        sustain: i32,
    ) -> Result<bool, ApiError> {
        let res = match self
            .client
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

    pub async fn set_rgb_all(
        &mut self,
        r: u8,
        g: u8,
        b: u8,
        sustain: i32,
    ) -> Result<bool, ApiError> {
        let res = match self
            .client
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

    // Set all leds to off then restore previous state after 1 millisecond
    pub async fn restore_rgb_leds(&mut self) -> Result<bool, ApiError> {
        let res = match self
            .client
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

    pub async fn set_status_led(
        &mut self,
        led: usize,
        on: bool,
        sustain: i32,
    ) -> Result<bool, ApiError> {
        let res = match self
            .client
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

    pub async fn restore_status_leds(&mut self) -> Result<bool, ApiError> {
        let res = match self
            .client
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

    pub async fn update_brightness(
        &mut self,
        increase: bool,
        steps: i32,
    ) -> Result<bool, ApiError> {
        let mut res = false;
        if steps < 1 || steps > 255 {
            return Err(ApiError {
                message: "Brightness steps must be between 1 and 255".to_string(),
            });
        }
        if increase {
            for _ in 0..steps {
                res = match self
                    .client
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

    pub async fn disconnect(&mut self) -> Result<bool, ApiError> {
        let res = match self
            .client
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
