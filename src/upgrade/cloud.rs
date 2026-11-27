//! Releases on CrabNebula Cloud.
//!
//! The server answers `GET <org>/<app>/<platform>/<current version>` with 204
//! when `current` string-equals its latest release, and with that release
//! otherwise, older or not. Version ordering is done here, with semver.
//! Every download is checked against a minisign signature (Tauri format).

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use serde::Deserialize;

/// ZSA's update endpoints on CrabNebula Cloud.
pub const ORG_ENDPOINT: &str = "https://cdn.crabnebula.app/update/zsa-technology-labs-inc";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// What the update server offers.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Release {
    pub version: String,
    #[serde(default)]
    pub notes: Option<String>,
    pub url: String,
    pub signature: String,
}

impl Release {
    /// The notes, trimmed, or `None` when there are none.
    pub fn notes(&self) -> Option<&str> {
        self.notes
            .as_deref()
            .map(str::trim)
            .filter(|notes| !notes.is_empty())
    }
}

/// An HTTP client with the update server's timeouts.
pub fn client(user_agent: &str) -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("could not set up the HTTP client")
}

/// The URL that asks for `app`'s release on `platform`.
pub fn release_url(app: &str, platform: &str, current: &str) -> String {
    format!("{ORG_ENDPOINT}/{app}/{platform}/{current}")
}

/// The release the server offers against `current`, or `None` for 204.
/// `subject` names the release in messages ("kontroll release for linux-x86_64").
pub fn check(
    client: &reqwest::blocking::Client,
    app: &str,
    platform: &str,
    current: &str,
    subject: &str,
) -> Result<Option<Release>> {
    let response = client
        .get(release_url(app, platform, current))
        .send()
        .map_err(network_error)?;
    let status = response.status().as_u16();
    let body = response.bytes().map_err(network_error)?;
    parse_response(status, &body, subject)
}

/// Downloads a release and checks it against its signature and `public_key`.
pub fn download(
    client: &reqwest::blocking::Client,
    release: &Release,
    public_key: &str,
) -> Result<Vec<u8>> {
    let response = client.get(&release.url).send().map_err(network_error)?;
    let status = response.status();
    if !status.is_success() {
        bail!("The download failed (HTTP {}).", status.as_u16());
    }
    let data = response.bytes().map_err(network_error)?.to_vec();
    verify(&data, &release.signature, public_key)
        .map_err(|_| anyhow!("The download's signature does not match. Nothing was changed."))?;
    Ok(data)
}

/// The release in a reply, or `None` for 204 (nothing newer).
pub fn parse_response(status: u16, body: &[u8], subject: &str) -> Result<Option<Release>> {
    match status {
        200 => serde_json::from_slice(body)
            .map(Some)
            .map_err(|_| anyhow!("The update server sent an unreadable reply.")),
        204 => Ok(None),
        400 | 404 => bail!("The update server has no {subject}."),
        other => bail!("The update server answered with HTTP {other}."),
    }
}

/// Whether `offered` is a later version than `current`.
pub fn is_newer(offered: &str, current: &str) -> Result<bool> {
    let parse = |v: &str| {
        semver::Version::parse(v.trim().trim_start_matches('v'))
            .map_err(|_| anyhow!("The update server sent an unreadable version: {v}."))
    };
    Ok(parse(offered)? > parse(current)?)
}

/// Checks `data` against a Tauri-format minisign signature and public key.
pub fn verify(data: &[u8], signature_b64: &str, public_key_b64: &str) -> Result<()> {
    let key = public_key(public_key_b64)?;
    let signature = minisign_verify::Signature::decode(&decode(signature_b64)?)?;
    key.verify(data, &signature, false)?;
    Ok(())
}

/// A Tauri-format minisign public key.
pub fn public_key(b64: &str) -> Result<minisign_verify::PublicKey> {
    Ok(minisign_verify::PublicKey::decode(&decode(b64)?)?)
}

fn decode(b64: &str) -> Result<String> {
    Ok(String::from_utf8(B64.decode(b64.trim())?)?)
}

