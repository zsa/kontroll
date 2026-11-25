//! Kontroll against a fake keyboard: checks the exact reports each command
//! writes. The fake answers like QMK's `oryx.c`.

use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use kontroll::{ApiError, KeyboardInfo, Kontroll, Transport};
use zapp_core::hid::{HidConnection, ReportLink};

const REPORT_SIZE: usize = 32;
const STOP_BIT: u8 = 0xFE;
const ERR_RGB_MATRIX_NOT_ENABLED: u8 = 4;
const ERR_STATUS_LED_OUT_OF_RANGE: u8 = 5;

type Sent = Arc<Mutex<Vec<[u8; REPORT_SIZE]>>>;

/// How the fake keyboard behaves.
#[derive(Clone, Copy)]
struct Firmware {
    /// Answers the handshake at all.
    answers: bool,
    /// Accepts the pairing.
    pairs: bool,
    /// Answers the protocol version request.
    has_protocol_version: bool,
    /// Has an RGB matrix.
    rgb: bool,
    layer: u8,
}

impl Default for Firmware {
    fn default() -> Self {
        Self {
            answers: true,
            pairs: true,
            has_protocol_version: true,
            rgb: true,
            layer: 2,
        }
    }
}

struct FakeKeyboard {
    firmware: Firmware,
    sent: Sent,
    pending: Vec<Vec<u8>>,
}

fn report(bytes: &[u8]) -> Vec<u8> {
    let mut report = vec![0u8; REPORT_SIZE];
    report[..bytes.len()].copy_from_slice(bytes);
    report
}

impl ReportLink for FakeKeyboard {
    fn send(&mut self, sent: &[u8; REPORT_SIZE]) -> io::Result<()> {
        self.sent.lock().unwrap().push(*sent);
        if !self.firmware.answers {
            return Ok(());
        }
        let error = |code: u8| report(&[0xFF, code]);
        match sent[0] {
            // GET_FW_VERSION
            0x00 => {
                let mut answer = vec![0x00];
                answer.extend_from_slice(b"abc12/def34");
                answer.push(STOP_BIT);
                self.pending.push(report(&answer));
            }
            // PAIRING_INIT
            0x01 if self.firmware.pairs => {
                self.pending.push(report(&[0x04, STOP_BIT]));
                self.pending.push(report(&[0x05, self.firmware.layer, STOP_BIT]));
            }
            // GET_PROTOCOL_VERSION
            0xFE if self.firmware.has_protocol_version => {
                self.pending.push(report(&[0xFE, 0x06, STOP_BIT]));
            }
            0xFE => self.pending.push(error(0xFF)),
            // SET_LAYER
            0x04 if sent[1] != 0 => {
                self.firmware.layer = sent[2];
                self.pending.push(report(&[0x05, sent[2], STOP_BIT]));
            }
            // SET_RGB_LED, SET_RGB_LED_ALL, UPDATE_BRIGHTNESS
            0x06 | 0x09 | 0x08 if !self.firmware.rgb => {
                self.pending.push(error(ERR_RGB_MATRIX_NOT_ENABLED))
            }
            // RGB_CONTROL
            0x05 => self.pending.push(report(&[0x08, sent[1]])),
            // SET_STATUS_LED
            0x07 if sent[1] > 5 => self.pending.push(error(ERR_STATUS_LED_OUT_OF_RANGE)),
            // STATUS_LED_CONTROL
            0x0A => self.pending.push(report(&[0x0B, sent[1]])),
            _ => {}
        }
        Ok(())
    }

    fn receive(&mut self, timeout: Duration) -> io::Result<Option<Vec<u8>>> {
        if self.pending.is_empty() {
            thread::sleep(timeout);
            return Ok(None);
        }
        Ok(Some(self.pending.remove(0)))
    }
}

/// A link that fails every read and write.
struct Unplugged;

impl ReportLink for Unplugged {
    fn send(&mut self, _: &[u8; REPORT_SIZE]) -> io::Result<()> {
        Err(io::Error::other("unplugged"))
    }

    fn receive(&mut self, _: Duration) -> io::Result<Option<Vec<u8>>> {
        Err(io::Error::other("unplugged"))
    }
}

