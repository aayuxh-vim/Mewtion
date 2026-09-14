//! Built-in motion sensors via the Linux IIO (Industrial I/O) sysfs interface.
//!
//! Laptops with a built-in accelerometer expose it under
//! `/sys/bus/iio/devices/iio:deviceN`, either from a dedicated driver or
//! through the HID sensor hub. Devices are discovered by the channels they
//! expose rather than by driver name, so no per-vendor allowlist is needed,
//! and a hub that splits accelerometer, gyroscope and gravity across sibling
//! devices works the same as a single combined device.
//!
//! Reading `*_raw` is the portable interface, but it is not a cheap memory
//! fetch on every driver: a HID sensor hub answers each read by requesting a
//! fresh report, which costs about one sampling period per axis and does not
//! parallelise. An accelerometer parked at 10 Hz therefore yields roughly
//! 5 Hz here, so this module reads only the axes the overlay consumes and
//! paces itself from the rate the hardware reports. Dedicated I2C
//! accelerometers return a cached register value immediately and run at the
//! full loop rate.

use crate::sensor::MotionSample;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

const IIO_DEVICES: &str = "/sys/bus/iio/devices";

/// Sampling faster than this gains nothing: the overlay redraws at 60 FPS and
/// vehicle motion has no content at these frequencies.
const MAX_POLL_HZ: f32 = 100.0;

/// Used when a device does not publish a sampling frequency.
const DEFAULT_POLL_HZ: f32 = 50.0;

/// Cutoff of the fallback gravity estimator. Low enough to pass sustained
/// orientation but not vehicle acceleration.
const GRAVITY_CUTOFF_HZ: f32 = 0.25;

/// Lateral and forward axes. The overlay ignores vertical acceleration, and on
/// a HID sensor hub every skipped accelerometer axis saves a full sampling
/// period, so axes outside this mask are left at zero in the emitted sample.
const PLANAR: [bool; 3] = [true, true, false];

/// Yaw rate, the only angular velocity the overlay mixes in.
const YAW: [bool; 3] = [false, false, true];

fn read_f32(path: &Path) -> Option<f32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// One three-axis channel group of a device, e.g. `in_accel_*`.
struct Triple {
    axes: [PathBuf; 3],
    scale: [f32; 3],
    offset: [f32; 3],
}

impl Triple {
    /// Opens `kind` on `dev`, or `None` if the device has no such channel.
    fn open(dev: &Path, kind: &str) -> Option<Triple> {
        let axes = ["x", "y", "z"].map(|axis| dev.join(format!("in_{kind}_{axis}_raw")));
        if !axes.iter().all(|path| path.exists()) {
            return None;
        }

        // Scale and offset are published either once for the whole group or
        // per axis; per-axis wins where both exist.
        let attr = |axis: &str, name: &str| {
            read_f32(&dev.join(format!("in_{kind}_{axis}_{name}")))
                .or_else(|| read_f32(&dev.join(format!("in_{kind}_{name}"))))
        };

        Some(Triple {
            axes,
            scale: ["x", "y", "z"].map(|axis| attr(axis, "scale").unwrap_or(1.0)),
            offset: ["x", "y", "z"].map(|axis| attr(axis, "offset").unwrap_or(0.0)),
        })
    }

    /// Reads the requested axes, leaving the rest at zero. The IIO ABI defines
    /// the processed value as `(raw + offset) * scale`, in m/s² for
    /// acceleration and rad/s for angular velocity.
    fn read(&self, wanted: [bool; 3]) -> Option<[f32; 3]> {
        let mut values = [0.0; 3];
        for (i, value) in values.iter_mut().enumerate() {
            if wanted[i] {
                *value = (read_f32(&self.axes[i])? + self.offset[i]) * self.scale[i];
            }
        }
        Some(values)
    }
}

/// Reads the rate a channel is currently running at.
fn sampling_rate(dev: &Path, kind: &str) -> Option<f32> {
    read_f32(&dev.join(format!("in_{kind}_sampling_frequency"))).filter(|hz| *hz > 0.0)
}

