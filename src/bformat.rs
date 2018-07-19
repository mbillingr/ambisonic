use cpal::{Sample as CpalSample, SampleFormat};
use rodio::{Sample, Source};

#[derive(Copy, Clone)]
pub struct Bweights {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Bweights {
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

    pub fn dot(&self, b: Bformat) -> f32 {
        self.w * b.w + self.x * b.x + self.y * b.y + self.z * b.z
    }

    pub fn scale(&self, s: f32) -> Bformat {
        Bformat {
            w: self.w * s,
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

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
}

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
