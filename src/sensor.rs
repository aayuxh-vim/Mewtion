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
