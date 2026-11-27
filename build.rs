//! Embeds an Info.plist in the macOS binary when Bluetooth is built in.
//!
//! The plist carries the Bluetooth usage description that macOS shows in its
//! permission prompt, in `__TEXT,__info_plist`.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos")
        || env::var_os("CARGO_FEATURE_BLE").is_none()
    {
        return;
    }

    let version = env::var("CARGO_PKG_VERSION").expect("cargo sets the package version");
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleIdentifier</key>
	<string>com.zsa.kontroll</string>
	<key>CFBundleName</key>
	<string>kontroll</string>
	<key>CFBundleShortVersionString</key>
	<string>{version}</string>
	<key>CFBundleVersion</key>
	<string>{version}</string>
	<key>NSBluetoothAlwaysUsageDescription</key>
	<string>kontroll talks to ZSA keyboards over Bluetooth to switch layers and set their lights.</string>
</dict>
</plist>
"#
    );

    let path = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("Info.plist");
    fs::write(&path, plist).expect("write Info.plist");
    println!(
        "cargo:rustc-link-arg-bins=-Wl,-sectcreate,__TEXT,__info_plist,{}",
        path.display()
    );
}