fn open_with(firmware: Firmware, pid: u16) -> (Result<Kontroll, ApiError>, Sent) {
    let sent = Sent::default();
    let link = FakeKeyboard {
        firmware,
        sent: sent.clone(),
        pending: Vec::new(),
    };
    let info = KeyboardInfo::usb(0, 0x3297, pid);
    (
        Kontroll::from_connection(HidConnection::over(link), info),
        sent,
    )
}

fn open_voyager() -> (Kontroll, Sent) {
    let (kontroll, sent) = open_with(Firmware::default(), 0x1977);
    (kontroll.unwrap(), sent)
}

/// The non-zero prefix of each report written since `from`, dropping the
/// protocol version requests Kontroll uses to know a command was written.
fn written(sent: &Sent, from: usize) -> Vec<Vec<u8>> {
    sent.lock().unwrap()[from..]
        .iter()
        .filter(|report| report[0] != 0xFE)
        .map(|report| {
            let len = report.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
            report[..len].to_vec()
        })
        .collect()
}

/// Runs `command` on a fresh Voyager and returns what it wrote.
fn command_bytes(command: impl FnOnce(&mut Kontroll) -> Result<(), ApiError>) -> Vec<Vec<u8>> {
    let (mut kontroll, sent) = open_voyager();
    let before = sent.lock().unwrap().len();
    command(&mut kontroll).unwrap();
    written(&sent, before)
}

#[test]
fn the_handshake_asks_for_firmware_pairing_and_protocol_version() {
    let (_kontroll, sent) = open_voyager();
    let sent = sent.lock().unwrap();
    let mut init_pairing = [0u8; REPORT_SIZE];
    init_pairing[0] = 0x01;
    let mut protocol = [0u8; REPORT_SIZE];
    protocol[0] = 0xFE;
    assert_eq!(sent[..], [[0u8; REPORT_SIZE], init_pairing, protocol]);
}

