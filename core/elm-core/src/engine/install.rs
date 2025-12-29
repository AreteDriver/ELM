// src/engine/install.rs
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::config::models::EngineV1;

pub struct EnginePaths {
    pub root: PathBuf,
    pub dist: PathBuf,
    pub marker: PathBuf,
}

pub fn engine_paths(engines_dir: &Path, engine_id: &str) -> EnginePaths {
    let root = engines_dir.join(engine_id);
    EnginePaths {
        dist: root.join("dist"),
        marker: root.join("installed.json"),
        root,
    }
}

pub fn ensure_engine_installed(engine: &EngineV1, engines_dir: &Path, downloads_dir: &Path) -> Result<PathBuf> {
    let p = engine_paths(engines_dir, &engine.id);
    if p.marker.exists() && p.dist.exists() {
        return Ok(p.dist.clone());
    }
    fs::create_dir_all(&p.root)?;
    fs::create_dir_all(&p.dist)?;
    fs::create_dir_all(downloads_dir)?;

    if engine.source.kind != "url" {
        return Err(anyhow!("v1 engine source.kind must be 'url'"));
    }

    let archive_path = downloads_dir.join(format!("{}.tar.gz", engine.id));
    download_to_file(&engine.source.url, &archive_path)?;

    verify_sha256(&archive_path, &engine.sha256)?;

    extract_tar_gz(&archive_path, &p.dist)?;

    let marker = serde_json::json!({
        "engine_id": engine.id,
        "sha256": engine.sha256
    });
    std::fs::write(&p.marker, serde_json::to_vec_pretty(&marker)?)?;

    Ok(p.dist)
}

fn download_to_file(url: &str, dest: &Path) -> Result<()> {
    let client = Client::new();
    let mut resp = client.get(url).send().with_context(|| format!("GET {url}"))?;
    resp.error_for_status_ref()?;

    let mut out = File::create(dest).with_context(|| format!("create {}", dest.display()))?;
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 { break; }
        out.write_all(&buf[..n])?;
    }
    Ok(())
}

fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
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
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest_dir: &Path) -> Result<()> {
    let f = File::open(archive)?;
    let gz = flate2::read::GzDecoder::new(f);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(dest_dir).with_context(|| format!("unpack to {}", dest_dir.display()))?;
    Ok(())
}
