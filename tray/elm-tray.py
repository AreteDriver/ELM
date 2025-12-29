#!/usr/bin/env python3
"""
ELM Tray - System tray icon for EVE Linux Manager
"""

import subprocess
import threading
import os
import signal
import sys

try:
    import pystray
    from PIL import Image
except ImportError:
    print("Installing dependencies...")
    subprocess.run([sys.executable, "-m", "pip", "install", "pystray", "pillow", "-q"])
    import pystray
    from PIL import Image

# Paths
HOME = os.environ.get("HOME", "")
ELM_BIN = os.path.join(HOME, ".local/bin/elm")
ICON_PATH = os.path.join(HOME, ".local/share/icons/eve-online.png")

# State
eve_running = False
eve_process = None


def load_icon():
    """Load the EVE icon or create a fallback."""
    try:
        if os.path.exists(ICON_PATH):
            return Image.open(ICON_PATH)
    except Exception:
        pass
    # Fallback: create a simple colored icon
    img = Image.new('RGB', (64, 64), color=(30, 30, 30))
    return img


def run_command(cmd, terminal=False):
    """Run an elm command."""
    if terminal:
        # Run in terminal for commands that need output
        subprocess.Popen(["x-terminal-emulator", "-e", f"{ELM_BIN} {cmd}; read -p 'Press Enter to close...'"])
    else:
        subprocess.Popen([ELM_BIN] + cmd.split())


def launch_eve(icon, item):
    """Launch EVE Online."""
    global eve_running, eve_process

    def run():
        global eve_running, eve_process
        eve_running = True
        update_menu(icon)
        eve_process = subprocess.Popen([ELM_BIN, "run"])
        eve_process.wait()
        eve_running = False
        eve_process = None
        update_menu(icon)

    threading.Thread(target=run, daemon=True).start()


def launch_profile(profile_name):
    """Return a function to launch a specific profile."""
    def launcher(icon, item):
        global eve_running, eve_process

        def run():
            global eve_running, eve_process
            eve_running = True
            update_menu(icon)
            eve_process = subprocess.Popen([ELM_BIN, "run", "--profile", profile_name])
            eve_process.wait()
            eve_running = False
            eve_process = None
            update_menu(icon)

        threading.Thread(target=run, daemon=True).start()

    return launcher


def show_status(icon, item):
    """Show ELM status in terminal."""
    run_command("status", terminal=True)


def show_doctor(icon, item):
    """Run system diagnostics."""
    run_command("doctor", terminal=True)


def check_updates(icon, item):
    """Check for engine updates."""
    run_command("update", terminal=True)


def show_logs(icon, item):
    """Show recent logs."""
    run_command("logs", terminal=True)


def open_config(icon, item):
    """Open config in editor."""
    subprocess.Popen([ELM_BIN, "config", "edit"])


def get_profiles():
    """Get list of available profiles."""
    profiles = []
    prefixes_dir = os.path.join(HOME, ".local/share/elm/prefixes")
    if os.path.exists(prefixes_dir):
        for name in os.listdir(prefixes_dir):
            if name.startswith("eve-") and os.path.isdir(os.path.join(prefixes_dir, name)):
                profile_name = name[4:]  # Remove "eve-" prefix
                profiles.append(profile_name)
    return sorted(profiles)


def create_menu():
    """Create the tray menu."""
    profiles = get_profiles()

    # Profile submenu
    if len(profiles) > 1:
        profile_items = [
            pystray.MenuItem(p, launch_profile(p)) for p in profiles
        ]
        profile_menu = pystray.Menu(*profile_items)
        launch_item = pystray.MenuItem("Launch", pystray.Menu(
            pystray.MenuItem("Default", launch_eve),
            pystray.Menu.SEPARATOR,
            *profile_items
        ))
    else:
        launch_item = pystray.MenuItem("Launch EVE", launch_eve)

    menu = pystray.Menu(
        launch_item,
        pystray.Menu.SEPARATOR,
        pystray.MenuItem("Status", show_status),
        pystray.MenuItem("Logs", show_logs),
        pystray.MenuItem("System Check", show_doctor),
        pystray.MenuItem("Check Updates", check_updates),
        pystray.Menu.SEPARATOR,
        pystray.MenuItem("Edit Config", open_config),
        pystray.Menu.SEPARATOR,
        pystray.MenuItem("Quit", quit_app)
    )
    return menu


def update_menu(icon):
    """Update the menu (refresh profiles, etc)."""
    icon.menu = create_menu()


def quit_app(icon, item):
    """Quit the tray app."""
    icon.stop()


def main():
    """Main entry point."""
    # Check if elm is installed
    if not os.path.exists(ELM_BIN):
        print(f"Error: elm not found at {ELM_BIN}")
        print("Install ELM first: ./install.sh")
        sys.exit(1)

    icon = pystray.Icon(
        "elm-tray",
        load_icon(),
        "EVE Linux Manager",
        create_menu()
    )

    # Handle SIGTERM/SIGINT
    def signal_handler(sig, frame):
        icon.stop()
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    print("ELM Tray running. Right-click the tray icon for options.")
    icon.run()


if __name__ == "__main__":
    main()
