use std::fmt;
use std::time::{Duration, Instant};

use serde::Serialize;
use zapp_core::device::ids::{self, Keyboard};
use zapp_core::hid::{
    HidConnection, HidEvent, CMD_RGB_CONTROL, CMD_SET_LAYER, CMD_SET_RGB_ALL_LED,
    CMD_SET_RGB_LED, CMD_SET_STATUS_LED, CMD_STATUS_LED_CONTROL, CMD_UPDATE_BRIGHTNESS,
};
use zapp_hid_proto::{CMD_GET_PROTOCOL_VERSION, ERR_UNKNOWN_COMMAND};

use crate::leds::LedMap;
use crate::ApiError;

/// How long to wait for the keyboard to answer.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(2);

/// Wait for queued commands to be written when the firmware has no protocol
/// version to answer with: `HidConnection`'s read loop flushes them every
/// 20 ms.
const FLUSH_WAIT: Duration = Duration::from_millis(100);

/// How the keyboard is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Usb,
    Bluetooth,
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Usb => write!(f, "USB"),
            Self::Bluetooth => write!(f, "Bluetooth"),
        }
    }
}

/// Where to open a keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// The raw HID interface of the first USB device with this VID/PID.
    Usb { vid: u16, pid: u16 },
    /// A Bluetooth board, by its platform handle.
    Bluetooth { key: String },
}

/// A keyboard Kontroll can control, as `kontroll list` shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyboardInfo {
    /// Position in [`Kontroll::list`]; what `--keyboard` takes.
    pub index: usize,
    pub friendly_name: String,
    pub transport: Transport,
    /// Extra state worth showing, such as "not bonded".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip)]
    pub keyboard: Option<Keyboard>,
    #[serde(skip)]
    pub target: Target,
}

impl KeyboardInfo {
    /// A USB keyboard, named after its PID.
    pub fn usb(index: usize, vid: u16, pid: u16) -> Self {
        Self {
            index,
            friendly_name: ids::target_name_for_pid(pid).to_string(),
            transport: Transport::Usb,
            note: None,
            keyboard: ids::identify_keyboard(vid, pid),
            target: Target::Usb { vid, pid },
        }
    }

    fn pid(&self) -> Option<u16> {
        match self.target {
            Target::Usb { pid, .. } => Some(pid),
            Target::Bluetooth { .. } => None,
        }
    }
}

impl fmt::Display for KeyboardInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} ({}", self.index, self.friendly_name, self.transport)?;
        if let Some(note) = &self.note {
            write!(f, ", {note}")?;
        }
        write!(f, ")")
    }
}

/// The opened keyboard, as `kontroll status` shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConnectedKeyboard {
    pub friendly_name: String,
    pub transport: Transport,
    /// `layoutId/revisionId`, from the firmware.
    pub firmware_version: String,
    /// `None` when the keyboard did not report it.
    pub current_layer: Option<u8>,
    /// `None` when the firmware predates the protocol version command.
    pub protocol_version: Option<u8>,
}

/// Kontroll's version and the opened keyboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Status {
    pub kontroll_version: String,
    pub keyboard: ConnectedKeyboard,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let k = &self.keyboard;
        let unknown = |value: Option<u8>| value.map_or("unknown".to_string(), |v| v.to_string());
        writeln!(f, "Kontroll version:\t{}", self.kontroll_version)?;
        writeln!(f, "Connected keyboard:\t{}", k.friendly_name)?;
        writeln!(f, "Transport:\t\t{}", k.transport)?;
        writeln!(f, "Firmware version:\t{}", k.firmware_version)?;
        writeln!(f, "Protocol version:\t{}", unknown(k.protocol_version))?;
        writeln!(f, "Current layer:\t\t{}", unknown(k.current_layer))
    }
}

/// One open keyboard. Commands go out as they are called; dropping it closes
/// the connection.
pub struct Kontroll {
    connection: Option<HidConnection>,
    info: KeyboardInfo,
    leds: LedMap,
    firmware_version: String,
    protocol_version: Option<u8>,
    layer: Option<u8>,
    paired: bool,
}

impl Kontroll {
    /// Every keyboard Kontroll can see: USB first, then Bluetooth.
    pub fn list() -> Result<Vec<KeyboardInfo>, ApiError> {
        let mut keyboards = usb_keyboards()?;
        keyboards.extend(ble_keyboards(keyboards.len())?);
        Ok(keyboards)
    }

