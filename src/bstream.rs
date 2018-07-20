//! Represent audio sources in *B-format*.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::Source;

use bformat::{Bformat, Bweights};

/// Convert a `rodio::Source` to a spatial `Bstream` source with associated controller
///
/// The input source must produce `f32` samples and is expected to have exactly one channel.
pub fn bstream<I: Source<Item = f32> + Send + 'static>(source: I) -> (Bstream, SoundController) {
    assert_eq!(source.channels(), 1);

    let bridge = Arc::new(BstreamBridge {
        commands: Mutex::new(Vec::new()),
        pending_commands: AtomicBool::new(false),
        stopped: AtomicBool::new(false),
    });

    let controller = SoundController {
        bridge: bridge.clone(),
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        doppler_factor: 1.0,
        speed_of_sound: 343.5, // m/s in air
    };

    let stream = Bstream {
        input: Box::new(source),
        bweights: Bweights::omni_source(),
        speed: 1.0,
        sampling_offset: 0.0,
        previous_sample: 0.0,
        next_sample: 0.0,
        bridge: bridge,
    };

    (stream, controller)
}

/// Spatial source
///
/// Consumes samples from the inner source and converts them to *B-format* samples.
pub struct Bstream {
    input: Box<Source<Item = f32> + Send>,
    bweights: Bweights,
    speed: f32,
    sampling_offset: f32,
    previous_sample: f32,
    next_sample: f32,
    bridge: Arc<BstreamBridge>,
}

impl Bstream {}

impl Source for Bstream {
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        assert_eq!(self.input.channels(), 1);
        1
    }

    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline(always)]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}

impl Iterator for Bstream {
    type Item = Bformat;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bridge.pending_commands.load(Ordering::SeqCst) {
            let mut commands = self.bridge.commands.lock().unwrap();

            for cmd in commands.drain(..) {
                match cmd {
                    Command::SetWeights(bw) => self.bweights = bw,
                    Command::SetSpeed(s) => self.speed = s,
                    Command::Stop => {
                        self.bridge.stopped.store(true, Ordering::SeqCst);
                        return None;
                    }
                }
            }

            self.bridge.pending_commands.store(false, Ordering::SeqCst);
        }

        while self.sampling_offset >= 1.0 {
            match self.input.next() {
                Some(x) => {
                    self.previous_sample = self.next_sample;
                    self.next_sample = x;
                }
                None => {
                    self.bridge.stopped.store(true, Ordering::SeqCst);
                    return None;
                }
            };
            self.sampling_offset -= 1.0;
        }

        let x = self.next_sample * self.sampling_offset
            + self.previous_sample * (1.0 - self.sampling_offset);

        self.sampling_offset += self.speed;
        Some(self.bweights.scale(x))
    }
}

enum Command {
    SetWeights(Bweights),
    SetSpeed(f32),
    Stop,
}

/// Bridges a Bstream and its controller across threads
pub struct BstreamBridge {
    commands: Mutex<Vec<Command>>,
    pending_commands: AtomicBool,
    stopped: AtomicBool,
}

/// Controls playback and position of a spatial audio source
pub struct SoundController {
    bridge: Arc<BstreamBridge>,
    position: [f32; 3],
    velocity: [f32; 3],
    doppler_factor: f32,
    speed_of_sound: f32,
}

impl SoundController {
    /// Set source position relative to listener
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
        let weights = Bweights::from_position(pos);
        let rate = self.doppler_rate();
        {
            let mut cmds = self.bridge.commands.lock().unwrap();
            cmds.push(Command::SetSpeed(rate));
            cmds.push(Command::SetWeights(weights));
        }
        self.bridge.pending_commands.store(true, Ordering::SeqCst);
    }

    /// Set source velocity relative to listener
    pub fn set_velocity(&mut self, vel: [f32; 3]) {
        self.velocity = vel;
        let rate = self.doppler_rate();
        {
            let mut cmds = self.bridge.commands.lock().unwrap();
            cmds.push(Command::SetSpeed(rate));
        }
        self.bridge.pending_commands.store(true, Ordering::SeqCst);
    }

    /// Stop playback
    pub fn stop(&self) {
        self.bridge.commands.lock().unwrap().push(Command::Stop);
        self.bridge.pending_commands.store(true, Ordering::SeqCst);
    }

    /// Set doppler factor
    pub fn set_doppler_factor(&mut self, factor: f32) {
        self.doppler_factor = factor;
    }

    /// compute doppler rate
    fn doppler_rate(&self) -> f32 {
        let dist = (self.position[0] * self.position[0]
            + self.position[1] * self.position[1]
            + self.position[1] * self.position[2])
            .sqrt();

        let relative_velocity = (self.position[0] * self.velocity[0]
            + self.position[1] * self.velocity[1]
            + self.position[1] * self.velocity[2]) / dist;

        self.speed_of_sound / (self.speed_of_sound + self.doppler_factor * relative_velocity)
    }
}
