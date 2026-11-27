//! `kontroll upgrade`: replaces the running kontroll with the newest release from CrabNebula Cloud.

mod cloud;

use std::io::{self, BufRead, Cursor, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

/// kontroll's app on CrabNebula Cloud.
const APP: &str = "io-zsa-kontroll";
/// Minisign public key (Tauri format: base64 of the key file) the archives are signed with.
const PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDhENDBFRUFEMzMxRUIyQTMKUldTanNoNHpyZTVBalVxdWFOd2JPeFJWNGxXbjR6V01maXlMTU1aamhDSFhXTkpyTHlmZU9NdUUK";
const CURRENT: &str = env!("CARGO_PKG_VERSION");
const USER_AGENT: &str = concat!("kontroll/", env!("CARGO_PKG_VERSION"));

#[cfg(windows)]
const EXE_NAME: &str = "kontroll.exe";
#[cfg(not(windows))]
const EXE_NAME: &str = "kontroll";

/// Checks for a newer release and, unless `check` is set, installs it.
pub fn run(yes: bool, check: bool) -> Result<()> {
    let platform = platform_for(std::env::consts::OS, std::env::consts::ARCH).ok_or_else(|| {
        anyhow!(
            "kontroll cannot update itself on this platform ({} {}).",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let client = cloud::client(USER_AGENT)?;
    let release = match cloud::check(
        &client,
        APP,
        platform,
        CURRENT,
        &format!("kontroll release for {platform}"),
    )? {
        Some(release) if cloud::is_newer(&release.version, CURRENT)? => release,
        _ => {
            println!("kontroll {CURRENT} is up to date.");
            return Ok(());
        }
    };

    println!(
        "kontroll {} is available (you have {CURRENT}).",
        release.version
    );
    if let Some(notes) = release.notes() {
        println!();
        println!("{notes}");
        println!();
    }
    if check {
        return Ok(());
    }

    let exe = install_path()?;
    let staging = stage_next_to(&exe)?;

    if !yes {
        if !io::stdin().is_terminal() {
            println!("Run `kontroll upgrade --yes` to install it.");
            return Ok(());
        }
        if !confirm("Update now? [y/N] ")? {
            return Ok(());
        }
    }

    println!("Downloading kontroll {}...", release.version);
    let archive = cloud::download(&client, &release, PUBLIC_KEY)?;
    let binary = extract(&archive, EXE_NAME)?;

    let mut staging = staging;
    staging
        .write_all(&binary)
        .and_then(|()| staging.flush())
        .with_context(|| {
            format!(
                "could not write to {}",
                exe.parent().unwrap_or(&exe).display()
            )
        })?;
    let staged = staging.into_temp_path();
    self_replace::self_replace(&staged)
        .with_context(|| format!("could not replace {}", exe.display()))?;
    drop(staged);

    println!("Updated to {}.", release.version);
    Ok(())
}

/// The CrabNebula update platform for an OS and architecture.
fn platform_for(os: &str, arch: &str) -> Option<&'static str> {
    Some(match (os, arch) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("macos", "x86_64") => "darwin-x86_64",
        ("macos", "aarch64") => "darwin-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        ("windows", "aarch64") => "windows-aarch64",
        _ => return None,
    })
}

/// The file named `name` from a .tar.gz or .zip archive.
fn extract(archive: &[u8], name: &str) -> Result<Vec<u8>> {
    let found = if archive.starts_with(&[0x1f, 0x8b]) {
        extract_tar_gz(archive, name)?
    } else if archive.starts_with(b"PK\x03\x04") {
        extract_zip(archive, name)?
    } else {
        bail!("The download is not an archive kontroll can read.");
    };
    found.ok_or_else(|| anyhow!("The download does not contain {name}."))
}

fn extract_tar_gz(archive: &[u8], name: &str) -> Result<Option<Vec<u8>>> {
    let unreadable = |_| anyhow!("The download is not an archive kontroll can read.");
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
    for entry in tar.entries().map_err(unreadable)? {
        let mut entry = entry.map_err(unreadable)?;
        let path = entry.path().map_err(unreadable)?;
        if entry.header().entry_type().is_file() && file_name_is(&path, name) {
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(unreadable)?;
            return Ok(Some(data));
        }
    }
    Ok(None)
}

fn extract_zip(archive: &[u8], name: &str) -> Result<Option<Vec<u8>>> {
    let unreadable = |_| anyhow!("The download is not an archive kontroll can read.");
    let mut zip = zip::ZipArchive::new(Cursor::new(archive)).map_err(unreadable)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(unreadable)?;
        let matches = entry.is_file()
            && entry
                .enclosed_name()
                .is_some_and(|path| file_name_is(&path, name));
        if matches {
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|_| anyhow!("The download is not an archive kontroll can read."))?;
            return Ok(Some(data));
        }
    }
    Ok(None)
}

fn file_name_is(path: &Path, name: &str) -> bool {
    path.file_name().is_some_and(|file| file == name)
}

