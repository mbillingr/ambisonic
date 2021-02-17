use rodio::Source;
use std::time::Duration;

/// Constant source
pub struct Ramp {
    sample_rate: u32,
    value: f32,
}

impl Ramp {
    pub fn new(sample_rate: u32) -> Self {
        Ramp {
            sample_rate,
            value: 0.0,
        }
    }
}

impl Iterator for Ramp {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let x = self.value;
        self.value += 1.0 / self.sample_rate as f32;
        Some(x)
    }
}

impl Source for Ramp {
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        1
    }

    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline(always)]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
