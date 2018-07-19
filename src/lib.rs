extern crate cpal;
pub extern crate rodio;

mod bformat;
mod bstream;
mod bmixer;
mod renderer;

use std::sync::Arc;

use bmixer::BmixerController;

pub struct AmbisonicBuilder {
    device: Option<rodio::Device>,
    sample_rate: u32,
}

impl AmbisonicBuilder {
    pub fn new() -> Self {
        AmbisonicBuilder {
            device: None,
            sample_rate: 44100,
        }
    }

    pub fn with_device(self, device: rodio::Device) -> Self {
        AmbisonicBuilder {
            device: Some(device),
            ..self
        }
    }

    pub fn with_sample_rate(self, sample_rate: u32) -> Self {
        AmbisonicBuilder {
            sample_rate,
            ..self
        }
    }

    pub fn build(self) -> Engine {
        let device = self.device
            .unwrap_or_else(|| rodio::default_output_device().unwrap());
        let sink = rodio::Sink::new(&device);

        let (mixer, controller) = bmixer::bmixer(self.sample_rate);
        let output = renderer::BstreamStereoRenderer::new(mixer);

        sink.append(output);

        Engine {
            sink,
            controller,
        }
    }
}

pub struct Engine {
    sink: rodio::Sink,
    controller: Arc<BmixerController>,
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
        engine.controller.play(source, [1.0, 0.0, 0.0]);

        sleep(Duration::from_millis(1000));

        let source = rodio::source::SineWave::new(330);
        engine.controller.play(source, [-1.0, 0.0, 0.0]);

        sleep(Duration::from_millis(2000));
    }
}
