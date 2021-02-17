//! Useful implementations of `rodio::Source`

mod constant;
mod noise;
mod ramp;

pub use self::constant::Constant;
pub use self::noise::Noise;
pub use self::ramp::Ramp;
