#!/bin/bash

echo "=== Starting Mewtion ==="

# 1. Check for ADB and set up port forwarding if a phone is connected
if command -v adb &>/dev/null; then
  echo "Checking for connected Android device..."
  # Check if any device is listed and authorized
  if adb devices | grep -qw "device"; then
    echo "Android device detected! Setting up ADB port forward (tcp:8765)..."
    adb forward tcp:8765 tcp:8765
  else
    echo "No Android device detected. Skipping port forwarding (will use laptop accelerometer if available)."
  fi
else
  echo "ADB is not installed. Skipping port forwarding."
fi

# 2. Launch the Mewtion overlay in release mode
echo "Launching Mewtion overlay..."
cargo run --release --bin Mewtion
