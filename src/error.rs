use std::fmt;

/// Firmware error codes (`Oryx_Error_Code` in QMK's `oryx.h`).
const ERR_RGB_MATRIX_NOT_ENABLED: u8 = 4;
const ERR_STATUS_LED_OUT_OF_RANGE: u8 = 5;
const ERR_UNKNOWN_COMMAND: u8 = 0xFF;

/// Everything that can go wrong talking to a keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// No ZSA keyboard on USB or Bluetooth.
    NoKeyboard,
    /// `index` is not in the list of keyboards.
    NoSuchKeyboard { index: usize, count: usize },
    /// Two or more identical USB keyboards: only the first can be opened.
    IdenticalKeyboards { index: usize },
    /// The OS refused to open the keyboard.
    PermissionDenied(String),
    /// The Bluetooth keyboard is not bonded to this computer.
    NotBonded(String),
    /// The keyboard did not answer the handshake in time.
    NoAnswer,
    /// The keyboard went away while talking to it.
    Disconnected,
    /// The keyboard did not accept the pairing, so it ignores layer changes.
    NotPaired,
    /// The keyboard answered a command with this error code.
    Firmware(u8),
    /// A value outside what the command or keyboard takes.
    InvalidArgument(String),
    /// A USB or HID failure, as the system reported it.
    Usb(String),
    /// A Bluetooth failure, as the system reported it.
    Bluetooth(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoKeyboard => write!(
                f,
                "No ZSA keyboard found. Plug one in, or bond a Wireless Voyager over Bluetooth."
            ),
            Self::NoSuchKeyboard { index, count } => write!(
                f,
                "There is no keyboard {index}: {count} found. Run `kontroll list` to see them."
            ),
            Self::IdenticalKeyboards { index } => write!(
                f,
                "Keyboard {index} is the same model as an earlier one on USB, and identical USB \
                 keyboards cannot be told apart. Only the first of them can be controlled."
            ),
            Self::PermissionDenied(detail) => {
                write!(f, "No permission to open the keyboard ({detail}).")?;
                if cfg!(target_os = "linux") {
                    write!(
                        f,
                        " Install ZSA's udev rules (50-zsa.rules from \
                         https://github.com/zsa/zapp/tree/main/udev), then unplug and replug the keyboard."
                    )?;
                }
                Ok(())
            }
            Self::NotBonded(name) => write!(
                f,
                "{name} is not bonded to this computer. Pair it in your Bluetooth settings first."
            ),
            Self::NoAnswer => write!(f, "The keyboard did not answer."),
            Self::Disconnected => write!(f, "The keyboard disconnected."),
            Self::NotPaired => write!(
                f,
                "The keyboard did not accept the connection, so it ignores layer changes."
            ),
            Self::Firmware(ERR_RGB_MATRIX_NOT_ENABLED) => {
                write!(f, "This keyboard's firmware has no RGB lighting to control.")
            }
            Self::Firmware(ERR_STATUS_LED_OUT_OF_RANGE) => {
                write!(f, "This keyboard has no status LED with that index.")
            }
            Self::Firmware(ERR_UNKNOWN_COMMAND) => {
                write!(f, "This keyboard's firmware does not support that command.")
            }
            Self::Firmware(code) => write!(f, "The keyboard reported error {code}."),
            Self::InvalidArgument(message) => write!(f, "{message}"),
            Self::Usb(message) => write!(f, "USB error: {message}"),
            Self::Bluetooth(message) => write!(f, "Bluetooth error: {message}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<zapp_core::ZappError> for ApiError {
    fn from(err: zapp_core::ZappError) -> Self {
        match err {
            zapp_core::ZappError::NoKeyboardFound => Self::NoKeyboard,
            zapp_core::ZappError::AccessDenied(err) => Self::PermissionDenied(err.to_string()),
            // hidapi's hidraw backend reports strerror(EACCES) in its message.
            zapp_core::ZappError::Hid(err) if err.to_string().contains("Permission denied") => {
                Self::PermissionDenied(err.to_string())
            }
            other => Self::Usb(other.to_string()),
        }
    }
}

#[cfg(feature = "ble")]
impl From<zapp_ble::oryx::OryxError> for ApiError {
    fn from(err: zapp_ble::oryx::OryxError) -> Self {
        use zapp_ble::oryx::OryxError;
        use zapp_ble::BleError;
        match err {
            OryxError::Ble(BleError::NotBonded(name)) => Self::NotBonded(name),
            OryxError::NoBoard(_) => Self::NoKeyboard,
            other => Self::Bluetooth(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidraw_permission_errors_are_recognised() {
        let err = zapp_core::ZappError::Hid(hidapi_error(
            "Failed to open a device with path '/dev/hidraw3': Permission denied",
        ));
        assert!(matches!(ApiError::from(err), ApiError::PermissionDenied(_)));
    }

    #[test]
    fn other_hid_errors_stay_usb_errors() {
        let err = zapp_core::ZappError::Hid(hidapi_error("hid_write/out: broken pipe"));
        assert!(matches!(ApiError::from(err), ApiError::Usb(_)));
    }

    fn hidapi_error(message: &str) -> hidapi::HidError {
        hidapi::HidError::HidApiError {
            message: message.to_string(),
        }
    }
}
