//! Represent audio sources in *B-format*.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::Source;

use bformat::{Bformat, Bweights};

/// Convert a `rodio::Source` to a spatial `Bstream` source with associated controller
///
/// The input source must produce `f32` samples and is expected to have exactly one channel.
pub fn bstream<I: Source<Item = f32> + Send + 'static>(
    source: I,
    pos: [f32; 3],
) -> (Bstream, Arc<BstreamController>) {
    assert_eq!(source.channels(), 1);

    let controller = Arc::new(BstreamController {
        commands: Mutex::new(Vec::new()),
        pending_commands: AtomicBool::new(false),
        stopped: AtomicBool::new(false),
    });

    let stream = Bstream {
        input: Box::new(source),
        bweights: Bweights::from_position(pos),
        controller: controller.clone(),
    };

    (stream, controller)
}

/// Spatial source
///
/// Consumes samples from the inner source and converts them to *B-format* samples.
pub struct Bstream {
    input: Box<Source<Item = f32> + Send>,
    bweights: Bweights,
    controller: Arc<BstreamController>,
}

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
        if self.controller.pending_commands.load(Ordering::SeqCst) {
            let mut commands = self.controller.commands.lock().unwrap();
            let mut new_pos = None;

            for cmd in commands.drain(..) {
                match cmd {
                    Command::SetPos(p) => new_pos = Some(p),
                    Command::Stop => {
                        self.controller.stopped.store(true, Ordering::SeqCst);
                        return None;
                    }
                }
            }

            if let Some(pos) = new_pos {
                self.bweights = Bweights::from_position(pos);
            }

            self.controller
                .pending_commands
                .store(false, Ordering::SeqCst);
        }
        match self.input.next() {
            Some(x) => Some(self.bweights.scale(x)),
            None => {
                self.controller.stopped.store(true, Ordering::SeqCst);
                None
            }
        }
    }
}

enum Command {
    SetPos([f32; 3]),
    Stop,
}

/// Controls playback and position of spatial audio source
pub struct BstreamController {
    commands: Mutex<Vec<Command>>,
    pending_commands: AtomicBool,
    stopped: AtomicBool,
}

impl BstreamController {
    /// Set source position
    pub fn set_position(&self, pos: [f32; 3]) {
        self.commands.lock().unwrap().push(Command::SetPos(pos));
        self.pending_commands.store(true, Ordering::SeqCst);
    }

    /// Stop playback
    pub fn stop(&self) {
        self.commands.lock().unwrap().push(Command::Stop);
        self.pending_commands.store(true, Ordering::SeqCst);
    }
}
