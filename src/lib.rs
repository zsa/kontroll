//! Kontroll controls ZSA keyboards directly: layers, RGB and status LEDs, and
//! brightness, over USB or Bluetooth (Wireless Voyager). No Keymapp needed.
//!
//! For more information or examples on how to use it, please refer to the [repository](https://github.com/zsa/kontroll)
//!
//! ## Usage
//! ```rust,no_run
//! use kontroll::Kontroll;
//!
//! // Every keyboard Kontroll can see, USB first, then Bluetooth.
//! for keyboard in Kontroll::list().unwrap() {
//!     println!("{keyboard}");
//! }
//!
//! // Open the first keyboard (or `Some(index)` from the list above).
//! let mut keyboard = Kontroll::open(None).unwrap();
//! keyboard.set_layer(1).unwrap();
//! keyboard.set_rgb_all(255, 0, 0).unwrap();
//! keyboard.restore_rgb_leds().unwrap();
//! ```
//! The API is blocking. Each call returns once the keyboard has received the
//! command; dropping the [`Kontroll`] closes the connection. The keyboard keeps
//! the layer and LED state after that.
//!
//! Two identical USB keyboards (same model) cannot be told apart: only the
//! first of them can be opened.
pub mod api;
mod error;
mod leds;
pub mod utils;

pub use api::{ConnectedKeyboard, KeyboardInfo, Kontroll, Status, Target, Transport};
pub use error::ApiError;
pub use zapp_core::device::ids::Keyboard;
