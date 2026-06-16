//! Signed self-update. Mirrors the Slint sibling: the app checks a small JSON
//! feed published with each GitHub release, and - when a newer build exists -
//! downloads just the new executable, verifies its minisign signature against the
//! embedded public key, and swaps it in place (the per-user install dir is
//! writable without elevation). A tampered or unsigned download is rejected
//! before it ever replaces the running binary. Only the app binary is fetched;
//! any bundled sidecar from the installer is left untouched.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// GitHub `owner/repo` that publishes Vayou releases.
const REPO: &str = "0hgawa/vayou-desktop";

/// The release feed: a small JSON manifest uploaded with each GitHub release.
/// Schema: `{ version, platforms: { "windows-x86_64": { url, signature } } }`,
/// where `signature` is the raw minisign `.minisig` text for the new executable.
const UPDATE_FEED: &str =
    "https://github.com/0hgawa/vayou-desktop/releases/latest/download/latest.json";

/// minisign public key the downloaded executable must be signed with. The
/// matching secret key (`.keys/vayou-desktop.key`) signs it at release time.
const UPDATE_PUBKEY: &str = "RWQZ44cVRYzob680SJHG6MGpWBsknSRubx28ElUU0tcXTQC1QH74X3Mb";

/// A newer release: version, the new executable URL, and its minisign signature.
/// Round-trips through the frontend between `check` and `install`.
#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
    pub signature: String,
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(concat!("Vayou/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())
}

/// `(major, minor, patch)` from a `v1.2.3`-ish tag, ignoring pre-release/build.
fn semver(v: &str) -> (u32, u32, u32) {
    let mut it = v.trim().trim_start_matches('v').split(['.', '-', '+']);
    let mut next = || it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (next(), next(), next())
}

/// Check the release feed; `Some(UpdateInfo)` when a newer build is published.
pub async fn check() -> Result<Option<UpdateInfo>, String> {
    let resp = client()?.get(UPDATE_FEED).send().await.map_err(|e| e.to_string())?;
    // No release (or no feed asset) published yet -> nothing newer than us.
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let version = json.get("version").and_then(|v| v.as_str()).ok_or("no version in feed")?;
    if semver(version) <= semver(env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }
    let plat = json.pointer("/platforms/windows-x86_64").ok_or("no windows-x86_64 in feed")?;
    let url = plat.get("url").and_then(|v| v.as_str()).ok_or("no url in feed")?;
    let signature = plat.get("signature").and_then(|v| v.as_str()).ok_or("no signature in feed")?;
    Ok(Some(UpdateInfo {
        version: version.to_owned(),
        url: url.to_owned(),
        signature: signature.to_owned(),
    }))
}

/// Download the new executable, verify its minisign signature against the
/// embedded public key, and swap it in for the running executable in place. The
/// caller then relaunches so the new image takes over.
pub async fn download_and_apply(info: &UpdateInfo) -> Result<(), String> {
    let bytes = client()?
        .get(&info.url)
        .send().await.map_err(|e| e.to_string())?
        .error_for_status().map_err(|e| e.to_string())?
        .bytes().await.map_err(|e| e.to_string())?;
    verify_signature(&bytes, &info.signature)?;
    // Stage + swap touch the disk and rename the running exe - keep them off the
    // async reactor.
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let staged = std::env::temp_dir().join("vayou-update.exe");
        std::fs::write(&staged, &bytes).map_err(|e| format!("write update: {e}"))?;
        self_replace::self_replace(&staged).map_err(|e| format!("replace executable: {e}"))?;
        let _ = std::fs::remove_file(&staged);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Reject a download whose minisign signature doesn't match the embedded key.
fn verify_signature(bytes: &[u8], signature: &str) -> Result<(), String> {
    use minisign_verify::{PublicKey, Signature};
    let pk = PublicKey::from_base64(UPDATE_PUBKEY).map_err(|e| format!("public key: {e}"))?;
    let sig = Signature::decode(signature).map_err(|e| format!("signature: {e}"))?;
    pk.verify(bytes, &sig, false)
        .map_err(|_| "signature does not match - refusing to install the download".to_owned())
}

/// Open the releases page in the default browser. Offered as a fallback when the
/// in-app update can't be applied.
pub fn open_release_page() {
    let url = format!("https://github.com/{REPO}/releases");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&url).spawn();
}

#[cfg(test)]
mod tests {
    use super::semver;

    #[test]
    fn parses_versions_ignoring_prefix_and_suffix() {
        assert_eq!(semver("v1.2.3"), (1, 2, 3));
        assert_eq!(semver("1.2.3"), (1, 2, 3));
        assert_eq!(semver("v2.0.0-rc1"), (2, 0, 0));
        assert_eq!(semver("1.4.0+build7"), (1, 4, 0));
        assert_eq!(semver("v2"), (2, 0, 0));
        assert_eq!(semver("garbage"), (0, 0, 0));
    }

    #[test]
    fn newer_versions_compare_greater() {
        assert!(semver("v1.0.1") > semver("v1.0.0"));
        assert!(semver("v2.0.0") > semver("v1.9.9"));
        assert!(semver("v1.0.0") <= semver("v1.0.0"));
    }
}
