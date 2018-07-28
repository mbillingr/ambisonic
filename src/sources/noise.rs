use rand::{
    distributions::{Distribution, StandardNormal}, thread_rng,
};
use rodio::Source;
use std::time::Duration;

pub struct Noise {
    sample_rate: u32,
}

impl Noise {
    pub fn new(sample_rate: u32) -> Self {
        Noise { sample_rate }
    }
}

impl Iterator for Noise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(StandardNormal.sample(&mut thread_rng()) as f32)
    }
}

impl Source for Noise {
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
