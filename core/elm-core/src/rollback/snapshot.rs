use anyhow::{Context, Result};
use std::fs::{self, File};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

pub fn snapshot_prefix(prefix_dir: &Path, snapshots_dir: &Path, snapshot_name: &str) -> Result<PathBuf> {
    fs::create_dir_all(snapshots_dir).with_context(|| format!("create {}", snapshots_dir.display()))?;
    let out_path = snapshots_dir.join(format!("{snapshot_name}.tar.zst"));

    let out = File::create(&out_path).with_context(|| format!("create {}", out_path.display()))?;
    let encoder = zstd::Encoder::new(out, 3).context("zstd encoder")?;
    let mut tar_builder = tar::Builder::new(encoder);
    tar_builder.follow_symlinks(false);

    // Walk directory manually to handle symlinks properly
    let mut visited_inodes: HashSet<u64> = HashSet::new();
    append_dir_recursive(&mut tar_builder, prefix_dir, Path::new("prefix"), &mut visited_inodes)?;

    let encoder = tar_builder.into_inner().context("finish tar")?;
    encoder.finish().context("finish zstd")?;

    Ok(out_path)
}

fn append_dir_recursive<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    src_path: &Path,
    tar_path: &Path,
    visited: &mut HashSet<u64>,
) -> Result<()> {
    let metadata = fs::symlink_metadata(src_path)?;

    // Track inodes to avoid infinite loops
    let inode = metadata.ino();
    if metadata.is_dir() && !visited.insert(inode) {
        return Ok(()); // Already visited this directory
    }

    if metadata.is_symlink() {
        // Store symlink as-is
        let target = fs::read_link(src_path)?;
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        builder.append_link(&mut header, tar_path, &target).ok();
    } else if metadata.is_dir() {
        builder.append_dir(tar_path, src_path).ok();

        if let Ok(entries) = fs::read_dir(src_path) {
            for entry in entries.flatten() {
                let child_src = entry.path();
                let child_tar = tar_path.join(entry.file_name());
                append_dir_recursive(builder, &child_src, &child_tar, visited)?;
            }
        }
    } else if metadata.is_file() {
        if let Ok(mut file) = File::open(src_path) {
            let mut header = tar::Header::new_gnu();
            header.set_metadata(&metadata);
            header.set_size(metadata.len());
            header.set_cksum();
            builder.append_data(&mut header, tar_path, &mut file).ok();
        }
    }

    Ok(())
}
