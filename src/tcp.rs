// tcp.rs -- phone-over-USB sensor bridge.
//
// Replaces the earlier BLE approach after a night of fighting a flaky
// BlueZ/adapter combo (including a confirmed BlueZ segfault). This is
// much simpler: the phone runs a tiny TCP server (see the Android app's
// MainActivity.kt), and `adb forward tcp:8765 tcp:8765` tunnels that
// port to localhost on the laptop over the same USB cable already used
// for adb. No pairing, no advertising, no GATT -- just a socket.
//
// Wire format: continuous stream of 12-byte records, each three
// little-endian f32 values (x, y, z). Fixed size means no framing is
// needed -- just read 12 bytes at a time.

use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct AccelSample {
    pub x: f32,
    pub y: f32,
    #[allow(dead_code)]
    pub z: f32,
}

const ADDR: &str = "127.0.0.1:8765";

/// Runs forever on a background thread: connects, streams samples into
/// `on_sample`, and reconnects on disconnect. Blocking I/O is fine here
/// since this has its own dedicated OS thread (see main.rs).
pub fn run_tcp_bridge_blocking(on_sample: impl Fn(AccelSample) + Send + 'static) {
    loop {
        match connect_and_stream(&on_sample) {
            Ok(()) => eprintln!("motion-dots: TCP stream ended cleanly, reconnecting in 2s..."),
            Err(e) => eprintln!("motion-dots: TCP error ({e}), reconnecting in 2s..."),
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn connect_and_stream(
    on_sample: &(impl Fn(AccelSample) + Send + 'static),
) -> std::io::Result<()> {
    eprintln!("motion-dots: connecting to {ADDR} (make sure `adb forward tcp:8765 tcp:8765` is running)...");
    let mut stream = TcpStream::connect(ADDR)?;
    stream.set_nodelay(true).ok();
    eprintln!("motion-dots: connected, streaming accel data");

    let mut buf = [0u8; 12];
    loop {
        stream.read_exact(&mut buf)?;
        let x = f32::from_le_bytes(buf[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(buf[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(buf[8..12].try_into().unwrap());
        on_sample(AccelSample { x, y, z });
    }
}
