use ambisonic::AmbisonicBuilder;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let scene = AmbisonicBuilder::new().build();

    let source = rodio::source::SineWave::new(440);
    let mut first = scene.play_omni(source);

    first.set_position([1.0, 0.0, 0.0]);
    sleep(Duration::from_millis(1000));

    let source = rodio::source::SineWave::new(330);
    let mut second = scene.play_omni(source);

    second.set_position([-1.0, 0.0, 0.0]);
    sleep(Duration::from_millis(1000));

    first.stop();
    sleep(Duration::from_millis(1000));

    second.set_position([0.0, 1.0, 0.0]);
    sleep(Duration::from_millis(1000));

    drop(scene);

    sleep(Duration::from_millis(1000));
}
