#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

# Prioritize local bundled libraries (e.g. libgtk4-layer-shell.so)
if [ -d "$DIR/lib" ]; then
  export LD_LIBRARY_PATH="$DIR/lib:${LD_LIBRARY_PATH:-}"
fi

echo "=== Starting Mewtion Ecosystem ==="

# 1. Check for USB / ADB connection
if command -v adb &>/dev/null; then
  if adb devices | grep -qw "device"; then
    echo "USB: Android device detected! Forwarding tcp:8765..."
    adb forward tcp:8765 tcp:8765
  else
    echo "No USB device detected. Defaulting to Wi-Fi mode."
  fi
else
  echo "ADB not installed. Defaulting to Wi-Fi mode."
fi

# 2. Launch Control Panel & Mewtion Overlay
echo "Launching Control Panel..."
"$DIR/bin/control_panel" &
CONTROL_PANEL_PID=$!

echo "Launching Mewtion overlay..."
"$DIR/bin/Mewtion" &
MEWTION_PID=$!

# Terminate background processes when script exits
trap "kill $CONTROL_PANEL_PID $MEWTION_PID 2>/dev/null" EXIT INT TERM
wait
