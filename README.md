# Mewtion

An open-source Linux desktop implementation inspired by Apple's **Vehicle Motion Cues**.

Mewtion helps reduce motion sickness when working on your laptop in a moving vehicle. It creates a transparent, click-through overlay of dots along the edges of your screen. Using real-time accelerometer data, the dots drift in the opposite direction of the vehicle's movement, helping resolve the sensory conflict between your stationary screen and the physical motion your inner ear feels.

## Architecture

The system consists of three parts working together for ultra-low latency:

1. **Sensor (Laptop / Android)**

   Mewtion first attempts to detect and use the laptop's built-in accelerometer, read directly from the kernel's **IIO** interface (`/sys/bus/iio/devices/`). This covers both dedicated accelerometer drivers and laptops whose sensors arrive through the HID sensor hub, and needs no phone, cable or extra daemon. If a compatible accelerometer is not available, it falls back to an Android companion app that reads gravity-filtered linear acceleration using `TYPE_LINEAR_ACCELERATION`.

   The overlay is driven by linear acceleration. Where the hardware publishes its own `gravity` channel, Mewtion subtracts it; otherwise gravity is tracked with a low-pass filter and subtracted, mirroring how Android derives `TYPE_LINEAR_ACCELERATION`.

2. **Tunnel (Network / ADB)**

   When using the Android fallback, the connection is network-agnostic. It can be established wirelessly over a local Wi-Fi network, a mobile hotspot, or through a physical USB cable using ADB port forwarding. Sensor data is streamed over a lightweight TCP socket.

3. **Overlay & Control Panel (Linux)**

   A Rust/GTK4 application renders an always-on-top, click-through canvas natively on Wayland. It uses **Layer Shell** (`gtk4-layer-shell`) to bind to the compositor and features a custom 60 FPS particle physics engine. A companion `iced`-based GUI Control Panel runs alongside it for live configuration.

> **Note on Compositors**
>
> The overlay uses the Wayland Layer Shell protocol, making it natively compatible with modern Wayland compositors like KDE Plasma, Sway, and Hyprland.

## Installation & Quick Start

You can either install pre-built binaries (recommended, no Rust/Cargo required) or compile from source.

### 1. Pre-built Packages (1-Click Run)

