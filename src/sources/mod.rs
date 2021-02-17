//! Useful implementations of `rodio::Source`

mod noise;
mod constant;

pub use self::noise::Noise;
pub use self::constant::Constant;
