use std::sync::Arc;

use kontroll::api::keymapp::{
    keyboard_service_server::KeyboardService, ConnectAnyKeyboardRequest, ConnectKeyboardRequest,
    DecreaseBrightnessRequest, DisconnectKeyboardRequest, GetKeyboardsRequest, GetStatusRequest,
    IncreaseBrightnessRequest, Keyboard, SetLayerRequest, SetRgbAllRequest, SetRgbLedRequest,
    SetStatusLedRequest,
};
use tempfile::NamedTempFile;
use tokio::net::{UnixListener, UnixStream};
use tonic::transport::Server;

pub struct Keymapp {
    keyboards: Vec<Keyboard>,
    connected_index: Option<usize>,
    next_request_fail: bool,
}

impl Default for Keymapp {
    fn default() -> Self {
        let keyboards = vec![Keyboard {
            id: 1,
            friendly_name: "Voyager".to_string(),
            is_connected: false,
        }];

        Self {
            keyboards,
            connected_index: None,
            next_request_fail: false,
        }
    }
}

pub async fn start_server(&self) {
    let sock = NamedTempFile::new().unwrap();

    let uds = UnixListener::bind(sock.path()).unwrap();
    let channel = UnixStream::connect(sock);

    Server::builder().add_service(self);
}

impl Keymapp {
    pub fn fail_next_request(&mut self, fail: bool) {
        self.next_request_fail = fail;
    }

    pub fn set_connected_index(&mut self, index: usize) {
        self.connected_index = Some(index);
    }
}

#[tonic::async_trait]
impl KeyboardService for Keymapp {
    async fn get_status(
        &self,
        _request: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn get_keyboards(
        &self,
        _request: tonic::Request<GetKeyboardsRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn connect_keyboard(
        &self,
        _request: tonic::Request<ConnectKeyboardRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn connect_any_keyboard(
        &self,
        _request: tonic::Request<ConnectAnyKeyboardRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn disconnect_keyboard(
        &self,
        _request: tonic::Request<DisconnectKeyboardRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn set_layer(
        &self,
        _request: tonic::Request<SetLayerRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn unset_layer(
        &self,
        _request: tonic::Request<SetLayerRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn set_rgb_led(
        &self,
        _request: tonic::Request<SetRgbLedRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn set_rgb_all(
        &self,
        _request: tonic::Request<SetRgbAllRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn set_status_led(
        &self,
        _request: tonic::Request<SetStatusLedRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn increase_brightness(
        &self,
        _request: tonic::Request<IncreaseBrightnessRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    async fn decrease_brightness(
        &self,
        _request: tonic::Request<DecreaseBrightnessRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }
}
