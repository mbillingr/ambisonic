//! *B-format* representation of audio samples

use cpal::{Sample as CpalSample, SampleFormat};
use rodio::Sample;

/// Audio sample in first-order *B-format*.
///
/// It encodes four components of the sound field at the lister position: omnidirectional level `w`
/// and the level gradient in `x`, `y`, and `z` directions.
#[derive(Copy, Clone)]
pub struct Bformat {
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

/// Weights for manipulating `Bformat` samples.
#[derive(Copy, Clone)]
pub struct Bweights {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Bweights {
    /// Weights that correspond to a omnidirectional source
    pub fn omni_source() -> Self {
        Bweights {
            w: 1.0 / 2f32.sqrt(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Compute weights that correspond to a sound source at given position.
    pub fn from_position(pos: [f32; 3]) -> Self {
        let dist = (pos[0] * pos[0] + pos[1] * pos[1] + pos[1] * pos[2]).sqrt();
        let falloff = 1.0 / dist.max(1.0); // todo: proper falloff and distance model(s)
        Bweights {
            w: falloff / 2f32.sqrt(),
            x: falloff * pos[0] / dist,
            y: falloff * pos[1] / dist,
            z: falloff * pos[2] / dist,
        }
    }

    /// Compute weights that correspond to a virtual microphone at the listener position.
    ///
    /// It takes a `direction` in which the microphone points (does not need to be normalized), and
    /// a directional characteristic 0 <= `p` <= 1. A `p==1` corresponds to an omnidirectional
    /// microphone, `p==0` to a bi-directional microphone, and `p==0.5` to a cardioid microphone
    /// (https://en.wikipedia.org/wiki/Microphone#Polar_patterns).
    pub fn virtual_microphone(direction: [f32; 3], p: f32) -> Self {
        let l = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[1] * direction[2])
            .sqrt();
        Bweights {
            w: p * 2f32.sqrt(),
            x: direction[0] * (1.0 - p) / l,
            y: direction[1] * (1.0 - p) / l,
            z: direction[2] * (1.0 - p) / l,
        }
    }

    /// Dot product of *B-format* weights and sample.
    ///
    /// If the weights correspond to a virtual microphone, the result is the signal recorded by that
    /// microphone.
    pub fn dot(&self, b: Bformat) -> f32 {
        self.w * b.w + self.x * b.x + self.y * b.y + self.z * b.z
    }

    /// Produce a *B-format* sample by scaling weights.
    ///
    /// If the weights correspond to a sound source, and `s` is the source's current level, the
    /// result is the *B-format* representation of the source.
    pub fn scale(&self, s: f32) -> Bformat {
        Bformat {
            w: self.w * s,
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}
