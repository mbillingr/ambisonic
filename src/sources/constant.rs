use rodio::Source;
use std::time::Duration;

/// Constant source
pub struct Constant {
    sample_rate: u32,
    value: f32,
}

impl Constant {
    pub fn new(value: f32, sample_rate: u32) -> Self {
        Constant { sample_rate, value }
    }
}

impl Iterator for Constant {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.value)
    }
}

impl Source for Constant {
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
