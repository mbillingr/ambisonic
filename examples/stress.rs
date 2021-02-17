use ambisonic::AmbisonicBuilder;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    use rodio::Source;
    let scene = AmbisonicBuilder::default().build();

    let mut f: u64 = 1;
    for _ in 0..1 {
        f = (f + f * f * 7 + f * f * f * 3 + 1) % 800;
        let source = rodio::source::SineWave::new(440); //.amplify(0.001);
        let _ = scene.play_omni(source);
    }

    sleep(Duration::from_secs(10));
}
