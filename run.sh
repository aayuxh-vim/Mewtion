#!/bin/bash

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

# 2. Build and Launch
echo "Compiling binaries..."
cargo build --release

echo "Launching Control Panel..."
cargo run --release --bin control_panel &

echo "Launching Mewtion overlay..."
cargo run --release --bin Mewtion

trap "kill 0" EXIT
