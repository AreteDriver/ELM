#!/bin/bash
# ELM Installer - EVE Linux Manager

set -e

echo "Installing ELM (EVE Linux Manager)..."

# Build
echo "Building..."
cargo build --release

# Install binary
echo "Installing binary to ~/.local/bin/"
mkdir -p ~/.local/bin
cp target/release/elm ~/.local/bin/

# Create data directories
echo "Creating data directories..."
mkdir -p ~/.local/share/elm/{engines,prefixes,snapshots,downloads,schemas}
mkdir -p ~/.config/elm/manifests

# Copy schemas
echo "Installing schemas..."
cp core/elm-core/schemas/*.json ~/.local/share/elm/schemas/

# Download EVE icon
ICON_PATH=~/.local/share/icons/eve-online.png
if [ ! -f "$ICON_PATH" ]; then
    echo "Downloading EVE icon..."
    mkdir -p ~/.local/share/icons
    curl -sL "https://images.evetech.net/corporations/1000125/logo?size=256" -o "$ICON_PATH" || true
fi

# Create desktop entry
echo "Creating desktop entry..."
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/eve-online-elm.desktop << 'DESKTOP'
[Desktop Entry]
Name=EVE Online (ELM)
Comment=Launch EVE Online via ELM
Exec=elm run
Icon=eve-online
Terminal=false
Type=Application
Categories=Game;
Keywords=EVE;Online;Space;MMO;
DESKTOP

# Update desktop database
update-desktop-database ~/.local/share/applications 2>/dev/null || true

echo ""
echo "Installation complete!"
echo ""
echo "Usage:"
echo "  elm run      - Launch EVE Online"
echo "  elm status   - Show installed components"
echo "  elm doctor   - Check system compatibility"
echo "  elm update   - Check for Proton updates"
echo ""
echo "First run will download GE-Proton and the EVE Launcher (~700MB)"
