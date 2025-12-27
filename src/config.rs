/// Default audio sample rate (CD quality)
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
/// Default bit depth
pub const DEFAULT_BIT_DEPTH: u16 = 16;
/// Default amplitude (leaves headroom to prevent clipping)
pub const AMPLITUDE: f64 = 0.8;

/// Audio configuration for sample rate and bit depth
#[derive(Clone, Copy)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub bit_depth: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            bit_depth: DEFAULT_BIT_DEPTH,
        }
    }
}
