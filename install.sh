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

# Install icon to hicolor theme for better compatibility
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
cp "$ICON_PATH" ~/.local/share/icons/hicolor/256x256/apps/ 2>/dev/null || true

# Create desktop entry with dock actions
echo "Creating desktop entry..."
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/eve-online-elm.desktop << DESKTOP
[Desktop Entry]
Name=EVE Online
Comment=Launch EVE Online via ELM
Exec=$HOME/.local/bin/elm run
Icon=$HOME/.local/share/icons/eve-online.png
Terminal=false
Type=Application
Categories=Game;
Keywords=EVE;Online;Space;MMO;CCP;
StartupNotify=true
StartupWMClass=eve.exe
Actions=status;doctor;update;

[Desktop Action status]
Name=Show Status
Exec=x-terminal-emulator -e $HOME/.local/bin/elm status

[Desktop Action doctor]
Name=System Check
Exec=x-terminal-emulator -e $HOME/.local/bin/elm doctor

[Desktop Action update]
Name=Check for Updates
Exec=x-terminal-emulator -e $HOME/.local/bin/elm update
DESKTOP

# Update desktop and icon caches
update-desktop-database ~/.local/share/applications 2>/dev/null || true
gtk-update-icon-cache -f ~/.local/share/icons/hicolor 2>/dev/null || true

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
