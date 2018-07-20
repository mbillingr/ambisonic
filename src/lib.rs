//! Ambisonic is a library for playing 3D audio.
//!
//! The library is built around the concept of a intermediate representation of the sound field,
//! called the *B-format*. The *B-format* describes what the listener should hear, independent of
//! their audio playback equipment. This leads to a clear separation of audio scene composition and
//! rendering. For details, see https://en.wikipedia.org/wiki/Ambisonics.
//!
//! In its current state, the library allows spatial composition of single-channel `rodio` sources
//! into a first-order *B-format* stream, and rendering the *B-format* stream to a two-channel
//! stereo signal. The result can be played through a `rodio` sink.

extern crate cpal;
pub extern crate rodio;

mod bformat;
mod bmixer;
mod bstream;
mod renderer;

use std::sync::Arc;

use bmixer::BmixerComposer;
use bstream::BstreamController;

/// A builder object for creating `Ambisonic` contexts
pub struct AmbisonicBuilder {
    device: Option<rodio::Device>,
    sample_rate: u32,
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
        let output = renderer::BstreamStereoRenderer::new(mixer);

        sink.append(output);

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
}

impl Default for AmbisonicBuilder {
    fn default() -> Self {
        AmbisonicBuilder {
            device: None,
            sample_rate: 44100,
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
    /// Returns a controller object that can be used to change the source position during playback.
    #[inline(always)]
    pub fn play<I>(&self, input: I, pos: [f32; 3]) -> Arc<BstreamController>
    where
        I: rodio::Source<Item = f32> + Send + 'static,
    {
        self.composer.play(input, pos)
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
        let first = engine.play(source, [1.0, 0.0, 0.0]);

        sleep(Duration::from_millis(1000));

        let source = rodio::source::SineWave::new(330);
        let second = engine.play(source, [-1.0, 0.0, 0.0]);

        sleep(Duration::from_millis(1000));

        first.stop();
        second.set_position([0.0, 1.0, 0.0]);

        sleep(Duration::from_millis(1000));

        drop(engine);

        sleep(Duration::from_millis(1000));
    }
}