Pre-compiled packages for Linux `x86_64` are available on the **[Releases](https://github.com/aayuxh-vim/Mewtion/releases)** page.

#### Option A: Universal AppImage (Recommended)
Works out of the box on almost any Linux distribution (Ubuntu, Fedora, Arch, openSUSE, Debian) without installing anything:
```bash
# 1. Make the AppImage executable
chmod +x Mewtion-*-x86_64.AppImage

# 2. Run it!
./Mewtion-*-x86_64.AppImage
```

#### Option B: Debian / Ubuntu Package (`.deb`)
Installs Mewtion as a system application and adds it directly to your system application drawer / start menu:
```bash
sudo apt install ./mewtion_*_amd64.deb
```
Launch it anytime from your application menu or by running `mewtion` in your terminal.

#### Option C: Portable Tarball (`.tar.gz`)
Standalone archive with bundled libraries and a portable launcher script:
```bash
tar -xvf mewtion-*-linux-x86_64.tar.gz
cd mewtion-*-linux-x86_64
./start.sh
```

#### Option D: Nix Flake
If you use Nix or NixOS, you can run Mewtion directly without installation:
```bash
nix run github:aayuxh-vim/Mewtion
```

---

### 2. Building from Source (Self-Build / Developers)

#### System Prerequisites

* Linux desktop environment with a **Wayland compositor supporting Layer Shell** (e.g. Sway, Hyprland, KDE Plasma 6)
* **Rust & Cargo** (installed via [rustup.rs](https://rustup.rs))
* **GTK4** and **GTK4 Layer Shell** development libraries:
  * **Arch Linux / Manjaro:**
    ```bash
    sudo pacman -S gtk4 gtk4-layer-shell base-devel
    ```
  * **Ubuntu / Debian:**
    ```bash
    sudo apt install pkg-config meson ninja-build valac libgtk-4-dev libwayland-dev wayland-protocols libxkbcommon-dev librsvg2-dev
    # Build gtk4-layer-shell from source (if not in distro repositories):
    git clone https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell
    cd /tmp/gtk4-layer-shell && meson setup build --prefix=/usr && ninja -C build && sudo ninja -C build install
    ```
* **Sensor Input:**
  * Built-in laptop accelerometer: detected automatically via `/sys/bus/iio/devices/`.
  * *Or* Android phone fallback: requires **ADB (`android-tools`)** and the **[Mewtion-Android](https://github.com/aayuxh-vim/Mewtion-Android)** companion app with USB Debugging enabled.

#### Build & Run

**Automated (recommended for dev):**
```bash
chmod +x run.sh
./run.sh
```
This script checks for ADB devices, forwards the network port if plugged in, compiles release binaries, and launches both the control panel and overlay.

**Manual Cargo Commands:**
```bash
cargo build --release
cargo run --release --bin control_panel &
cargo run --release --bin Mewtion
```

## Performance

Mewtion is designed around low-latency motion feedback and smooth rendering.

Key goals include:

- 60 FPS particle rendering
- Adaptive software deadband (noise gate) to eliminate anchor drift on sensitive hardware
- Low-latency sensor processing (Sensor Fusion combining Accelerometer + Gyroscope)
- Native click-through overlay via Wayland Layer Shell
- Minimal CPU and memory usage
- Automatic fallback to an Android device when required

## Future Enhancements

- [x] **Wayland Native Support:** Add native Wayland support using Layer Shell protocols.
- [x] **Laptop Accelerometer Support:** Detect and use the laptop's built-in accelerometer when available, eliminating the need for a phone and USB connection.
- [x] **UI:** Add a graphical settings menu to customize dot size, opacity, margins, acceleration sensitivity, and animation behavior.
- [ ] **Sensor Calibration:** Add automatic and manual calibration to account for device orientation and sensor bias.
- [x] **Sensor Fusion:** Combine accelerometer and gyroscope data for more accurate motion detection and smoother movement.
- [ ] **BLE Support:** Implement Bluetooth Low Energy as an alternative to the USB connection.
- [ ] **iOS Support:** Create an iOS companion app to broadcast sensor data. *(Note: I do not own a Mac to develop the iOS companion app. If you are an iOS developer, contributions using* *`CoreMotion`* *and* *`NWConnection`* *are highly welcome! You can reference the* [***Mewtion-Android***](https://github.com/aayuxh-vim/Mewtion-Android) *repository for the expected stream format).*
- [ ] **Windows Support:** Port the window management logic to the Windows API.
- [ ] **SteamOS In-Game Overlay Support:** Add support for displaying Mewtion's motion cues over games running on SteamOS, with compatibility for gamescope, fullscreen, and borderless modes while maintaining click-through behavior and minimal performance overhead.

## Troubleshooting

### Mewtion does not detect the laptop accelerometer

Mewtion looks for a device under `/sys/bus/iio/devices/` exposing `in_accel_{x,y,z}_raw`. List what your machine provides with:

```bash
grep . /sys/bus/iio/devices/iio:device*/name
```

On startup Mewtion prints which source it chose, so you can confirm at a glance which sensors it found.

### Overlay dots drift when the phone is still

Ensure you are running the latest version of the Linux overlay. A software deadband has been implemented to filter out hardware-specific micro-vibrations and noise.

### Cannot connect via Wi-Fi/Hotspot

Ensure your phone's screen is on and the Mewtion Android app is actively running. If using a local network, ensure your router does not block local peer-to-peer device communication (AP Isolation).

### Motion feels choppy or lags behind the vehicle

Mewtion prints the accelerometer's rate at startup. A HID sensor hub answers each `*_raw` read by fetching a fresh report, costing about one sampling period per axis. An accelerometer parked at 10 Hz therefore yields roughly 5 Hz of motion updates, while raising it to 100 Hz gives about 50 Hz. Dedicated I2C accelerometers return a cached value immediately and are not affected.

Check the current rate:

```bash
cat /sys/bus/iio/devices/iio:device*/in_accel_sampling_frequency
```

Mewtion asks for a faster rate at startup, but the attribute is root-owned, so the request is skipped when running as a normal user. The simplest fix is to let udev set it, since udev rules run as root. Create `/etc/udev/rules.d/99-iio-sampling.rules`:

```
SUBSYSTEM=="iio", KERNEL=="iio:device*", ATTR{in_accel_sampling_frequency}="100"
```

Apply it with `sudo udevadm control --reload && sudo udevadm trigger --subsystem-match=iio`. Re-triggering can renumber `iio:deviceN`, which is why Mewtion identifies sensors by the channels they expose rather than by device number.

### ADB cannot detect the phone

Check that:

- USB debugging is enabled.
- The phone is connected using a USB data cable.
- The device is authorized on the phone.
- ADB is installed and available in your terminal.

Check the connection with `adb devices`, then create the port forward manually if the script fails:

```bash
adb forward tcp:8765 tcp:8765
```

### The overlay does not appear

Make sure you are running under a Wayland session and your compositor supports the Layer Shell protocol.

## Steam Deck Testing

Steam Deck users and testers are welcome to experiment with Mewtion on their devices and help evaluate potential **SteamOS in-game overlay support**.

If you have a Steam Deck, feel free to test Mewtion under SteamOS and report your experience, including compositor behavior, fullscreen and gamescope compatibility, performance, and any issues encountered.

Feedback and contributions from Steam Deck users are especially welcome as we explore native in-game overlay support.


## Contributing

Contributions are welcome.

You can contribute by:

- Adding support for new sensor sources
- Improving sensor processing
- Optimizing the particle physics engine
- Developing the Android or future iOS companion apps
- Adding configuration options
- Fixing bugs
- Improving documentation

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
