use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn restore_prefix(snapshot_tar_zst: &Path, prefix_dir: &Path) -> Result<()> {
    if prefix_dir.exists() {
        fs::remove_dir_all(prefix_dir).with_context(|| format!("remove {}", prefix_dir.display()))?;
    }
    fs::create_dir_all(prefix_dir).with_context(|| format!("create {}", prefix_dir.display()))?;

    let f = fs::File::open(snapshot_tar_zst).with_context(|| format!("open {}", snapshot_tar_zst.display()))?;
    let decoder = zstd::Decoder::new(f).context("zstd decoder")?;
    let mut archive = tar::Archive::new(decoder);

    // Unpack to parent so it recreates a "prefix" directory
    let parent = prefix_dir.parent().unwrap_or(prefix_dir);
    archive.unpack(parent).context("untar snapshot")?;

    // Move extracted "prefix" into place
    let extracted = parent.join("prefix");
    if extracted.exists() && extracted != prefix_dir {
        if prefix_dir.exists() {
            fs::remove_dir_all(prefix_dir)?;
        }
        fs::rename(extracted, prefix_dir).context("rename extracted prefix")?;
    }
    Ok(())
}
