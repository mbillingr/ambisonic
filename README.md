# ambisonic

## Compose and play 3D audio.
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](
https://github.com/mbillingr/ambisonic#license)
[![Cargo](https://img.shields.io/crates/v/ambisonic.svg)](
https://crates.io/crates/ambisonic)
[![Documentation](https://docs.rs/ambisonic/badge.svg)](
https://docs.rs/ambisonic/latest/ambisonic/)

![Tests](https://github.com/mbillingr/ambisonic/workflows/Tests/badge.svg)

The ambisonic library provides 3D sound scene support on top of [`rodio`](https://crates.io/crates/rodio).
It allows positioning and moving sound sources freely in 3D space around a virtual listener,
and playing the resulting spatial mix in real-time over a sound card.

### Features
- Realistic directional audio
- Take `rodio` sound sources and place them in space
- Doppler effect on moving sounds

### Gallery
- [Video](https://www.youtube.com/watch?v=LrLn5t2zEp4) that demonstrates spatial audio in a 3D graphics scene by [@bjadamson](https://github.com/bjadamson)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ambisonic = "0.4"
```

## Usage Example
Ambisonic can generate a sine wave for you:

```rust
use std::thread::sleep;
use std::time::Duration;
use ambisonic::{rodio, AmbisonicBuilder};

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
```

Since Ambisonic is built ontop of Rodio, any [file format](https://github.com/RustAudio/rodio#audio-playback-library) that Rodio supports can also be loaded and positioned in 3D space.

```rust
let file = std::fs::File::open("path/to/your/file.wav").unwrap();
let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
let source = source.repeat_infinite();

let mut sound = scene.play_at(source.convert_samples(), [50.0, 1.0, 0.0]);

// move sound from right to left
sound.set_velocity([-10.0, 0.0, 0.0]);
for i in 0..1000 {
    sound.adjust_position([50.0 - i as f32 / 10.0, 1.0, 0.0]);
    sleep(Duration::from_millis(10));
}
sound.set_velocity([0.0, 0.0, 0.0]);
```

### Technical Details

`ambisonic` is built around the concept of an intermediate representation of the sound field,
called *B-format*. The *B-format* describes what the listener should hear, independent of
their audio playback equipment. This leads to a clear separation of audio scene composition and
rendering. For details, see [Wikipedia](https://en.wikipedia.org/wiki/Ambisonics).

In its current state, the library allows spatial composition of single-channel `rodio` sources
into a first-order *B-format* stream. The chosen renderer then decodes the *B-format* stream
into audio signals for playback.

Currently, the following renderers are available:

- Stereo: simple and efficient playback on two stereo speakers or headphones
- HRTF: realistic 3D sound over headphones using head related transfer functions

Although at the moment only stereo output is supported, the *B-format* abstraction should make
it easy to implement arbitrary speaker configurations in the future.

## Learning Resources

https://github.com/mbillingr/ambisonic/blob/master/documents/info.md

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
