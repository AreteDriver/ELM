# ELM - EVE Linux Manager

A Rust-based CLI tool for running EVE Online on Linux using Proton/Wine.

## Features

- **Engine Management**: Download and manage GE-Proton versions with SHA256 verification
- **Prefix Management**: Create and manage Wine prefixes for EVE
- **Snapshot/Rollback**: Save and restore prefix states using tar.zst compression
- **Auto-Update**: Check for and install new GE-Proton releases from GitHub
- **System Diagnostics**: Verify your system meets EVE's requirements

## Installation

### From Source

```bash
git clone https://github.com/AreteDriver/ELM.git
cd ELM/elm
cargo build --release
cp target/release/elm ~/.local/bin/
```

### Dependencies

- Rust 1.70+
- Python 3 (for Proton)
- Steam (for Proton compatibility layer)
- Vulkan drivers

## Quick Start

```bash
# Run EVE (auto-installs engine and creates prefix on first run)
elm run

# Check system compatibility
elm doctor

# View installed components
elm status
```

## Commands

### `elm run`

Launch EVE Online. On first run, this will:
1. Download and install GE-Proton
2. Create a Wine prefix
3. Download and install the EVE Launcher
4. Launch the game

### `elm status`

Show installed engines, prefixes, and snapshots.

### `elm doctor`

Run system diagnostics to verify:
- Vulkan support and version
- GPU vendor and driver
- Steam installation
- Required libraries
- Disk space

### `elm update [--install]`

Check for GE-Proton updates. Use `--install` to automatically download and install the latest version.

### `elm engine install`

```bash
elm engine install \
  --schemas ~/.local/share/elm/schemas \
  --engine path/to/engine.json \
  --engines-dir ~/.local/share/elm/engines \
  --downloads-dir ~/.local/share/elm/downloads
```

### `elm prefix init`

```bash
elm prefix init \
  --proton-root ~/.local/share/elm/engines/ge-proton10-27/dist/GE-Proton10-27 \
  --prefix ~/.local/share/elm/prefixes/eve-default
```

### `elm snapshot`

Create a backup of your prefix:

```bash
elm snapshot \
  --prefix ~/.local/share/elm/prefixes/eve-default \
  --snapshots ~/.local/share/elm/snapshots \
  --name my-backup
```

### `elm rollback`

Restore a prefix from a snapshot:

```bash
elm rollback \
  --snapshot ~/.local/share/elm/snapshots/my-backup.tar.zst \
  --prefix ~/.local/share/elm/prefixes/eve-default
```

### `elm validate`

Validate JSON config files against schemas:

```bash
elm validate \
  --schemas ~/.local/share/elm/schemas \
  --manifest path/to/manifest.json
```

### `elm install eve`

Download and install the EVE Launcher into a prefix:

```bash
elm install eve \
  --prefix ~/.local/share/elm/prefixes/eve-default \
  --proton-root ~/.local/share/elm/engines/ge-proton10-27/dist/GE-Proton10-27
```

## Configuration

Configs are stored in `~/.config/elm/`:

- `manifests/eve-online.json` - EVE manifest with engine reference and environment variables

Data is stored in `~/.local/share/elm/`:

- `engines/` - Downloaded Proton versions
- `prefixes/` - Wine prefixes
- `snapshots/` - Prefix backups
- `downloads/` - Downloaded archives
- `schemas/` - JSON schemas

## Environment Variables

The following environment variables are set automatically for optimal EVE performance:

| Variable | Value | Purpose |
|----------|-------|---------|
| `DXVK_ASYNC` | `1` | Async shader compilation |
| `PROTON_NO_ESYNC` | `0` | Enable esync |
| `PROTON_NO_FSYNC` | `0` | Enable fsync |
| `PROTON_ENABLE_NVAPI` | `1` | NVIDIA API support |
| `VKD3D_FEATURE_LEVEL` | `12_1` | DirectX 12 feature level |

## Troubleshooting

### "No Vulkan support detected"

Install Vulkan drivers for your GPU:

```bash
# NVIDIA
sudo apt install nvidia-driver-xxx

# AMD
sudo apt install mesa-vulkan-drivers

# Intel
sudo apt install mesa-vulkan-drivers
```

### "Steam not found"

Install Steam - it provides the Proton compatibility layer:

```bash
sudo apt install steam
```

### EVE crashes on launch

Try rolling back to a known-good prefix state:

```bash
elm rollback --snapshot ~/.local/share/elm/snapshots/eve-fresh-install.tar.zst \
             --prefix ~/.local/share/elm/prefixes/eve-default
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.