    /// Opens keyboard `index` from [`Kontroll::list`], or the first one found
    /// when `None`. Waits for the handshake to finish.
    pub fn open(index: Option<usize>) -> Result<Self, ApiError> {
        let usb = usb_keyboards()?;
        let info = match index {
            None => match usb.first() {
                Some(first) => first.clone(),
                None => ble_keyboards(0)?
                    .into_iter()
                    .next()
                    .ok_or(ApiError::NoKeyboard)?,
            },
            Some(index) if index < usb.len() => {
                let info = usb[index].clone();
                if usb[..index].iter().any(|other| other.target == info.target) {
                    return Err(ApiError::IdenticalKeyboards { index });
                }
                info
            }
            Some(index) => {
                let ble = ble_keyboards(usb.len())?;
                let count = usb.len() + ble.len();
                if count == 0 {
                    return Err(ApiError::NoKeyboard);
                }
                ble.into_iter()
                    .nth(index - usb.len())
                    .ok_or(ApiError::NoSuchKeyboard { index, count })?
            }
        };

        let connection = connect(&info.target)?;
        Self::from_connection(connection, info)
    }

    /// Runs Kontroll over an already open connection, such as one built with
    /// `HidConnection::over` on a test link. Waits for the handshake.
    pub fn from_connection(connection: HidConnection, info: KeyboardInfo) -> Result<Self, ApiError> {
        let mut kontroll = Self {
            connection: Some(connection),
            leds: LedMap::for_keyboard(info.keyboard, info.pid()),
            info,
            firmware_version: String::new(),
            protocol_version: None,
            layer: None,
            paired: false,
        };
        kontroll.handshake()?;
        Ok(kontroll)
    }

    /// The opened keyboard and Kontroll's version.
    pub fn status(&self) -> Status {
        Status {
            kontroll_version: env!("CARGO_PKG_VERSION").to_string(),
            keyboard: ConnectedKeyboard {
                friendly_name: self.info.friendly_name.clone(),
                transport: self.info.transport,
                firmware_version: self.firmware_version.clone(),
                current_layer: self.layer,
                protocol_version: self.protocol_version,
            },
        }
    }

    /// The keyboard this was opened on.
    pub fn info(&self) -> &KeyboardInfo {
        &self.info
    }

    /// Moves to `layer`, turning every other layer off.
    pub fn set_layer(&mut self, layer: u8) -> Result<(), ApiError> {
        if !self.paired {
            return Err(ApiError::NotPaired);
        }
        self.send(CMD_SET_LAYER, &[1, layer]);
        self.sync()
    }

    /// Sets one RGB LED. Takes over the lighting until restored.
    pub fn set_rgb_led(&mut self, index: usize, r: u8, g: u8, b: u8) -> Result<(), ApiError> {
        let led = self.leds.firmware_index(index)?;
        self.send(CMD_SET_RGB_LED, &[led, r, g, b]);
        self.sync()
    }

    /// Sets every RGB LED. Takes over the lighting until restored.
    pub fn set_rgb_all(&mut self, r: u8, g: u8, b: u8) -> Result<(), ApiError> {
        self.send(CMD_SET_RGB_ALL_LED, &[r, g, b]);
        self.sync()
    }

    /// Hands the lighting back to the keyboard's own RGB settings.
    pub fn restore_rgb_leds(&mut self) -> Result<(), ApiError> {
        self.send(CMD_RGB_CONTROL, &[0]);
        self.sync()
    }

    /// Turns status LED `led` (0 to 5) on or off. Takes over the status LEDs
    /// until restored.
    pub fn set_status_led(&mut self, led: u8, on: bool) -> Result<(), ApiError> {
        self.send(CMD_SET_STATUS_LED, &[led, u8::from(on)]);
        self.sync()
    }

    /// Hands the status LEDs back to the keyboard.
    pub fn restore_status_leds(&mut self) -> Result<(), ApiError> {
        self.send(CMD_STATUS_LED_CONTROL, &[0]);
        self.sync()
    }

    /// Steps the RGB brightness up or down, 1 to 255 steps.
    pub fn update_brightness(&mut self, increase: bool, steps: i32) -> Result<(), ApiError> {
        if !(1..=255).contains(&steps) {
            return Err(ApiError::InvalidArgument(
                "Brightness steps must be between 1 and 255".to_string(),
            ));
        }
        for _ in 0..steps {
            self.send(CMD_UPDATE_BRIGHTNESS, &[u8::from(increase)]);
        }
        self.sync()
    }

    fn connection(&self) -> &HidConnection {
        self.connection
            .as_ref()
            .expect("connection is only taken on drop")
    }

    fn send(&self, cmd: u8, params: &[u8]) {
        self.connection().send(cmd, params);
    }

