use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::config::models::ManifestV1;

/// Download a file from URL to destination
fn download_file(url: &str, dest: &Path) -> Result<()> {
    println!("Downloading: {}", url);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let mut resp = client.get(url).send().with_context(|| format!("GET {url}"))?;
    resp.error_for_status_ref()?;

    let mut out = File::create(dest).with_context(|| format!("create {}", dest.display()))?;
    let mut buf = [0u8; 1024 * 64];
    let mut total = 0u64;

    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 { break; }
        out.write_all(&buf[..n])?;
        total += n as u64;
        if total % (1024 * 1024) == 0 {
            println!("  Downloaded {} MB", total / (1024 * 1024));
        }
    }
    println!("  Complete: {} bytes", total);
    Ok(())
}

/// Install game from manifest into prefix
pub async fn install_from_manifest(
    manifest: &ManifestV1,
    prefix_dir: &Path,
    proton_root: &Path,
    downloads_dir: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(downloads_dir)?;

    // Download the installer
    let installer_filename = manifest.installer.source.url
        .split('/')
        .last()
        .unwrap_or("installer.exe");
    let installer_path = downloads_dir.join(installer_filename);

    if !installer_path.exists() {
        download_file(&manifest.installer.source.url, &installer_path)?;
    } else {
        println!("Using cached installer: {}", installer_path.display());
    }

    // Verify SHA256 if provided
    if let Some(expected_sha) = &manifest.installer.source.sha256 {
        verify_sha256(&installer_path, expected_sha)?;
    }

    // Create install directory in prefix
    let install_dir = prefix_dir
        .join("pfx/drive_c")
        .join(&manifest.installer.install_dir);
    fs::create_dir_all(&install_dir)?;

    // Run the installer with Proton
    println!("Running installer...");
    run_installer(&installer_path, prefix_dir, proton_root).await?;

    Ok(install_dir)
}

fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
    use sha2::{Digest, Sha256};

    let mut f = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 128];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let got = hex::encode(hasher.finalize());
    if got.to_lowercase() != expected_hex.to_lowercase() {
        return Err(anyhow!(
            "sha256 mismatch for {}: expected {}, got {}",
            path.display(),
            expected_hex,
            got
        ));
    }
    println!("SHA256 verified: {}", expected_hex);
    Ok(())
}

async fn run_installer(
    installer_exe: &Path,
    prefix_dir: &Path,
    proton_root: &Path,
) -> Result<()> {
    let proton = proton_root.join("proton");
    if !proton.exists() {
        return Err(anyhow!("proton not found: {}", proton.display()));
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let steam_path = format!("{home}/.steam/steam");

    let status = Command::new("python3")
        .arg(&proton)
        .env("STEAM_COMPAT_DATA_PATH", prefix_dir)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_path)
        .arg("run")
        .arg(installer_exe)
        .status()
        .await
        .context("running installer")?;

    if !status.success() {
        return Err(anyhow!("installer failed with status: {status}"));
    }

    Ok(())
}

/// Run silent/unattended install for EVE launcher specifically
pub async fn install_eve_launcher(
    prefix_dir: &Path,
    proton_root: &Path,
    downloads_dir: &Path,
) -> Result<PathBuf> {
    const EVE_LAUNCHER_URL: &str = "https://launcher.ccpgames.com/eve-online/release/win32/x64/eve-online-1.9.4+Setup.exe";

    fs::create_dir_all(downloads_dir)?;

    let installer_path = downloads_dir.join("eve-online-1.9.4+Setup.exe");

    if !installer_path.exists() {
        download_file(EVE_LAUNCHER_URL, &installer_path)?;
    } else {
        println!("Using cached installer: {}", installer_path.display());
    }

    // Create EVE directory structure
    let eve_dir = prefix_dir.join("pfx/drive_c/EVE");
    fs::create_dir_all(&eve_dir)?;

    // Run the installer
    println!("Running EVE Launcher installer...");
    println!("Note: Complete the installer GUI when it appears.");

    let proton = proton_root.join("proton");
    let home = std::env::var("HOME").unwrap_or_default();
    let steam_path = format!("{home}/.steam/steam");

    let status = Command::new("python3")
        .arg(&proton)
        .env("STEAM_COMPAT_DATA_PATH", prefix_dir)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_path)
        .arg("run")
        .arg(&installer_path)
        .status()
        .await
        .context("running EVE installer")?;

    if !status.success() {
        return Err(anyhow!("EVE installer failed with status: {status}"));
    }

    // Check common install locations
    let launcher_path = prefix_dir.join("pfx/drive_c/EVE/Launcher/evelauncher.exe");
    let alt_path = prefix_dir.join("pfx/drive_c/Program Files/CCP/EVE/Launcher/evelauncher.exe");

    if launcher_path.exists() {
        println!("EVE Launcher installed at: {}", launcher_path.display());
        Ok(launcher_path)
    } else if alt_path.exists() {
        println!("EVE Launcher installed at: {}", alt_path.display());
        Ok(alt_path)
    } else {
        println!("Warning: Could not verify launcher location. Check prefix manually.");
        Ok(eve_dir)
    }
}
