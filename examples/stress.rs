/**
Playing many sources simultaneously will lead to stuttering if compiled in Debug mode
but hopefully not in Release mode.
*/

use ambisonic::AmbisonicBuilder;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    use rodio::Source;
    let scene = AmbisonicBuilder::default().build();

    for _ in 0..500 {
        let source = rodio::source::SineWave::new(440).amplify(0.001);
        let _ = scene.play_omni(source);
    }

    sleep(Duration::from_secs(10));
}
