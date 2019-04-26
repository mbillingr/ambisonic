//! ## Compose and play 3D audio.
//!
//! The library provides 3D sound scene support on top of [`rodio`](https://crates.io/crates/rodio).
//! It allows positioning and moving sound sources freely in 3D space around a virtual listener,
//! and playing the resulting spatial mix in real-time over a sound card.
//!
//! `ambisonic` is built around the concept of an intermediate representation of the sound field,
//! called *B-format*. The *B-format* describes what the listener should hear, independent of
//! their audio playback equipment. This leads to a clear separation of audio scene composition and
//! rendering. For details, see [Wikipedia](https://en.wikipedia.org/wiki/Ambisonics).
//!
//! In its current state, the library allows spatial composition of single-channel `rodio` sources
//! into a first-order *B-format* stream. The chosen renderer then decodes the *B-format* stream
//! into audio signals for playback.
//!
//! Currently, the following renderers are available:
//!
//! - Stereo: simple and efficient playback on two stereo speakers or headphones
//! - HRTF: realistic 3D sound over headphones using head related transfer functions
//!
//! Although at the moment only stereo output is supported, the *B-format* abstraction should make
//! it easy to implement arbitrary speaker configurations in the future.
//!
//! ## Usage Example
//!
//! ```
//! use std::thread::sleep;
//! use std::time::Duration;
//! use ambisonic::{rodio, AmbisonicBuilder};
//!
//! let scene = AmbisonicBuilder::default().build();
//!
//! let source = rodio::source::SineWave::new(440);
//! let mut sound = scene.play(source);
//! sound.set_position([50.0, 1.0, 0.0]);
//!
//! // move sound from right to left
//! sound.set_velocity([-10.0, 0.0, 0.0]);
//! for i in 0..1000 {
//!     sound.adjust_position([50.0 - i as f32 / 10.0, 1.0, 0.0]);
//!     sleep(Duration::from_millis(10));
//! }
//! sound.set_velocity([0.0, 0.0, 0.0]);
//! ```



pub use rodio;

mod bformat;
mod bmixer;
mod bstream;
mod renderer;
pub mod sources;

use std::f32;
use std::sync::Arc;

use crate::bmixer::BmixerComposer;
pub use crate::bstream::SoundController;
pub use crate::renderer::{HrtfConfig, StereoConfig};

/// Configure playback parameters
pub enum PlaybackConfiguration {
    /// Stereo playback
    Stereo(StereoConfig),

    /// Headphone playback using head related transfer functions
    Hrtf(HrtfConfig),
}

impl Default for PlaybackConfiguration {
    fn default() -> Self {
        PlaybackConfiguration::Stereo(StereoConfig::default())
    }
}

impl From<StereoConfig> for PlaybackConfiguration {
    fn from(cfg: StereoConfig) -> Self {
        PlaybackConfiguration::Stereo(cfg)
    }
}

impl From<HrtfConfig> for PlaybackConfiguration {
    fn from(cfg: HrtfConfig) -> Self {
        PlaybackConfiguration::Hrtf(cfg)
    }
}

/// A builder object for creating `Ambisonic` contexts
pub struct AmbisonicBuilder {
    device: Option<rodio::Device>,
    sample_rate: u32,
    config: PlaybackConfiguration,
}

impl AmbisonicBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the ambisonic context
    pub fn build(self) -> Ambisonic {
        let device = self.device
            .unwrap_or_else(|| rodio::default_output_device().unwrap());
        let sink = rodio::Sink::new(&device);

        let (mixer, controller) = bmixer::bmixer(self.sample_rate);

        match self.config {
            PlaybackConfiguration::Stereo(cfg) => {
                let output = renderer::BstreamStereoRenderer::new(mixer, cfg);
                sink.append(output);
            }

            PlaybackConfiguration::Hrtf(cfg) => {
                let output = renderer::BstreamHrtfRenderer::new(mixer, cfg);
                sink.append(output);
            }
        }

        Ambisonic {
            sink,
            composer: controller,
        }
    }

    /// Select device (defaults to `rodio::default_output_device()`
    pub fn with_device(self, device: rodio::Device) -> Self {
        AmbisonicBuilder {
            device: Some(device),
            ..self
        }
    }

    /// Set sample rate fo the ambisonic mix
    pub fn with_sample_rate(self, sample_rate: u32) -> Self {
        AmbisonicBuilder {
            sample_rate,
            ..self
        }
    }

    /// Set playback configuration
    pub fn with_config(self, config: PlaybackConfiguration) -> Self {
        AmbisonicBuilder { config, ..self }
    }
}

impl Default for AmbisonicBuilder {
    fn default() -> Self {
        AmbisonicBuilder {
            device: None,
            sample_rate: 48000,
            config: PlaybackConfiguration::default(),
        }
    }
}

/// High-level Ambisonic Context.
///
/// Stops playing all sounds when dropped.
pub struct Ambisonic {
    // disable warning that `sink` is unused. We need it to keep the audio alive.
    #[allow(dead_code)]
    sink: rodio::Sink,
    composer: Arc<BmixerComposer>,
}

impl Ambisonic {
    /// Add a single-channel `Source` to the sound scene at a position relative to the listener
    ///
    /// Returns a controller object that can be used to control the source during playback.
    #[inline(always)]
    pub fn play<I>(&self, input: I) -> SoundController
    where
        I: rodio::Source<Item = f32> + Send + 'static,
    {
        self.composer.play(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn it_works() {
        let engine = AmbisonicBuilder::new().build();

        let source = rodio::source::SineWave::new(440);
        let mut first = engine.play(source);

        first.set_position([1.0, 0.0, 0.0]);
        sleep(Duration::from_millis(1000));

        let source = rodio::source::SineWave::new(330);
        let mut second = engine.play(source);

        second.set_position([-1.0, 0.0, 0.0]);
        sleep(Duration::from_millis(1000));

        first.stop();
        second.set_position([0.0, 1.0, 0.0]);
        sleep(Duration::from_millis(1000));

        drop(engine);

        sleep(Duration::from_millis(1000));
    }

    #[test]
    fn move_sound() {
        let scene = AmbisonicBuilder::default().build();

        let source = rodio::source::SineWave::new(440);
        let mut sound = scene.play(source);
        sound.set_position([50.0, 1.0, 0.0]);

        // move sound from right to left
        sound.set_velocity([-10.0, 0.0, 0.0]);
        for i in 0..1000 {
            sound.adjust_position([50.0 - i as f32 / 10.0, 1.0, 0.0]);
            sleep(Duration::from_millis(10));
        }
        sound.set_velocity([0.0, 0.0, 0.0]);
    }

    #[test]
    fn bench() {
        use rodio::Source;

        let scene = AmbisonicBuilder::default().build();

        let mut f: u64 = 1;
        for _ in 0..850 {
            f = (f + f * f * 7 + f * f * f * 3 + 1) % 800;
            let source = rodio::source::SineWave::new(440).amplify(0.001);
            let _ = scene.play(source);
        }

        sleep(Duration::from_secs(10));
    }

    #[test]
    fn hrir() {
        let cfg = HrtfConfig::default();
        let scene = AmbisonicBuilder::default().with_config(cfg.into()).build();

        let source = sources::Noise::new(48000);

        let mut sound = scene.play(source);

        for i in 0..1000 {
            sound.adjust_position([(500 - i) as f32 / 10.0, 1.0, 0.0]);
            sleep(Duration::from_millis(10));
        }
    }
}
