use ambisonic::{sources, AmbisonicBuilder, HrtfConfig};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let cfg = HrtfConfig::default();
    let scene = AmbisonicBuilder::default().with_config(cfg.into()).build();

    let source = sources::Noise::new(48000);

    let mut sound = scene.play_at(source, [50.0, 1.0, 0.0]);

    for i in 0..1000 {
        sound.adjust_position([(500 - i) as f32 / 10.0, 1.0, 0.0]);
        sleep(Duration::from_millis(10));
    }
}
