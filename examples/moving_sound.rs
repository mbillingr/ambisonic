use ambisonic::AmbisonicBuilder;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let scene = AmbisonicBuilder::default().build();

    let source = rodio::source::SineWave::new(440);
    let mut sound = scene.play_at(source, [50.0, 1.0, 0.0]);

    // move sound from right to left
    sound.set_velocity([-10.0, 0.0, 0.0]);
    for i in 0..1000 {
        sound.adjust_position([50.0 - i as f32 / 10.0, 1.0, 0.0]);
        sleep(Duration::from_millis(10));
    }
    sound.set_velocity([0.0, 0.0, 0.0]);
}
