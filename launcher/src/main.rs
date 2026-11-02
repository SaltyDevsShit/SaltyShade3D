//! SaltyShade 3D launcher: what the desktop and Start menu icons open.
//!
//! 1. Asks GitHub for the newest SaltyShade 3D release (`update.json` names the repository).
//! 2. If it is newer than the installed one (`version.txt`), downloads its installer and runs it
//!    silently; the installer closes the app, updates it and starts this launcher again with
//!    `--no-update`, which opens the app.
//! 3. Otherwise (or offline, or GitHub slow) it opens the app straight away.
//!
//! Downloads go through Windows' own curl.exe, so the launcher carries no TLS stack.

#![windows_subsystem = "windows"]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const APP_EXE: &str = "SaltyShade3D.exe";
/// No console window for curl.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn main() {
    let dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_else(|| PathBuf::from("."));
    let skip = std::env::args().any(|a| a == "--no-update");
    if !skip {
        if let Some(installer) = pending_update(&dir) {
            // The installer takes over: it closes the app, updates it and relaunches us.
            if Command::new(&installer).args(["/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CLOSEAPPLICATIONS"]).spawn().is_ok() {
                return;
            }
        }
    }
    let _ = Command::new(dir.join(APP_EXE)).current_dir(&dir).spawn();
}

/// The newest release's installer, downloaded, when it differs from what is installed.
fn pending_update(dir: &Path) -> Option<PathBuf> {
    let repo = read_repo(dir)?;
    let installed = fs::read_to_string(dir.join("version.txt")).unwrap_or_default().trim().to_string();
    let api = format!("https://api.github.com/repos/{repo}/releases/latest");
    let body = curl(&["-sSfL", "--max-time", "4", "-H", "Accept: application/vnd.github+json", "-H", "User-Agent: SaltyShade3D-Launcher", &api])?;
    let release: serde_json::Value = serde_json::from_slice(&body).ok()?;
    let tag = release["tag_name"].as_str()?.trim_start_matches('v');
    if tag.is_empty() || tag == installed {
        return None;
    }
    let url = release["assets"].as_array()?.iter().find_map(|a| {
        let name = a["name"].as_str()?;
        name.ends_with("-Setup.exe").then(|| a["browser_download_url"].as_str()).flatten()
    })?;

    let tmp = std::env::temp_dir().join("SaltyShade3D-update");
    fs::create_dir_all(&tmp).ok()?;
    // One download at a time, should the icon be double-clicked again meanwhile.
    let lock = tmp.join("download.lock");
    let fresh_lock = fs::metadata(&lock).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age.as_secs() < 600);
    if fresh_lock {
        return None;
    }
    fs::write(&lock, tag).ok()?;
    let installer = tmp.join(format!("SaltyShade3D-{tag}-Setup.exe"));
    let ok = curl(&["-sSfL", "--max-time", "900", "-H", "User-Agent: SaltyShade3D-Launcher", "-o", &installer.to_string_lossy(), url]).is_some();
    let _ = fs::remove_file(&lock);
    // A real installer is megabytes; anything smaller is an error page.
    (ok && fs::metadata(&installer).is_ok_and(|m| m.len() > 1_000_000)).then_some(installer)
}

/// `owner/name` from update.json, unless it is still the placeholder.
fn read_repo(dir: &Path) -> Option<String> {
    let cfg: serde_json::Value = serde_json::from_slice(&fs::read(dir.join("update.json")).ok()?).ok()?;
    let repo = cfg["repo"].as_str()?.trim();
    (repo.contains('/') && !repo.contains("YOUR_")).then(|| repo.to_string())
}

fn curl(args: &[&str]) -> Option<Vec<u8>> {
    let mut cmd = Command::new("curl.exe");
    cmd.args(args).stdin(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().ok()?;
    out.status.success().then_some(out.stdout)
}
