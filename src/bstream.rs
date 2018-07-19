use std::sync::Arc;
use std::time::Duration;

use rodio::Source;

use bformat::{Bformat, Bweights};

pub struct Bstream {
    input: Box<Source<Item = f32> + Send>,
    bweights: Bweights,
}

impl Bstream {
    pub fn new<I: Source<Item = f32> + Send + 'static>(source: I, pos: [f32; 3]) -> Self {
        Bstream {
            input: Box::new(source),
            bweights: Bweights::from_position(pos),
        }
    }
}

impl Source for Bstream {
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        assert_eq!(self.input.channels(), 1);
        1
    }

    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline(always)]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}

impl Iterator for Bstream {
    type Item = Bformat;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.input.next()?;
        Some(self.bweights.scale(x))
    }
}
