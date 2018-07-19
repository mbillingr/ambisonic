extern crate cpal;
pub extern crate rodio;

use std::time::Duration;

use cpal::{Sample as CpalSample, SampleFormat};
use rodio::{Sample, Source};


#[derive(Copy, Clone)]
struct Bweights {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Bweights {
    fn dot(&self, b: Bformat) -> f32 {
        self.w * b.w + self.x * b.x + self.y * b.y + self.z * b.z
    }

    fn virtual_microphone(direction: [f32; 3], p: f32) -> Bweights {
        Bweights {
            w: p * 2f32.sqrt(),
            x: direction[0] * (1.0 - p),
            y: direction[1] * (1.0 - p),
            z: direction[2] * (1.0 - p),
        }
    }
}


#[derive(Copy, Clone)]
struct Bformat {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Sample for Bformat {
    fn lerp(first: Self, second: Self, numerator: u32, denominator: u32) -> Self {
        let alpha = numerator as f32 / denominator as f32;
        Bformat {
            w: first.w * alpha + second.w * (1.0 - alpha),
            x: first.x * alpha + second.x * (1.0 - alpha),
            y: first.y * alpha + second.y * (1.0 - alpha),
            z: first.z * alpha + second.z * (1.0 - alpha),
        }
    }

    fn amplify(self, alpha: f32) -> Self {
        Bformat {
            w: self.w * alpha,
            x: self.x * alpha,
            y: self.y * alpha,
            z: self.z * alpha,
        }
    }

    fn saturating_add(self, other: Self) -> Self {
        Bformat {
            w: self.w + other.w,
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn zero_value() -> Self {
        Bformat {
            w: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

// Why? Oh, why!?
unsafe impl CpalSample for Bformat {
    fn get_format() -> SampleFormat {
        panic!("The B-format is not intended to be used as a CPAL sample directly. Use a renderer instead.")
    }

    fn to_f32(&self) -> f32 {
        panic!("The B-format is not intended to be used as a CPAL sample directly. Use a renderer instead.")
    }

    fn to_i16(&self) -> i16 {
        panic!("The B-format is not intended to be used as a CPAL sample directly. Use a renderer instead.")
    }

    fn to_u16(&self) -> u16 {
        panic!("The B-format is not intended to be used as a CPAL sample directly. Use a renderer instead.")
    }

    fn from<S>(_s: &S) -> Self
    where
        S: CpalSample,
    {
        panic!("The B-format is not intended to be used as a CPAL sample directly. Use a renderer instead.")
    }
}

struct BstreamStereoRenderer {
    left_mic: Bweights,
    right_mic: Bweights,
    buffered_sample: Option<f32>,
    bstream: Source<Item = Bformat>,
}

impl Source for BstreamStereoRenderer {
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.bstream.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        2 // well, it's stereo...
    }

    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.bstream.sample_rate()
    }

    #[inline(always)]
    fn total_duration(&self) -> Option<Duration> {
        self.bstream.total_duration()
    }
}

impl Iterator for BstreamStereoRenderer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffered_sample.take() {
            Some(s) => Some(s),
            None => {
                let sample = self.bstream.next()?;

                let left = self.left_mic.dot(sample);
                let right = self.right_mic.dot(sample);

                // emit left channel now, and right channel next time
                self.buffered_sample = Some(right);
                Some(left)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
