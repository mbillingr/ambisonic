//! *B-format* representation of audio samples

use std::f32;
use std::iter::{FromIterator, Sum};
use std::ops;

use cpal::{Sample as CpalSample, SampleFormat};
use rodio::Sample;

/// Audio sample in first-order *B-format*.
///
/// It encodes four components of the sound field at the lister position: omnidirectional level `w`
/// and the level gradient in `x`, `y`, and `z` directions.
#[derive(Debug, Copy, Clone)]
pub struct Bformat {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Bformat {
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Bformat {w, x, y, z}
    }
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

impl ops::Mul<Bweights> for Bformat {

    type Output = Bformat;

    fn mul(self, rhs: Bweights) -> Bformat {
        Bformat {
            w: self.w * rhs.w,
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Add<Bformat> for Bformat {

    type Output = Bformat;

    fn add(self, rhs: Bformat) -> Bformat {
        self.saturating_add(rhs)
    }
}

impl Sum for Bformat {

    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Bformat::zero_value(), Bformat::saturating_add)
    }
}

/// Weights for manipulating `Bformat` samples.
#[derive(Debug, Copy, Clone)]
pub struct Bweights {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Bweights {
    /// Initialze new weights with given values
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Bweights { w, x, y, z }
    }

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
        let dist = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
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

    /// adjust weights towards target
    pub fn approach(&mut self, target: &Bweights, max_step: f32) {
        // if this turns out too slow we could try to replace it with simple steps along each dimension
        let dir = [
            target.w - self.w,
            target.x - self.x,
            target.y - self.y,
            target.z - self.z,
        ];
        let dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2] + dir[3] * dir[3]).sqrt();
        if dist <= max_step {
            *self = *target;
        } else {
            let d = max_step / dist;
            self.w += dir[0] * d;
            self.x += dir[1] * d;
            self.y += dir[2] * d;
            self.z += dir[3] * d;
        }
    }
}

impl FromIterator<f32> for Bweights {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let bw = Bweights {
            w: iter.next().unwrap(),
            x: iter.next().unwrap(),
            y: iter.next().unwrap(),
            z: iter.next().unwrap(),
        };
        assert!(iter.next().is_none());
        bw
    }
}

/// Arbitrary matrix for manipulating `Bformat` samples.
pub enum Btransform {
    Matrix([f32; 16]),
    Quaternion{r: f32, i: f32, j: f32, k: f32}
}

impl Btransform {
    pub fn from_matrix(matrix: [f32; 16]) -> Self {
        Btransform::Matrix(matrix)
    }

    pub fn zero() -> Self {
        Btransform::Matrix([0.0; 16])
    }

    pub fn identity() -> Self {
        Btransform::Matrix([1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0, 0.0, 0.0,
                            0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 1.0])
    }

    /// The user is responsible for passing a unit vector as `axis`. This is not checked!
    pub fn rotation(angle: f32, axis: [f32; 3]) -> Self {
        let r = (angle * 0.5).cos();
        let s = (angle * 0.5).sin();
        Btransform::Quaternion {
            r,
            i: axis[0] * s,
            j: axis[1] * s,
            k: axis[2] * s,
        }
    }

    pub fn apply(&self, b: Bformat) -> Bformat {
        match self {
            Btransform::Matrix(m) => Bformat {
                w: b.w * m[0] + b.x * m[1] + b.y * m[2] + b.z * m[3],
                x: b.w * m[4] + b.x * m[5] + b.y * m[6] + b.z * m[7],
                y: b.w * m[8] + b.x * m[9] + b.y * m[10] + b.z * m[11],
                z: b.w * m[12] + b.x * m[13] + b.y * m[14] + b.z * m[15],
            },

            Btransform::Quaternion{r, i, j, k} => {
                let (a2, b2, c2, d2) = (r * r, i * i, j * j, k * k);
                let ab = 2.0 * r * i;
                let ac = 2.0 * r * j;
                let ad = 2.0 * r * k;
                let bc = 2.0 * i * j;
                let bd = 2.0 * i * k;
                let cd = 2.0 * j * k;

                Bformat
                {
                    w: b.w,
                    x: b.x * (a2 + b2 - c2 - d2) + b.y * (bc - ad) + b.z * (bd + ac),
                    y: b.x * (bc + ad) + b.y * (a2 - b2 + c2 - d2) + b.z * (cd - ab),
                    z: b.x * (bd - ac) + b.y * (cd + ab) + b.z * (a2 - b2 - c2 + d2),
                }
            }
        }
    }

