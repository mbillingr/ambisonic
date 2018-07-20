//! Render *B-format* audio streams to streams suitable for playback on audio equipment.

use std::time::Duration;

use rodio::{Sample, Source};

use bformat::{Bformat, Bweights};
use bmixer::BstreamMixer;

/// Render a *B-format* stream to a stereo representation.
///
/// Suitable for playback over two speakers arranged in front of the user.
/// The default setting assumes a symmetric arrangement of +/- 45º.
pub struct BstreamStereoRenderer<I> {
    input: I,
    buffered_sample: Option<f32>,
    left_mic: Bweights,
    right_mic: Bweights,
}

impl<I> BstreamStereoRenderer<I> {
    /// Construct a new stereo renderer with default settings
    pub fn new(input: I) -> Self {
        BstreamStereoRenderer {
            input,
            buffered_sample: None,
            left_mic: Bweights::virtual_microphone([-1.0, 1.0, 0.0], 0.5),
            right_mic: Bweights::virtual_microphone([1.0, 1.0, 0.0], 0.5),
        }
    }
}

impl<I> Source for BstreamStereoRenderer<I>
where
    I: Source<Item = Bformat>,
{
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        2 // well, it's stereo...
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

impl<I> Iterator for BstreamStereoRenderer<I>
where
    I: Source<Item = Bformat>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffered_sample.take() {
            Some(s) => Some(s),
            None => {
                let sample = self.input.next()?;

                let left = self.left_mic.dot(sample);
                let right = self.right_mic.dot(sample);

                // emit left channel now, and right channel next time
                self.buffered_sample = Some(right);
                Some(left)
            }
        }
    }
}
