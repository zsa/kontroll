use std::fmt;

use keymapp::{
    ConnectAnyKeyboardRequest, ConnectKeyboardRequest, DisconnectKeyboardRequest,
    GetKeyboardsRequest,
};
use tonic::Request;

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

pub async fn get_client() -> Result<KeyboardServiceClient<tonic::transport::Channel>, ApiError> {
    // Get port number from environment variable or use default
    let port = std::env::var("KEYMAPP_PORT").unwrap_or("50051".to_string());
    let addr = format!("http://localhost:{}", port);
    let timeout = std::time::Duration::from_secs(5);
    let client = match tokio::time::timeout(timeout, KeyboardServiceClient::connect(addr)).await {
        Ok(Ok(c)) => Ok(c),
        Err(_) => Err(ApiError { message: format!("Connection to Keymapp timed out, make sure the api is running and listening to port {}", port) }),
        Ok(Err(e)) => Err(ApiError { message: format!("Connection to Keymapp failed, with error {}", e.to_string()) })
    };

    Ok(client?)
}

pub async fn list_keyboards() -> Result<Vec<Keyboard>, ApiError> {
    let mut cli = get_client().await?;
    println!("Getting keyboards");
    let req = Request::new(GetKeyboardsRequest {});
    let res = match cli.get_keyboards(req).await {
        Ok(r) => r.into_inner().keyboards,
        Err(e) => {
            return Err(ApiError {
                message: format!("Failed to get keyboards: {}", e.message()),
            })
        }
    };
    Ok(res)
}

pub async fn connect(index: usize) -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let req = Request::new(ConnectKeyboardRequest { id: index as i32 });
    let res = match cli.connect_keyboard(req).await {
        Ok(r) => r.into_inner().success,
        Err(e) => {
            return Err(ApiError {
                message: format!("Failed to connect: {}", e.message()),
            })
        }
    };

    Ok(res)
}

pub async fn connect_any() -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let req = Request::new(ConnectAnyKeyboardRequest {});
    let res = match cli.connect_any_keyboard(req).await {
        Ok(r) => r.into_inner().success,
        Err(e) => {
            return Err(ApiError {
                message: format!("Failed to connect: {}", e.message()),
            })
        }
    };

    Ok(res)
}

pub async fn set_layer(index: usize) -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli
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
    index: usize,
    r: u8,
    g: u8,
    b: u8,
    sustain: i32,
) -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli
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

pub async fn set_rgb_all(r: u8, g: u8, b: u8, sustain: i32) -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli
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
pub async fn restore_rgb_leds() -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli
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

pub async fn set_status_led(led: usize, on: bool, sustain: i32) -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli
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

pub async fn restore_status_leds() -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli
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

pub async fn update_brightness(increase: bool) -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    if increase {
        let res = match cli
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

        Ok(res)
    } else {
        let res = match cli
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

        Ok(res)
    }
}

pub async fn disconnect() -> Result<bool, ApiError> {
    let mut cli = get_client().await?;
    let res = match cli.disconnect_keyboard(DisconnectKeyboardRequest {}).await {
        Ok(r) => r.into_inner().success,
        Err(e) => {
            return Err(ApiError {
                message: format!("Failed to disconnect: {}", e.message()),
            })
        }
    };

    Ok(res)
}