    pub fn matrix_form(self) -> Self {
        match self {
            Btransform::Matrix(m) => self,
            Btransform::Quaternion{r, i, j, k} => {
                let (a2, b2, c2, d2) = (r * r, i * i, j * j, k * k);
                let ab = 2.0 * r * i;
                let ac = 2.0 * r * j;
                let ad = 2.0 * r * k;
                let bc = 2.0 * i * j;
                let bd = 2.0 * i * k;
                let cd = 2.0 * j * k;
                
                Btransform::Matrix([
                    1.0, 0.0, 0.0, 0.0,
                    0.0, a2 + b2 - c2 - d2, bc - ad, bd + ac,
                    0.0, bc + ad, a2 - b2 + c2 - d2, cd - ab,
                    0.0, bd - ac, cd + ab, a2 - b2 - c2 + d2,
                ])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_close {
        ($left:expr, $right:expr, $tol:expr) => ({
            match (&$left, &$right) {
                (left_val, right_val) => {
                    assert!((left_val.w - right_val.w).abs() < $tol);
                    assert!((left_val.x - right_val.x).abs() < $tol);
                    assert!((left_val.y - right_val.y).abs() < $tol);
                    assert!((left_val.z - right_val.z).abs() < $tol);
              }
            }
        });

        ($left:expr, $right:expr) => ( {
            assert_close!($left, $right, 1e-6)
        });
    }

    #[test]
    fn zero() {
        let x = Bformat::new(1.0, 20.0, 0.3, 42.0);
        assert_close!(Btransform::zero().apply(x), Bformat::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn identity() {
        let x = Bformat::new(1.0, 20.0, 0.3, 42.0);
        assert_close!(Btransform::identity().apply(x), x);
    }

    #[test]
    fn rotate() {
        let x = Bformat::new(2.0, 1.0, 0.0, 0.0);

        let r = Btransform::rotation(0.0, [1.0, 2.0, 3.0]);
        assert_close!(r.apply(x), x);
        assert_close!(r.matrix_form().apply(x), x);

        let r = Btransform::rotation(f32::consts::PI, [0.0, 1.0, 0.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, -1.0, 0.0, 0.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, -1.0, 0.0, 0.0));

        let r = Btransform::rotation(f32::consts::PI, [0.0, 0.0, 1.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, -1.0, 0.0, 0.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, -1.0, 0.0, 0.0));

        let r = Btransform::rotation(f32::consts::PI, [1.0, 0.0, 0.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, 1.0, 0.0, 0.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, 1.0, 0.0, 0.0));

        let r = Btransform::rotation(f32::consts::PI / 2.0, [0.0, 1.0, 0.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, 0.0, 0.0, -1.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, 0.0, 0.0, -1.0));

        let r = Btransform::rotation(f32::consts::PI / -2.0, [0.0, 1.0, 0.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, 0.0, 0.0, 1.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, 0.0, 0.0, 1.0));

        let r = Btransform::rotation(f32::consts::PI / 2.0, [0.0, 0.0, 1.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, 0.0, 1.0, 0.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, 0.0, 1.0, 0.0));

        let r = Btransform::rotation(f32::consts::PI / -2.0, [0.0, 0.0, 1.0]);
        assert_close!(r.apply(x), Bformat::new(2.0, 0.0, -1.0, 0.0));
        assert_close!(r.matrix_form().apply(x), Bformat::new(2.0, 0.0, -1.0, 0.0));

    }
}