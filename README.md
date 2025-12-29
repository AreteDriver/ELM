# ELM - EVE Linux Manager

A Rust-based CLI tool for running EVE Online on Linux using Proton/Wine.

## Features

- **One-Click Launch**: `elm run` handles everything automatically
- **Multi-Account Support**: Manage multiple profiles for multiboxing
- **Engine Management**: Download and manage GE-Proton versions with SHA256 verification
- **Snapshot/Rollback**: Save and restore prefix states using tar.zst compression
- **Auto-Update**: Check for and install new GE-Proton releases from GitHub
- **Settings Presets**: Switch between performance, quality, and balanced modes
- **System Diagnostics**: Verify your system meets EVE's requirements

## Installation

### From Source

```bash
git clone https://github.com/AreteDriver/ELM.git
cd ELM/elm
./install.sh
```

### Arch Linux (AUR)

```bash
yay -S elm-eve-git
```

### Manual Build

```bash
cargo build --release
cp target/release/elm ~/.local/bin/
```

### Dependencies

- Rust 1.70+ (build only)
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

### Core Commands

| Command | Description |
|---------|-------------|
| `elm run` | Launch EVE Online (auto-setup on first run) |
| `elm status` | Show installed engines, prefixes, snapshots |
| `elm doctor` | System compatibility diagnostics |
| `elm update` | Check for GE-Proton updates |
| `elm clean` | Remove old engines and download cache |
| `elm logs` | View Wine/Proton and EVE logs |

### `elm run [--profile NAME]`

Launch EVE Online. On first run, this will:
1. Download and install GE-Proton
2. Create a Wine prefix
3. Download and install the EVE Launcher
4. Launch the game

```bash
elm run                    # Launch default profile
elm run --profile alt      # Launch alternate profile
```

### `elm update [--install]`

Check for GE-Proton updates.

```bash
elm update                 # Check for updates
elm update --install       # Download and install latest
```

### `elm clean`

Clean up disk space:

```bash
elm clean --downloads      # Remove downloaded archives
elm clean --engines        # Remove old engine versions (keep latest)
elm clean --all            # Remove both
elm clean --all --dry-run  # Preview what would be removed
```

### `elm logs`

View logs for debugging:

```bash
elm logs                   # Show last 50 lines of launcher log
elm logs --list            # List all available log files
elm logs -n 100            # Show last 100 lines
elm logs --log-type squirrel  # View installer logs
elm logs --log-type proton    # View proton logs
```

### Profile Management

Manage multiple EVE accounts with separate prefixes:

```bash
elm profile list                    # List all profiles
elm profile create <name>           # Create new profile
elm profile clone <source> <target> # Clone existing profile
elm profile info <name>             # Show profile details
elm profile delete <name>           # Delete profile
```

**Multiboxing example:**
```bash
elm profile create alt1
elm profile create alt2
elm run --profile default &
elm run --profile alt1 &
elm run --profile alt2 &
```

### Configuration

Manage settings and presets:

```bash
elm config init            # Create default config files
elm config show            # Display current settings
elm config edit            # Open config in $EDITOR
elm config preset <name>   # Apply a settings preset
```

**Available presets:**
- `performance` - Maximum FPS, FSR upscaling, shader cache
- `quality` - Native resolution, no async shaders
- `balanced` - Good performance with FPS counter
- `debug` - Verbose logging for troubleshooting

### Snapshot/Rollback

Backup and restore your prefix:

```bash
elm snapshot \
  --prefix ~/.local/share/elm/prefixes/eve-default \
  --snapshots ~/.local/share/elm/snapshots \
  --name before-patch

elm rollback \
  --snapshot ~/.local/share/elm/snapshots/before-patch.tar.zst \
  --prefix ~/.local/share/elm/prefixes/eve-default
```

## Configuration Files

Configs are stored in `~/.config/elm/`:

- `manifests/eve-online.json` - EVE manifest with engine reference and environment variables

Data is stored in `~/.local/share/elm/`:

- `engines/` - Downloaded Proton versions
- `prefixes/` - Wine prefixes
- `snapshots/` - Prefix backups
- `downloads/` - Downloaded archives

## Environment Variables

Default environment variables for optimal EVE performance:

| Variable | Value | Purpose |
|----------|-------|---------|
| `DXVK_ASYNC` | `1` | Async shader compilation |
| `PROTON_NO_ESYNC` | `0` | Enable esync |
| `PROTON_NO_FSYNC` | `0` | Enable fsync |
| `PROTON_ENABLE_NVAPI` | `1` | NVIDIA API support |
| `VKD3D_FEATURE_LEVEL` | `12_1` | DirectX 12 feature level |
| `WINE_FULLSCREEN_FSR` | `1` | AMD FidelityFX upscaling |

Use `elm config preset <name>` to switch between optimized configurations.

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

1. Check logs: `elm logs`
2. Try debug preset: `elm config preset debug`
3. Roll back to known-good state:
```bash
elm rollback --snapshot ~/.local/share/elm/snapshots/eve-fresh-install.tar.zst \
             --prefix ~/.local/share/elm/prefixes/eve-default
```

### View detailed diagnostics

```bash
elm doctor
elm logs --log-type proton
elm logs --log-type squirrel
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.
