//! Represent audio sources in *B-format*.

use crate::bformat::{Bformat, Bweights};
use crate::constants::SPEED_OF_SOUND;
use rodio::Source;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Convert a `rodio::Source` to a spatial `Bstream` source with associated controller
///
/// The input source must produce `f32` samples and is expected to have exactly one channel.
pub fn bstream<I: Source<Item = f32> + Send + 'static>(
    source: I,
    config: BstreamConfig,
) -> (Bstream, SoundController) {
    assert_eq!(source.channels(), 1);

    let bridge = Arc::new(BstreamBridge {
        commands: Mutex::new(Vec::new()),
        pending_commands: AtomicBool::new(false),
        stopped: AtomicBool::new(false),
    });

    let (position, weights) = match config.position {
        Some(p) => (p, Bweights::from_position(p)),
        None => ([0.0, 0.0, 0.0], Bweights::omni_source()),
    };

    let controller = SoundController {
        bridge: bridge.clone(),
        position,
        velocity: config.velocity,
        doppler_factor: config.doppler_factor,
        speed_of_sound: config.speed_of_sound,
    };

    let stream = Bstream {
        input: Box::new(source),
        bweights: weights,
        target_weights: weights,
        speed: compute_doppler_rate(
            position,
            config.velocity,
            config.doppler_factor,
            config.speed_of_sound,
        ),
        sampling_offset: 0.0,
        previous_sample: 0.0,
        next_sample: 0.0,
        bridge,
    };

    (stream, controller)
}

/// Initial configuration for constructing `Bstream`s
pub struct BstreamConfig {
    position: Option<[f32; 3]>,
    velocity: [f32; 3],
    doppler_factor: f32,
    speed_of_sound: f32,
}

impl Default for BstreamConfig {
    fn default() -> Self {
        BstreamConfig {
            position: None,
            velocity: [0.0, 0.0, 0.0],
            doppler_factor: 1.0,
            speed_of_sound: SPEED_OF_SOUND,
        }
    }
}

impl BstreamConfig {
    /// Create new `BstreamConfig` with default settings.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set initial position relative to listener.
    pub fn with_position(mut self, p: [f32; 3]) -> Self {
        self.position = Some(p);
        self
    }

    /// Set initial velocity.
    pub fn with_velocity(mut self, v: [f32; 3]) -> Self {
        self.velocity = v;
        self
    }

    /// Set doppler factor for this stream.
    pub fn with_doppler_factor(mut self, d: f32) -> Self {
        self.doppler_factor = d;
        self
    }

    /// Set speed of sound for this stream.
    pub fn with_speed_of_sound(mut self, s: f32) -> Self {
        self.speed_of_sound = s;
        self
    }
}

/// Spatial source
///
/// Consumes samples from the inner source and converts them to *B-format* samples.
pub struct Bstream {
    input: Box<dyn Source<Item = f32> + Send>,
    bridge: Arc<BstreamBridge>,

    bweights: Bweights,
    target_weights: Bweights,

    speed: f32,
    sampling_offset: f32,
    previous_sample: f32,
    next_sample: f32,
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
                    Command::SetTarget(bw) => self.target_weights = bw,
                    Command::SetSpeed(s) => self.speed = s,
                    Command::Stop => {
                        self.bridge.stopped.store(true, Ordering::SeqCst);
                        return None;
                    }
                }
            }

            self.bridge.pending_commands.store(false, Ordering::SeqCst);
        }

        // adjusting the weights slowly avoids audio artifacts but prevents very fast position
        // changes
        self.bweights.approach(&self.target_weights, 0.001);

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

#[derive(Debug)]
enum Command {
    SetWeights(Bweights),
    SetTarget(Bweights),
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
    ///
    /// Abruptly changing the position of a sound source may cause
    /// popping artifacts. Use this function only to set the source's
    /// initial position, and dynamically adjust the position with
    /// `adjust_position`.
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
        let weights = Bweights::from_position(pos);
        let rate = self.doppler_rate();
        {
            let mut cmds = self.bridge.commands.lock().unwrap();
            cmds.push(Command::SetSpeed(rate));
            cmds.push(Command::SetWeights(weights));
            cmds.push(Command::SetTarget(weights));
        }
        self.bridge.pending_commands.store(true, Ordering::SeqCst);
    }
    /// Adjust source position relative to listener
    ///
    /// The source transitions smoothly to the new position.
    /// Use this function to dynamically change the position of a
    /// sound source while it is playing.
    pub fn adjust_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
        let weights = Bweights::from_position(pos);
        let rate = self.doppler_rate();
        {
            let mut cmds = self.bridge.commands.lock().unwrap();
            cmds.push(Command::SetSpeed(rate));
            cmds.push(Command::SetTarget(weights));
        }
        self.bridge.pending_commands.store(true, Ordering::SeqCst);
    }

    /// Set source velocity relative to listener
    ///
    /// The velocity determines how much doppler effect to apply
    /// but has no effect on the source's position. Use
    /// `adjust_position` to update the source's position.
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
        compute_doppler_rate(
            self.position,
            self.velocity,
            self.doppler_factor,
            self.speed_of_sound,
        )
    }
}

/// compute doppler rate
fn compute_doppler_rate(
    position: [f32; 3],
    velocity: [f32; 3],
    doppler_factor: f32,
    speed_of_sound: f32,
) -> f32 {
    let dist =
        (position[0] * position[0] + position[1] * position[1] + position[2] * position[2]).sqrt();

    let relative_velocity;

    if dist.abs() < EPS {
        relative_velocity =
            (velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2])
                .sqrt();
    } else {
        relative_velocity =
            (position[0] * velocity[0] + position[1] * velocity[1] + position[2] * velocity[2])
                / dist;
    }

    speed_of_sound / (speed_of_sound + doppler_factor * relative_velocity)
}

const EPS: f32 = 1e-6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_doppler_effect_if_velocity_is_zero() {
        let position = [0.0, 1.0, 0.0];
        let velocity = [0.0, 0.0, 0.0];

        let rate = compute_doppler_rate(position, velocity, 1.0, 1.0);

        assert_eq!(rate, 1.0);
    }

    #[test]
    fn doppler_effect_depends_on_velocity_if_position_is_zero() {
        let position = [0.0, 0.0, 0.0];
        let velocity = [1.0, 1.0, 1.0];

        let rate = compute_doppler_rate(position, velocity, 1.0, 1.0);

        assert_eq!(rate, 1.0 / (1.0 + f32::sqrt(3.0)));
    }
}
