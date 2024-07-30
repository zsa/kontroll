//! Kontroll connects to Keymapp's API, allowing control of ZSAs keyboard programmaticaly.
//!
//! For more information or examples on how to use it, please refer to the [repository](https://github.com/zsa/kontroll)
//!
//! ## Usage
//! ```rust,no_run
//! use kontroll::Kontroll;
//! // Note: The port is optional, if not provided, it will default to
//! // "$CONFIG_DIR/.keymapp/keymapp.sock" on Unix and 50051 on Windows.
//! let port = None
//! let api = Kontroll::new(port).await.unwrap();
//! let keyboards = api.list_keyboards().await.unwrap();
//! ```
//! The above example will list all the keyboards connected to the system. You can
//! check other available methods in the [`Kontroll`] struct.
pub mod api;
pub mod utils;

pub use api::Kontroll;
