#!/bin/bash

echo "=== Starting Mewtion Ecosystem ==="

# 1. Check for ADB and set up port forwarding if a phone is connected
if command -v adb &>/dev/null; then
  echo "Checking for connected Android device..."
  if adb devices | grep -qw "device"; then
    echo "Android device detected! Setting up ADB port forward (tcp:8765)..."
    adb forward tcp:8765 tcp:8765
  else
    echo "No Android device detected. Skipping port forwarding."
  fi
else
  echo "ADB is not installed. Skipping port forwarding."
fi

# 2. Build the project first to ensure both are ready
echo "Compiling binaries..."
cargo build --release

# 3. Launch the Control Panel in the background
echo "Launching Control Panel..."
cargo run --release --bin control_panel &

# 4. Launch the Wayland Overlay in the foreground
echo "Launching Mewtion overlay..."
cargo run --release --bin Mewtion

# 5. Cleanup: If you close the overlay terminal, kill the control panel too
trap "kill 0" EXIT
