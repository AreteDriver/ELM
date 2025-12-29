#!/bin/bash
# Install ELM Tray

set -e

echo "Installing ELM Tray..."

# Create virtual environment
VENV_DIR="$HOME/.local/share/elm/tray-venv"
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
fi

# Install Python dependencies
echo "Installing dependencies..."
"$VENV_DIR/bin/pip" install pystray pillow -q

# Copy tray script
mkdir -p ~/.local/bin
cp "$(dirname "$0")/elm-tray.py" ~/.local/share/elm/elm-tray.py
chmod +x ~/.local/share/elm/elm-tray.py

# Create wrapper script
cat > ~/.local/bin/elm-tray << EOF
#!/bin/bash
exec "$VENV_DIR/bin/python" "$HOME/.local/share/elm/elm-tray.py" "\$@"
EOF
chmod +x ~/.local/bin/elm-tray

# Create autostart entry
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/elm-tray.desktop << EOF
[Desktop Entry]
Type=Application
Name=ELM Tray
Comment=EVE Linux Manager system tray
Exec=$HOME/.local/bin/elm-tray
Icon=$HOME/.local/share/icons/eve-online.png
Terminal=false
Categories=Game;
X-GNOME-Autostart-enabled=true
EOF

echo ""
echo "ELM Tray installed!"
echo ""
echo "Start now:     elm-tray &"
echo "Runs on login: automatically via autostart"
echo ""
echo "Right-click the tray icon for options."
