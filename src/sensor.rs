/// A single motion reading, shared by every sensor source.
///
/// Acceleration is linear acceleration in m/s² (gravity already removed,
/// matching Android's `TYPE_LINEAR_ACCELERATION`) and angular velocity is in
/// rad/s.
///
/// A source may leave axes the overlay does not consume at zero rather than
/// pay to read them; see the `iio` module.
#[derive(Debug, Clone, Copy, Default)]
pub struct MotionSample {
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub gx: f32,
    pub gy: f32,
    pub gz: f32,
}

#[cfg(test)]
mod tests {
    use super::MotionSample;

    #[test]
    fn default_sample_has_no_motion_on_any_axis() {
        let sample = MotionSample::default();

        assert_eq!(sample.ax, 0.0);
        assert_eq!(sample.ay, 0.0);
        assert_eq!(sample.az, 0.0);
        assert_eq!(sample.gx, 0.0);
        assert_eq!(sample.gy, 0.0);
        assert_eq!(sample.gz, 0.0);
    }

    #[test]
    fn samples_are_copyable_without_losing_axis_values() {
        let original = MotionSample {
            ax: 1.0,
            ay: -2.0,
            az: 3.5,
            gx: -4.25,
            gy: 5.0,
            gz: -6.0,
        };
        let copied = original;

        assert_eq!(copied.ax, 1.0);
        assert_eq!(copied.ay, -2.0);
        assert_eq!(copied.az, 3.5);
        assert_eq!(copied.gx, -4.25);
        assert_eq!(copied.gy, 5.0);
        assert_eq!(copied.gz, -6.0);
        assert_eq!(original.ax, 1.0);
    }
}