#[test]
fn status_reports_what_the_handshake_learned() {
    let (kontroll, _) = open_voyager();
    let status = kontroll.status();
    assert_eq!(status.kontroll_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(status.keyboard.friendly_name, "Voyager");
    assert_eq!(status.keyboard.transport, Transport::Usb);
    assert_eq!(status.keyboard.firmware_version, "abc12/def34");
    assert_eq!(status.keyboard.current_layer, Some(2));
    assert_eq!(status.keyboard.protocol_version, Some(6));
}

#[test]
fn status_json_keeps_the_keyboard_object() {
    let (kontroll, _) = open_voyager();
    let json: serde_json::Value = serde_json::to_value(kontroll.status()).unwrap();
    assert_eq!(json["keyboard"]["friendly_name"], "Voyager");
    assert_eq!(json["keyboard"]["firmware_version"], "abc12/def34");
    assert_eq!(json["keyboard"]["current_layer"], 2);
    assert_eq!(json["keyboard"]["transport"], "usb");
}

#[test]
fn set_layer_moves_to_the_layer() {
    assert_eq!(command_bytes(|k| k.set_layer(3)), [vec![0x04, 0x01, 0x03]]);
}

#[test]
fn set_layer_updates_the_known_layer() {
    let (mut kontroll, _) = open_voyager();
    kontroll.set_layer(4).unwrap();
    assert_eq!(kontroll.status().keyboard.current_layer, Some(4));
}

#[test]
fn set_layer_needs_the_pairing() {
    let firmware = Firmware {
        pairs: false,
        ..Firmware::default()
    };
    let (kontroll, sent) = open_with(firmware, 0x1977);
    let mut kontroll = kontroll.unwrap();
    let before = sent.lock().unwrap().len();
    assert_eq!(kontroll.set_layer(1), Err(ApiError::NotPaired));
    assert!(written(&sent, before).is_empty());
}

#[test]
fn set_rgb_led_sends_index_and_color() {
    assert_eq!(
        command_bytes(|k| k.set_rgb_led(5, 0x10, 0x20, 0x30)),
        [vec![0x06, 5, 0x10, 0x20, 0x30]]
    );
}

#[test]
fn set_rgb_led_remaps_moonlander_indices() {
    let (kontroll, sent) = open_with(Firmware::default(), 0x1969);
    let mut kontroll = kontroll.unwrap();
    let before = sent.lock().unwrap().len();
    kontroll.set_rgb_led(1, 1, 2, 3).unwrap();
    assert_eq!(written(&sent, before), [vec![0x06, 5, 1, 2, 3]]);
}

#[test]
fn set_rgb_led_refuses_an_led_the_board_lacks() {
    let (mut kontroll, sent) = open_voyager();
    let before = sent.lock().unwrap().len();
    assert!(matches!(
        kontroll.set_rgb_led(52, 1, 2, 3),
        Err(ApiError::InvalidArgument(_))
    ));
    assert!(written(&sent, before).is_empty());
}

#[test]
fn set_rgb_all_sends_the_color() {
    assert_eq!(
        command_bytes(|k| k.set_rgb_all(0xFF, 0x80, 0x01)),
        [vec![0x09, 0xFF, 0x80, 0x01]]
    );
}

#[test]
fn restore_rgb_leds_releases_rgb_control() {
    assert_eq!(command_bytes(|k| k.restore_rgb_leds()), [vec![0x05]]);
}

#[test]
fn set_status_led_sends_index_and_state() {
    assert_eq!(
        command_bytes(|k| k.set_status_led(2, true)),
        [vec![0x07, 2, 1]]
    );
    assert_eq!(command_bytes(|k| k.set_status_led(3, false)), [vec![0x07, 3]]);
}

#[test]
fn restore_status_leds_releases_status_led_control() {
    assert_eq!(command_bytes(|k| k.restore_status_leds()), [vec![0x0A]]);
}

#[test]
fn update_brightness_sends_one_report_per_step() {
    assert_eq!(
        command_bytes(|k| k.update_brightness(true, 3)),
        [vec![0x08, 1], vec![0x08, 1], vec![0x08, 1]]
    );
    assert_eq!(
        command_bytes(|k| k.update_brightness(false, 1)),
        [vec![0x08]]
    );
}

#[test]
fn update_brightness_takes_1_to_255_steps() {
    let (mut kontroll, sent) = open_voyager();
    let before = sent.lock().unwrap().len();
    assert!(kontroll.update_brightness(true, 0).is_err());
    assert!(kontroll.update_brightness(false, 256).is_err());
    assert!(written(&sent, before).is_empty());
}

#[test]
fn firmware_errors_come_back_as_errors() {
    let firmware = Firmware {
        rgb: false,
        ..Firmware::default()
    };
    let (kontroll, _) = open_with(firmware, 0x1977);
    let mut kontroll = kontroll.unwrap();
    assert_eq!(
        kontroll.set_rgb_all(1, 2, 3),
        Err(ApiError::Firmware(ERR_RGB_MATRIX_NOT_ENABLED))
    );

    let (mut kontroll, _) = open_voyager();
    assert_eq!(
        kontroll.set_status_led(6, true),
        Err(ApiError::Firmware(ERR_STATUS_LED_OUT_OF_RANGE))
    );
}

#[test]
fn firmware_without_protocol_version_still_works() {
    let firmware = Firmware {
        has_protocol_version: false,
        ..Firmware::default()
    };
    let (kontroll, sent) = open_with(firmware, 0x1977);
    let mut kontroll = kontroll.unwrap();
    assert_eq!(kontroll.status().keyboard.protocol_version, None);
    let before = sent.lock().unwrap().len();
    kontroll.set_layer(1).unwrap();
    assert_eq!(written(&sent, before), [vec![0x04, 0x01, 0x01]]);
}

#[test]
fn a_silent_keyboard_is_an_error() {
    let firmware = Firmware {
        answers: false,
        ..Firmware::default()
    };
    let (kontroll, _) = open_with(firmware, 0x1977);
    assert!(matches!(kontroll, Err(ApiError::NoAnswer)));
}

#[test]
fn an_unplugged_keyboard_is_an_error() {
    let info = KeyboardInfo::usb(0, 0x3297, 0x1977);
    let kontroll = Kontroll::from_connection(HidConnection::over(Unplugged), info);
    assert!(matches!(kontroll, Err(ApiError::Disconnected)));
}
