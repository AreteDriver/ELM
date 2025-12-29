use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

pub struct LaunchSpec {
    pub proton_root: PathBuf,
    pub prefix_dir: PathBuf,
    pub exe_path_in_prefix: PathBuf, // relative to pfx/, e.g. drive_c/.../evelauncher.exe
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

pub async fn launch(spec: LaunchSpec) -> Result<()> {
    let proton = spec.proton_root.join("proton");
    if !proton.exists() {
        return Err(anyhow!("proton not found: {}", proton.display()));
    }

    // Proton uses pfx/ subdirectory for the actual Wine prefix
    let exe_abs = spec.prefix_dir.join("pfx").join(&spec.exe_path_in_prefix);
    if !exe_abs.exists() {
        return Err(anyhow!("exe not found: {}", exe_abs.display()));
    }

    // Find Steam installation path
    let home = std::env::var("HOME").unwrap_or_default();
    let steam_path = format!("{home}/.steam/steam");

    let mut cmd = Command::new("python3");
    cmd.arg(&proton);
    cmd.env("STEAM_COMPAT_DATA_PATH", &spec.prefix_dir);
    cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_path);

    for (k, v) in &spec.env {
        cmd.env(k, v);
    }

    cmd.arg("run").arg(exe_abs);

    for a in &spec.args {
        cmd.arg(a);
    }

    let status = cmd.status().await.context("launch proton run")?;
    if !status.success() {
        return Err(anyhow!("launch failed with status: {status}"));
    }
    Ok(())
}

/// Launch EVE in background (for multiboxing) - spawns process and returns immediately
pub fn launch_background(spec: LaunchSpec) -> Result<()> {
    let proton = spec.proton_root.join("proton");
    if !proton.exists() {
        return Err(anyhow!("proton not found: {}", proton.display()));
    }

    // Proton uses pfx/ subdirectory for the actual Wine prefix
    let exe_abs = spec.prefix_dir.join("pfx").join(&spec.exe_path_in_prefix);
    if !exe_abs.exists() {
        return Err(anyhow!("exe not found: {}", exe_abs.display()));
    }

    // Find Steam installation path
    let home = std::env::var("HOME").unwrap_or_default();
    let steam_path = format!("{home}/.steam/steam");

    let mut cmd = std::process::Command::new("python3");
    cmd.arg(&proton);
    cmd.env("STEAM_COMPAT_DATA_PATH", &spec.prefix_dir);
    cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_path);

    for (k, v) in &spec.env {
        cmd.env(k, v);
    }

    cmd.arg("run").arg(exe_abs);

    for a in &spec.args {
        cmd.arg(a);
    }

    // Spawn without waiting - process runs independently
    cmd.spawn().context("spawn proton run")?;
    Ok(())
}