    /// Waits for the firmware version, the pairing answer, the first layer
    /// event and the protocol version. The firmware sends the layer right
    /// after pairing, before it answers the protocol version request.
    fn handshake(&mut self) -> Result<(), ApiError> {
        let deadline = Instant::now() + ANSWER_TIMEOUT;
        let mut have_firmware = false;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let Some(event) = self.connection().recv_timeout(remaining) else {
                return if have_firmware {
                    Ok(())
                } else {
                    Err(ApiError::NoAnswer)
                };
            };
            match event {
                HidEvent::FirmwareVersion {
                    layout_id,
                    revision_id,
                } => {
                    self.firmware_version = format!("{layout_id}/{revision_id}");
                    have_firmware = true;
                }
                HidEvent::PairingSuccess => self.paired = true,
                HidEvent::PairingFail => self.paired = false,
                HidEvent::LayerChanged(layer) => self.layer = Some(layer),
                HidEvent::ProtocolVersion(version) if have_firmware => {
                    self.protocol_version = Some(version);
                    return Ok(());
                }
                // Firmware without the protocol version command.
                HidEvent::Error(ERR_UNKNOWN_COMMAND) if have_firmware => return Ok(()),
                HidEvent::Disconnected => return Err(ApiError::Disconnected),
                _ => {}
            }
        }
    }

    /// Waits until every queued command is on the wire, and returns the first
    /// error the keyboard answered them with.
    ///
    /// Commands go out in order, so the answer to a protocol version request
    /// sent last means every earlier command was written.
    fn sync(&mut self) -> Result<(), ApiError> {
        let barrier = self.protocol_version.is_some();
        let wait = if barrier {
            self.send(CMD_GET_PROTOCOL_VERSION, &[]);
            ANSWER_TIMEOUT
        } else {
            FLUSH_WAIT
        };
        let deadline = Instant::now() + wait;
        let mut error = None;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let Some(event) = self.connection().recv_timeout(remaining) else {
                return match (error, barrier) {
                    (Some(code), _) => Err(ApiError::Firmware(code)),
                    (None, true) => Err(ApiError::NoAnswer),
                    (None, false) => Ok(()),
                };
            };
            match event {
                HidEvent::ProtocolVersion(_) if barrier => {
                    return error.map_or(Ok(()), |code| Err(ApiError::Firmware(code)));
                }
                HidEvent::Error(code) => {
                    error.get_or_insert(code);
                }
                HidEvent::LayerChanged(layer) => self.layer = Some(layer),
                HidEvent::Disconnected => return Err(ApiError::Disconnected),
                _ => {}
            }
        }
    }
}

impl Drop for Kontroll {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            connection.close();
        }
    }
}

/// ZSA keyboards running their firmware on USB, numbered from 0.
fn usb_keyboards() -> Result<Vec<KeyboardInfo>, ApiError> {
    let devices = zapp_core::device::list_devices()?;
    Ok(devices
        .into_iter()
        .filter(|device| device.bootloader.is_none() && device.keyboard.is_some())
        .enumerate()
        .map(|(index, device)| KeyboardInfo::usb(index, device.vid, device.pid))
        .collect())
}

#[cfg(feature = "ble")]
fn ble_keyboards(first_index: usize) -> Result<Vec<KeyboardInfo>, ApiError> {
    use zapp_ble::{BleBoard, Presence};

    let Some(mut backend) = zapp_ble::backend::for_platform(&[]) else {
        return Ok(Vec::new());
    };
    let sightings = match backend.sightings() {
        Ok(sightings) => sightings,
        // Bluetooth off or no adapter: nothing to list.
        Err(_) => return Ok(Vec::new()),
    };
    Ok(sightings
        .iter()
        .filter_map(|sighting| {
            let board = BleBoard::from_sighting(sighting)?;
            let note = if !sighting.paired {
                Some("not bonded".to_string())
            } else if board.presence == Presence::Paired {
                Some("not in range".to_string())
            } else {
                None
            };
            Some((board, note))
        })
        .enumerate()
        .map(|(i, (board, note))| KeyboardInfo {
            index: first_index + i,
            friendly_name: board
                .keyboard
                .map(|keyboard| keyboard.to_string())
                .or(board.name)
                .unwrap_or_else(|| "ZSA keyboard".to_string()),
            transport: Transport::Bluetooth,
            note,
            keyboard: board.keyboard,
            target: Target::Bluetooth { key: board.key },
        })
        .collect())
}

#[cfg(not(feature = "ble"))]
fn ble_keyboards(_first_index: usize) -> Result<Vec<KeyboardInfo>, ApiError> {
    Ok(Vec::new())
}

fn connect(target: &Target) -> Result<HidConnection, ApiError> {
    match target {
        Target::Usb { vid, pid } => Ok(HidConnection::open(*vid, *pid)?),
        #[cfg(feature = "ble")]
        Target::Bluetooth { key } => Ok(zapp_ble::oryx::open(Some(key))?),
        #[cfg(not(feature = "ble"))]
        Target::Bluetooth { .. } => Err(ApiError::Bluetooth(
            "this build of Kontroll has no Bluetooth support".to_string(),
        )),
    }
}