fn network_error(err: reqwest::Error) -> anyhow::Error {
    if err.is_timeout() {
        anyhow!("The update server did not answer in time.")
    } else if err.is_connect() {
        anyhow!("Cannot reach the update server. Check the internet connection.")
    } else {
        anyhow!("The update request failed: {err}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// zapp's updater key, which did not sign the fixtures.
    const OTHER_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEU5QkQ1OUJGNzc1MEQzMApSV1F3RFhYM205V2JEdW5RU3pyZWptK3VwbXVpc2Y1TVJHMFZsb0N4enl2RlNQUGR6ajVYSGJRbgo=";
    const TAR_GZ: &[u8] = include_bytes!("../../testdata/upgrade/kontroll-test.tar.gz");
    const TAR_GZ_SIG: &str = include_str!("../../testdata/upgrade/kontroll-test.tar.gz.sig");
    const ZIP: &[u8] = include_bytes!("../../testdata/upgrade/kontroll-test.zip");
    const ZIP_SIG: &str = include_str!("../../testdata/upgrade/kontroll-test.zip.sig");

    fn release(version: &str) -> Release {
        Release {
            version: version.into(),
            notes: None,
            url: "u".into(),
            signature: "s".into(),
        }
    }

    #[test]
    fn newer_versions() {
        assert!(is_newer("2.0.1", "2.0.0").unwrap());
        assert!(is_newer("2.1.0", "2.0.9").unwrap());
        assert!(is_newer("v2.0.1", "2.0.0").unwrap());
        assert!(!is_newer("2.0.0", "2.0.0").unwrap());
        assert!(!is_newer("1.0.9", "2.0.0").unwrap());
        assert!(!is_newer("2.0.1-beta.1", "2.0.1").unwrap());
        assert!(is_newer("nonsense", "2.0.0").is_err());
    }

    #[test]
    fn no_content_means_up_to_date() {
        assert_eq!(
            parse_response(204, b"", "kontroll release for linux-x86_64").unwrap(),
            None
        );
    }

    #[test]
    fn release_reply() {
        let body = br#"{"version":"2.0.1","notes":"- Fixes","pub_date":"2026-11-25T00:00:00Z",
            "url":"https://cdn.example/kontroll.tar.gz","signature":"c2ln","format":"tar.gz"}"#;
        let release = parse_response(200, body, "kontroll release for linux-x86_64")
            .unwrap()
            .unwrap();
        assert_eq!(
            release,
            Release {
                version: "2.0.1".into(),
                notes: Some("- Fixes".into()),
                url: "https://cdn.example/kontroll.tar.gz".into(),
                signature: "c2ln".into(),
            }
        );
    }

    #[test]
    fn release_reply_without_notes() {
        let body = br#"{"version":"2.0.1","url":"u","signature":"s"}"#;
        let release = parse_response(200, body, "kontroll release for linux-x86_64")
            .unwrap()
            .unwrap();
        assert_eq!(release.notes, None);
        assert_eq!(release.notes(), None);
    }

    #[test]
    fn blank_notes_read_as_none() {
        let mut release = release("2.0.1");
        release.notes = Some("  \n".into());
        assert_eq!(release.notes(), None);
        release.notes = Some("\n- Fixes\n".into());
        assert_eq!(release.notes(), Some("- Fixes"));
    }

    #[test]
    fn bad_replies() {
        let subject = "kontroll release for linux-x86_64";
        assert!(parse_response(200, b"<html>", subject).is_err());
        let err = parse_response(400, b"", subject).unwrap_err();
        assert_eq!(
            err.to_string(),
            "The update server has no kontroll release for linux-x86_64."
        );
        let err = parse_response(404, b"", subject).unwrap_err();
        assert_eq!(
            err.to_string(),
            "The update server has no kontroll release for linux-x86_64."
        );
        let err = parse_response(500, b"", subject).unwrap_err();
        assert_eq!(err.to_string(), "The update server answered with HTTP 500.");
    }

    #[test]
    fn feed_url() {
        assert_eq!(
            release_url("io-zsa-kontroll", "linux-x86_64", "2.0.0"),
            "https://cdn.crabnebula.app/update/zsa-technology-labs-inc/io-zsa-kontroll/linux-x86_64/2.0.0"
        );
    }

    #[test]
    fn signature_accepts_signed_archives() {
        verify(TAR_GZ, TAR_GZ_SIG, super::super::PUBLIC_KEY).unwrap();
        verify(ZIP, ZIP_SIG, super::super::PUBLIC_KEY).unwrap();
    }

    #[test]
    fn signature_rejects_tampered_archive() {
        let mut tampered = TAR_GZ.to_vec();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        assert!(verify(&tampered, TAR_GZ_SIG, super::super::PUBLIC_KEY).is_err());
    }

    #[test]
    fn signature_rejects_other_archive() {
        assert!(verify(ZIP, TAR_GZ_SIG, super::super::PUBLIC_KEY).is_err());
    }

    #[test]
    fn signature_rejects_other_key() {
        assert!(verify(TAR_GZ, TAR_GZ_SIG, OTHER_PUBLIC_KEY).is_err());
    }

    #[test]
    fn signature_rejects_garbage() {
        assert!(verify(TAR_GZ, "not base64!", super::super::PUBLIC_KEY).is_err());
        assert!(verify(TAR_GZ, TAR_GZ_SIG, "").is_err());
    }
}