/// Sensor hubs commonly idle at 10 Hz, which bounds how fast this path can
/// sample. Raising it needs write access to a root-owned attribute, so failure
/// is expected and simply leaves the device at its default rate.
fn raise_sampling_rate(dev: &Path, kind: &str) {
    let frequency = dev.join(format!("in_{kind}_sampling_frequency"));
    if !frequency.exists() || sampling_rate(dev, kind).is_some_and(|hz| hz >= MAX_POLL_HZ) {
        return;
    }

    let supported: Vec<f32> = fs::read_to_string(
        dev.join(format!("in_{kind}_sampling_frequency_available")),
    )
    .map(|list| {
        list.split_whitespace()
            .filter_map(|value| value.parse().ok())
            .collect()
    })
    .unwrap_or_default();

    // Prefer the slowest supported rate that still keeps up; if the device
    // cannot reach it, take the fastest it offers.
    let target = supported
        .iter()
        .copied()
        .filter(|hz| *hz >= MAX_POLL_HZ)
        .min_by(|a, b| a.total_cmp(b))
        .or_else(|| supported.iter().copied().max_by(|a, b| a.total_cmp(b)))
        .unwrap_or(MAX_POLL_HZ);

    let _ = fs::write(&frequency, format!("{target}"));
}

pub struct IioSource {
    accel: Triple,
    gyro: Option<Triple>,
    gravity: Option<Triple>,
    /// Rate the accelerometer reports, which bounds the whole loop.
    accel_hz: f32,
}

impl IioSource {
    /// Finds the local motion sensors, or `None` when the machine has no
    /// accelerometer exposed through IIO.
    pub fn discover() -> Option<IioSource> {
        let mut devices: Vec<PathBuf> = fs::read_dir(IIO_DEVICES)
            .ok()?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("iio:device"))
            })
            .collect();
        devices.sort();

        let (mut accel, mut gyro, mut gravity) = (None, None, None);
        let mut accel_hz = DEFAULT_POLL_HZ;

        for dev in &devices {
            for (slot, kind) in [
                (&mut accel, "accel"),
                (&mut gyro, "anglvel"),
                (&mut gravity, "gravity"),
            ] {
                if slot.is_none() {
                    if let Some(triple) = Triple::open(dev, kind) {
                        raise_sampling_rate(dev, kind);
                        if kind == "accel" {
                            accel_hz = sampling_rate(dev, kind).unwrap_or(DEFAULT_POLL_HZ);
                        }
                        *slot = Some(triple);
                    }
                }
            }
        }

        Some(IioSource {
            accel: accel?,
            gyro,
            gravity,
            accel_hz,
        })
    }

    /// Rate the loop aims for, bounded by what the accelerometer produces.
    fn poll_hz(&self) -> f32 {
        self.accel_hz.min(MAX_POLL_HZ)
    }

    pub fn describe(&self) -> String {
        let gravity = match self.gravity {
            Some(_) => "hardware gravity channel",
            None => "estimated gravity",
        };
        let gyro = match self.gyro {
            Some(_) => "gyroscope",
            None => "no gyroscope",
        };
        format!("accelerometer at {:.0} Hz, {gyro}, {gravity}", self.poll_hz())
    }
}

