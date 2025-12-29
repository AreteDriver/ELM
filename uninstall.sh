#!/bin/bash
# ELM Uninstaller - EVE Linux Manager

echo "Uninstalling ELM (EVE Linux Manager)..."
echo ""

# Remove binary
if [ -f ~/.local/bin/elm ]; then
    rm ~/.local/bin/elm
    echo "✓ Removed binary"
fi

# Remove desktop entry
if [ -f ~/.local/share/applications/eve-online-elm.desktop ]; then
    rm ~/.local/share/applications/eve-online-elm.desktop
    update-desktop-database ~/.local/share/applications 2>/dev/null || true
    echo "✓ Removed desktop entry"
fi

# Remove icon
if [ -f ~/.local/share/icons/eve-online.png ]; then
    rm ~/.local/share/icons/eve-online.png
    echo "✓ Removed icon"
fi

echo ""
echo "ELM binary and desktop entry removed."
echo ""
echo "User data preserved at:"
echo "  ~/.local/share/elm/  (engines, prefixes, snapshots)"
echo "  ~/.config/elm/       (configuration)"
echo ""
echo "To completely remove all data, run:"
echo "  rm -rf ~/.local/share/elm ~/.config/elm"
