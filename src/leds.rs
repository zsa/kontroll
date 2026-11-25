//! RGB LED indices: Kontroll's index to the firmware's.
//!
//! The Moonlander and ErgoDox EZ (STM32) tables are the ones Keymapp 1.3.7
//! applied before sending `SET_RGB_LED`, so scripts written for Kontroll 1.x
//! address the same LEDs. LED counts come from QMK's `keyboard.json` /
//! `info.json` (`rgb_matrix.layout`) for each board.

use zapp_core::device::ids::Keyboard;

use crate::ApiError;

/// Moonlander (PIDs 0x1969, 0x1972): Kontroll index -> firmware index.
const MOONLANDER: [u8; 72] = [
    0, 5, 10, 15, 20, 25, 29, 1, 6, 11, 16, 21, 26, 30, 2, 7, 12, 17, 22, 27, 31, 3, 8, 13, 18,
    23, 28, 4, 9, 14, 19, 24, 35, 32, 33, 34, 65, 61, 56, 51, 46, 41, 36, 66, 62, 57, 52, 47, 42,
    37, 67, 63, 58, 53, 48, 43, 38, 64, 59, 54, 49, 44, 39, 60, 55, 50, 45, 40, 71, 70, 69, 68,
];

/// ErgoDox EZ on STM32 (PIDs 0x2010, 0x2020, 0x2030): Kontroll index ->
/// firmware index.
const ERGODOX_STM32: [u8; 48] = [
    28, 27, 26, 25, 24, 33, 32, 31, 30, 29, 38, 37, 36, 35, 34, 43, 42, 41, 40, 39, 47, 46, 45,
    44, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
];

/// Voyager and Wireless Voyager `rgb_matrix.layout` length.
const VOYAGER_LEDS: usize = 52;

/// Planck EZ Glow (PID 0xC6CF) `rgb_matrix.layout` length.
const PLANCK_GLOW_LEDS: usize = 47;

/// How a keyboard numbers its RGB LEDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LedMap {
    /// Kontroll's index is the firmware's, below this count.
    Direct(usize),
    /// Kontroll's index picks the firmware's from this table.
    Table(&'static [u8]),
    /// Unknown board: the index is sent as given.
    Unknown,
}

impl LedMap {
    pub(crate) fn for_keyboard(keyboard: Option<Keyboard>, pid: Option<u16>) -> Self {
        match keyboard {
            Some(Keyboard::Voyager | Keyboard::WirelessVoyager) => Self::Direct(VOYAGER_LEDS),
            Some(Keyboard::Moonlander) => Self::Table(&MOONLANDER),
            Some(Keyboard::ErgodoxEz) if matches!(pid, Some(0x2010 | 0x2020 | 0x2030)) => {
                Self::Table(&ERGODOX_STM32)
            }
            Some(Keyboard::PlanckEz) if pid == Some(0xC6CF) => Self::Direct(PLANCK_GLOW_LEDS),
            _ => Self::Unknown,
        }
    }

    /// The firmware index for `index`.
    pub(crate) fn firmware_index(self, index: usize) -> Result<u8, ApiError> {
        let out_of_range = |count: usize| {
            ApiError::InvalidArgument(format!(
                "LED {index} does not exist on this keyboard: it has LEDs 0 to {}.",
                count - 1
            ))
        };
        match self {
            Self::Direct(count) if index < count => Ok(index as u8),
            Self::Direct(count) => Err(out_of_range(count)),
            Self::Table(table) => table
                .get(index)
                .copied()
                .ok_or_else(|| out_of_range(table.len())),
            Self::Unknown => u8::try_from(index).map_err(|_| out_of_range(256)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_permutation(table: &[u8]) -> bool {
        let mut sorted = table.to_vec();
        sorted.sort_unstable();
        sorted.iter().enumerate().all(|(i, &v)| v as usize == i)
    }

    #[test]
    fn tables_cover_every_led_once() {
        assert!(is_permutation(&MOONLANDER));
        assert!(is_permutation(&ERGODOX_STM32));
    }

    #[test]
    fn voyager_indices_pass_through_up_to_its_led_count() {
        let map = LedMap::for_keyboard(Some(Keyboard::Voyager), Some(0x1977));
        assert_eq!(map.firmware_index(51), Ok(51));
        assert!(map.firmware_index(52).is_err());
    }

    #[test]
    fn moonlander_indices_are_remapped() {
        let map = LedMap::for_keyboard(Some(Keyboard::Moonlander), Some(0x1972));
        assert_eq!(map.firmware_index(1), Ok(5));
        assert_eq!(map.firmware_index(71), Ok(68));
        assert!(map.firmware_index(72).is_err());
    }

    #[test]
    fn only_stm32_ergodox_boards_are_remapped() {
        let stm32 = LedMap::for_keyboard(Some(Keyboard::ErgodoxEz), Some(0x2010));
        assert_eq!(stm32.firmware_index(0), Ok(28));
        let teensy = LedMap::for_keyboard(Some(Keyboard::ErgodoxEz), Some(0x4976));
        assert_eq!(teensy, LedMap::Unknown);
    }

    #[test]
    fn unknown_boards_take_any_byte() {
        assert_eq!(LedMap::Unknown.firmware_index(255), Ok(255));
        assert!(LedMap::Unknown.firmware_index(256).is_err());
    }
}
