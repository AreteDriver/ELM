use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use tokio::process::Command;

pub async fn ensure_prefix_initialized(prefix_dir: &Path, proton_root: &Path) -> Result<()> {
    fs::create_dir_all(prefix_dir).with_context(|| format!("create prefix {}", prefix_dir.display()))?;

    // Heuristic: if pfx/drive_c exists, assume initialized (Proton layout)
    if prefix_dir.join("pfx/drive_c").exists() {
        return Ok(());
    }

    let proton = proton_root.join("proton");
    if !proton.exists() {
        return Err(anyhow!("proton runner not found at {}", proton.display()));
    }

    // Find Steam installation path
    let home = std::env::var("HOME").unwrap_or_default();
    let steam_path = format!("{home}/.steam/steam");

    // Initialize prefix with required Proton environment
    let status = Command::new("python3")
        .arg(&proton)
        .env("STEAM_COMPAT_DATA_PATH", prefix_dir)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_path)
        .env("WINEPREFIX", prefix_dir.join("pfx"))
        .arg("run")
        .arg("wineboot")
        .status()
        .await
        .context("running wineboot")?;

    if !status.success() {
        return Err(anyhow!("wineboot failed with status: {status}"));
    }
    Ok(())
}