/// Streams samples until the process exits, calling `on_sample` for each one.
///
/// The overlay wants linear acceleration. Where the device publishes a gravity
/// channel it is subtracted directly; otherwise gravity is tracked with a
/// low-pass filter and subtracted, which is how Android derives
/// `TYPE_LINEAR_ACCELERATION` on devices without a dedicated sensor.
pub fn run_iio_source_blocking<F>(source: IioSource, mut on_sample: F)
where
    F: FnMut(MotionSample),
{
    let period = Duration::from_secs_f32(1.0 / source.poll_hz());
    let rc = 1.0 / (2.0 * std::f32::consts::PI * GRAVITY_CUTOFF_HZ);

    let mut estimated_gravity: Option<[f32; 3]> = None;
    let mut last_sample_at: Option<Instant> = None;

    loop {
        let started = Instant::now();

        // A transient read failure (sensor resuming, hub busy) should skip the
        // sample rather than end the stream.
        let Some(accel) = source.accel.read(PLANAR) else {
            sleep(period);
            continue;
        };

        let gravity = match source.gravity.as_ref().and_then(|g| g.read(PLANAR)) {
            Some(gravity) => gravity,
            None => {
                // Drivers that block until a fresh report and drivers that
                // return instantly give very different intervals, so derive
                // the filter constant from the interval actually observed.
                let dt = last_sample_at
                    .map(|at| at.elapsed().as_secs_f32())
                    .unwrap_or(period.as_secs_f32())
                    .clamp(1e-3, 1.0);
                let alpha = dt / (rc + dt);

                // Seeding from the first reading avoids a lurch across the
                // screen while the filter converges from zero.
                let gravity = estimated_gravity.get_or_insert(accel);
                for (axis, sampled) in gravity.iter_mut().zip(accel) {
                    *axis += alpha * (sampled - *axis);
                }
                *gravity
            }
        };

        let gyro = source
            .gyro
            .as_ref()
            .and_then(|g| g.read(YAW))
            .unwrap_or_default();

        last_sample_at = Some(Instant::now());

        on_sample(MotionSample {
            ax: accel[0] - gravity[0],
            ay: accel[1] - gravity[1],
            az: accel[2] - gravity[2],
            gx: gyro[0],
            gy: gyro[1],
            gz: gyro[2],
        });

        // Drivers that block until a fresh report have already consumed the
        // interval; only sleep off whatever is left, so fast drivers are paced
        // instead of spinning a core.
        if let Some(remaining) = period.checked_sub(started.elapsed()) {
            sleep(remaining);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mewtion-iio-tests-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create isolated IIO fixture directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, name: &str, contents: &str) {
            fs::write(self.0.join(name), contents).expect("write IIO fixture attribute");
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove isolated IIO fixture directory");
        }
    }

    fn write_axes(dir: &TempDir, kind: &str, values: [&str; 3]) {
        for (axis, value) in ["x", "y", "z"].into_iter().zip(values) {
            dir.write(&format!("in_{kind}_{axis}_raw"), value);
        }
    }

    fn triple_with_zeroes(dir: &TempDir, kind: &str) -> Triple {
        write_axes(dir, kind, ["0", "0", "0"]);
        Triple::open(dir.path(), kind).expect("complete channel group should open")
    }

    #[test]
    fn read_f32_trims_valid_values_and_rejects_invalid_inputs() {
        let dir = TempDir::new();
        dir.write("valid", "  -12.75\n");
        dir.write("invalid", "12.5 Hz");

        assert_eq!(read_f32(&dir.path().join("valid")), Some(-12.75));
        assert_eq!(read_f32(&dir.path().join("invalid")), None);
        assert_eq!(read_f32(&dir.path().join("missing")), None);
    }

    #[test]
    fn triple_open_requires_all_three_raw_axes() {
        let dir = TempDir::new();
        dir.write("in_accel_x_raw", "1");
        dir.write("in_accel_y_raw", "2");

        assert!(Triple::open(dir.path(), "accel").is_none());

        dir.write("in_accel_z_raw", "3");
        assert!(Triple::open(dir.path(), "accel").is_some());
    }

    #[test]
    fn triple_read_applies_attribute_precedence_defaults_and_axis_mask() {
        let dir = TempDir::new();
        write_axes(&dir, "accel", ["2", "4", "not-a-number"]);
        dir.write("in_accel_scale", "10");
        dir.write("in_accel_y_scale", "0.5");
        dir.write("in_accel_offset", "1");
        dir.write("in_accel_x_offset", "-1");

        let triple = Triple::open(dir.path(), "accel").expect("complete accelerometer");

        assert_eq!(triple.scale, [10.0, 0.5, 10.0]);
        assert_eq!(triple.offset, [-1.0, 1.0, 1.0]);
        assert_eq!(triple.read(PLANAR), Some([10.0, 2.5, 0.0]));
        assert_eq!(triple.read([false; 3]), Some([0.0; 3]));
        assert_eq!(triple.read([false, false, true]), None);
    }

    #[test]
    fn triple_open_uses_identity_conversion_when_attributes_are_unusable() {
        let dir = TempDir::new();
        write_axes(&dir, "anglvel", ["1.5", "-2", "3"]);
        dir.write("in_anglvel_scale", "invalid");
        dir.write("in_anglvel_offset", "invalid");

        let triple = Triple::open(dir.path(), "anglvel").expect("complete gyroscope");

        assert_eq!(triple.read([true; 3]), Some([1.5, -2.0, 3.0]));
    }

    #[test]
    fn sampling_rate_accepts_only_positive_numeric_frequencies() {
        let dir = TempDir::new();
        let frequency = "in_accel_sampling_frequency";

        assert_eq!(sampling_rate(dir.path(), "accel"), None);
        for invalid in ["0", "-25", "invalid"] {
            dir.write(frequency, invalid);
            assert_eq!(sampling_rate(dir.path(), "accel"), None);
        }

        dir.write(frequency, "62.5\n");
        assert_eq!(sampling_rate(dir.path(), "accel"), Some(62.5));
    }

    #[test]
    fn raise_sampling_rate_chooses_slowest_supported_rate_at_or_above_cap() {
        let dir = TempDir::new();
        dir.write("in_accel_sampling_frequency", "10");
        dir.write(
            "in_accel_sampling_frequency_available",
            "garbage 200 50 125 100",
        );

        raise_sampling_rate(dir.path(), "accel");

        assert_eq!(
            fs::read_to_string(dir.path().join("in_accel_sampling_frequency")).unwrap(),
            "100"
        );
    }

    #[test]
    fn raise_sampling_rate_uses_fastest_supported_rate_when_cap_is_unavailable() {
        let dir = TempDir::new();
        dir.write("in_accel_sampling_frequency", "10");
        dir.write("in_accel_sampling_frequency_available", "12.5 25 50");

        raise_sampling_rate(dir.path(), "accel");

        assert_eq!(
            fs::read_to_string(dir.path().join("in_accel_sampling_frequency")).unwrap(),
            "50"
        );
    }

    #[test]
    fn raise_sampling_rate_defaults_to_cap_when_supported_rates_are_missing() {
        let dir = TempDir::new();
        dir.write("in_accel_sampling_frequency", "10");

        raise_sampling_rate(dir.path(), "accel");

        assert_eq!(
            fs::read_to_string(dir.path().join("in_accel_sampling_frequency")).unwrap(),
            "100"
        );
    }

    #[test]
    fn raise_sampling_rate_leaves_absent_or_fast_frequency_attributes_alone() {
        let absent = TempDir::new();
        raise_sampling_rate(absent.path(), "accel");
        assert!(!absent.path().join("in_accel_sampling_frequency").exists());

        let fast = TempDir::new();
        fast.write("in_accel_sampling_frequency", "125.5");
        fast.write("in_accel_sampling_frequency_available", "200");
        raise_sampling_rate(fast.path(), "accel");
        assert_eq!(
            fs::read_to_string(fast.path().join("in_accel_sampling_frequency")).unwrap(),
            "125.5"
        );
    }

    #[test]
    fn source_poll_rate_is_capped_and_description_reports_available_channels() {
        let dir = TempDir::new();
        let source = IioSource {
            accel: triple_with_zeroes(&dir, "accel"),
            gyro: Some(triple_with_zeroes(&dir, "anglvel")),
            gravity: Some(triple_with_zeroes(&dir, "gravity")),
            accel_hz: 250.0,
        };

        assert_eq!(source.poll_hz(), MAX_POLL_HZ);
        assert_eq!(
            source.describe(),
            "accelerometer at 100 Hz, gyroscope, hardware gravity channel"
        );
    }

    #[test]
    fn source_description_reports_estimated_gravity_and_missing_gyro() {
        let dir = TempDir::new();
        let source = IioSource {
            accel: triple_with_zeroes(&dir, "accel"),
            gyro: None,
            gravity: None,
            accel_hz: 62.5,
        };

        assert_eq!(source.poll_hz(), 62.5);
        assert_eq!(
            source.describe(),
            "accelerometer at 62 Hz, no gyroscope, estimated gravity"
        );
    }

    #[test]
    #[should_panic(expected = "sample verified")]
    fn stream_subtracts_hardware_gravity_and_reads_only_yaw_rate() {
        let dir = TempDir::new();
        write_axes(&dir, "accel", ["10", "20", "unread vertical axis"]);
        write_axes(&dir, "gravity", ["1", "2", "unread vertical axis"]);
        write_axes(
            &dir,
            "anglvel",
            ["unread roll axis", "unread pitch axis", "3.5"],
        );
        let source = IioSource {
            accel: Triple::open(dir.path(), "accel").unwrap(),
            gyro: Triple::open(dir.path(), "anglvel"),
            gravity: Triple::open(dir.path(), "gravity"),
            accel_hz: MAX_POLL_HZ,
        };

        run_iio_source_blocking(source, |sample| {
            assert_eq!(sample.ax, 9.0);
            assert_eq!(sample.ay, 18.0);
            assert_eq!(sample.az, 0.0);
            assert_eq!(sample.gx, 0.0);
            assert_eq!(sample.gy, 0.0);
            assert_eq!(sample.gz, 3.5);
            panic!("sample verified");
        });
    }

    #[test]
    #[should_panic(expected = "sample verified")]
    fn estimated_gravity_seeds_from_first_reading_without_initial_lurch() {
        let dir = TempDir::new();
        write_axes(&dir, "accel", ["4", "-7", "unread vertical axis"]);
        let source = IioSource {
            accel: Triple::open(dir.path(), "accel").unwrap(),
            gyro: None,
            gravity: None,
            accel_hz: MAX_POLL_HZ,
        };

        run_iio_source_blocking(source, |sample| {
            assert_eq!(sample.ax, 0.0);
            assert_eq!(sample.ay, 0.0);
            assert_eq!(sample.az, 0.0);
            assert_eq!(sample.gz, 0.0);
            panic!("sample verified");
        });
    }
}