/// The running executable, with symlinks resolved.
fn install_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("could not locate the running kontroll")?;
    exe.canonicalize()
        .with_context(|| format!("could not locate the running kontroll ({})", exe.display()))
}

/// The package manager that owns `exe`, if its path says so.
fn package_manager(exe: &Path) -> Option<&'static str> {
    let path = exe.to_string_lossy();
    if path.starts_with("/nix/store/") {
        Some("Nix")
    } else if path.contains("/Cellar/") {
        Some("Homebrew")
    } else if path.contains("/mise/installs/") {
        Some("mise")
    } else if cfg!(target_os = "linux")
        && path.starts_with("/usr/")
        && !path.starts_with("/usr/local/")
    {
        Some("your package manager")
    } else {
        None
    }
}

/// A temporary file beside `exe`, or why kontroll cannot replace itself there.
fn stage_next_to(exe: &Path) -> Result<tempfile::NamedTempFile> {
    if let Some(manager) = package_manager(exe) {
        bail!(
            "kontroll is installed at {}, which {manager} manages. Update it through {manager}.",
            exe.display()
        );
    }
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow!("could not locate the folder kontroll is installed in"))?;
    tempfile::Builder::new()
        .prefix(".kontroll-upgrade")
        .tempfile_in(dir)
        .map_err(|err| match err.kind() {
            io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => anyhow!(
                "kontroll is installed at {}, and that folder is not writable. \
                 Run the upgrade with permission to write there.",
                exe.display()
            ),
            _ => anyhow!(err).context(format!("could not write to {}", dir.display())),
        })
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Holds `README` and `kontroll` (contents "kontroll test binary\n").
    const TAR_GZ: &[u8] = include_bytes!("../testdata/upgrade/kontroll-test.tar.gz");
    /// Holds `README` and `kontroll.exe` (contents "kontroll test binary\n").
    const ZIP: &[u8] = include_bytes!("../testdata/upgrade/kontroll-test.zip");

    #[test]
    fn platforms() {
        assert_eq!(platform_for("linux", "x86_64"), Some("linux-x86_64"));
        assert_eq!(platform_for("linux", "aarch64"), Some("linux-aarch64"));
        assert_eq!(platform_for("macos", "x86_64"), Some("darwin-x86_64"));
        assert_eq!(platform_for("macos", "aarch64"), Some("darwin-aarch64"));
        assert_eq!(platform_for("windows", "x86_64"), Some("windows-x86_64"));
        assert_eq!(platform_for("windows", "aarch64"), Some("windows-aarch64"));
        assert_eq!(platform_for("freebsd", "x86_64"), None);
        assert_eq!(platform_for("linux", "riscv64"), None);
    }

    #[test]
    fn release_key_decodes() {
        cloud::public_key(PUBLIC_KEY).unwrap();
    }

    #[test]
    fn extracts_binary_from_tar_gz() {
        assert_eq!(
            extract(TAR_GZ, "kontroll").unwrap(),
            b"kontroll test binary\n"
        );
        assert!(extract(TAR_GZ, "kontroll.exe").is_err());
    }

    #[test]
    fn extracts_binary_from_zip() {
        assert_eq!(
            extract(ZIP, "kontroll.exe").unwrap(),
            b"kontroll test binary\n"
        );
        assert!(extract(ZIP, "kontroll").is_err());
    }

    #[test]
    fn rejects_unknown_archive() {
        let err = extract(b"plain bytes", "kontroll").unwrap_err();
        assert_eq!(
            err.to_string(),
            "The download is not an archive kontroll can read."
        );
    }

    #[test]
    fn refuses_nix_store() {
        let err =
            stage_next_to(Path::new("/nix/store/abc-kontroll-2.0.0/bin/kontroll")).unwrap_err();
        assert!(err.to_string().contains("which Nix manages"));
    }

    #[test]
    fn package_managers() {
        let owner = |path: &str| package_manager(Path::new(path));
        assert_eq!(
            owner("/nix/store/abc-kontroll-2.0.0/bin/kontroll"),
            Some("Nix")
        );
        assert_eq!(
            owner("/opt/homebrew/Cellar/kontroll/2.0.0/bin/kontroll"),
            Some("Homebrew")
        );
        assert_eq!(
            owner("/home/u/.local/share/mise/installs/github-zsa-kontroll/2.0.0/kontroll"),
            Some("mise")
        );
        assert_eq!(owner("/usr/local/bin/kontroll"), None);
        assert_eq!(owner("/home/u/.cargo/bin/kontroll"), None);
        if cfg!(target_os = "linux") {
            assert_eq!(owner("/usr/bin/kontroll"), Some("your package manager"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn refuses_read_only_folder() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        let result = stage_next_to(&dir.path().join("kontroll"));
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.unwrap_err().to_string().contains("not writable"));
    }

    #[test]
    fn stages_beside_executable() {
        let dir = tempfile::tempdir().unwrap();
        let staged = stage_next_to(&dir.path().join("kontroll")).unwrap();
        assert_eq!(staged.path().parent(), Some(dir.path()));
    }
}
