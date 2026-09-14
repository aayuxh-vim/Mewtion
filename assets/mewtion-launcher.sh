#!/usr/bin/env bash
set -e

export LD_LIBRARY_PATH="/usr/lib/mewtion:${LD_LIBRARY_PATH:-}"

if command -v adb &>/dev/null && adb devices | grep -qw "device"; then
  adb forward tcp:8765 tcp:8765 2>/dev/null || true
fi

/usr/lib/mewtion/control_panel &
CP_PID=$!
/usr/lib/mewtion/Mewtion &
MW_PID=$!

trap "kill $CP_PID $MW_PID 2>/dev/null" EXIT INT TERM
wait
