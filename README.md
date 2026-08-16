# Mewtion

An open-source Linux desktop implementation inspired by Apple's **Vehicle Motion Cues**.

Mewtion helps reduce motion sickness when working on your laptop in a moving vehicle. It creates a transparent, click-through overlay of dots along the edges of your screen. Using real-time accelerometer data, the dots drift in the opposite direction of the vehicle's movement, helping resolve the sensory conflict between your stationary screen and the physical motion your inner ear feels.

## Architecture

The system consists of three parts working together for ultra-low latency:

1. **Sensor (Laptop / Android)**

   Mewtion first attempts to detect and use the laptop's built-in accelerometer. If a compatible accelerometer is not available, it falls back to an Android companion app that reads gravity-filtered linear acceleration using `TYPE_LINEAR_ACCELERATION`.

2. **Tunnel (ADB)**

   When using the Android fallback, the connection is established through USB using ADB port forwarding. Sensor data is streamed over a TCP socket.

3. **Overlay (Linux)**

   A Rust/GTK4 application renders an always-on-top, click-through canvas natively on Wayland. It uses **Layer Shell** (`gtk4-layer-shell`) to bind to the compositor and features a custom 60 FPS particle physics engine with detaching anchors and organic flow patterns.

> **Note on Compositors**
>
> The overlay uses the Wayland Layer Shell protocol, making it natively compatible with modern Wayland compositors like KDE Plasma, Sway, and Hyprland.

## Prerequisites

* Linux desktop environment with a **Wayland compositor supporting Layer Shell**
* **Rust / Cargo**
* **ADB (Android Debug Bridge)** for the Android fallback

  * On Arch Linux:

    ```bash
    sudo pacman -S android-tools
    ```
* An Android device running the **[Mewtion-Android](https://github.com/aayuxh-vim/Mewtion-Android)** companion app with **USB Debugging** enabled
* A laptop with a supported accelerometer for laptop-based sensor input

## Usage & Automation

### Quick Start (Using the Shell Script)

We provide an automated `run.sh` script that checks for your connected Android device, sets up ADB port forwarding automatically, and launches the Mewtion overlay in release mode.

1. Make sure the script is executable (first time only):

   ```bash
   chmod +x run.sh
   ```

2. Run everything in one go:

   ```bash
   ./run.sh
   ```

### Manual Setup & Running

#### 1. Build Mewtion

Clone the repository and build the Linux overlay:

```bash
cargo build --release
```

#### 2. Android Fallback Setup

If your laptop does not have a compatible accelerometer:

1. Download and install the companion app from the **[Mewtion-Android Repository](https://github.com/aayuxh-vim/Mewtion-Android)**.

2. Connect your phone via USB with USB Debugging enabled.

3. Forward the sensor TCP port:

   ```bash
   adb forward tcp:8765 tcp:8765
   ```

4. Open the Mewtion app on your phone. It features a background foreground service so it can stay active even when minimized.

#### 3. Run Mewtion

Start the overlay natively on Wayland:

```bash
cargo run --release --bin Mewtion
```

## Performance

Mewtion is designed around low-latency motion feedback and smooth rendering.

Key goals include:

* 60 FPS particle rendering
* Low-latency sensor processing
* Native click-through overlay via Wayland Layer Shell
* Minimal CPU and memory usage
* Real-time acceleration response
* Automatic sensor selection
* Automatic fallback to an Android device when required

## Future Enhancements

* [x] **Wayland Native Support:** Add native Wayland support using Layer Shell protocols.
* [ ] **Laptop Accelerometer Support:** Detect and use the laptop's built-in accelerometer when available, eliminating the need for a phone and USB connection.
* [x] **UI:** Add a graphical settings menu to customize dot size, opacity, margins, acceleration sensitivity, and animation behavior.
* [ ] **Sensor Calibration:** Add automatic and manual calibration to account for device orientation and sensor bias.
* [ ] **Sensor Fusion:** Combine accelerometer and gyroscope data for more accurate motion detection and smoother movement.
* [ ] **BLE Support:** Implement Bluetooth Low Energy as an alternative to the USB connection.
* [ ] **iOS Support:** Create an iOS companion app to broadcast sensor data.
* [ ] **Windows Support:** Port the window management logic to the Windows API.
* [ ] **Multi-Monitor Support:** Support motion cues across multiple displays.
* [ ] **Adaptive Motion Sensitivity:** Automatically adjust dot movement based on the intensity of detected motion.

## Troubleshooting

### Mewtion does not detect the laptop accelerometer

Make sure your laptop exposes its accelerometer through a Linux-supported sensor interface.

If no compatible accelerometer is detected, connect an Android device and use the phone fallback.

### ADB cannot detect the phone

Check that:

* USB debugging is enabled.
* The phone is connected using a USB data cable.
* The device is authorized on the phone.
* ADB is installed and available in your terminal.

Check the connection with:

```bash
adb devices
```

Then create the port forward:

```bash
adb forward tcp:8765 tcp:8765
```

### The overlay does not appear

Make sure you are running under a Wayland session and your compositor supports the Layer Shell protocol.

## Contributing

Contributions are welcome.

You can contribute by:

* Adding support for new sensor sources
* Improving sensor processing
* Optimizing the particle physics engine
* Developing the Android or future iOS companion apps
* Adding configuration options
* Fixing bugs
* Improving documentation

## License

This project is licensed under the MIT License.
