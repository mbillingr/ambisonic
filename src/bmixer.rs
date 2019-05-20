//! *B-format* mixer
//!
//! This module provides functionality for dynamically composing sound sources into a 3D sound
//! scene.

use crate::bformat::Bformat;
use crate::bstream::{self, Bstream, BstreamConfig, SoundController};
use rodio::{source::UniformSourceIterator, Sample, Source};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Construct a 3D sound mixer and associated sound composer.
pub fn bmixer(sample_rate: u32) -> (BstreamMixer, Arc<BmixerComposer>) {
    let controller = Arc::new(BmixerComposer {
        sample_rate,
        pending_streams: Mutex::new(Vec::new()),
        has_pending: AtomicBool::new(false),
    });

    let mixer = BstreamMixer {
        controller: controller.clone(),
        active_streams: Vec::with_capacity(8),
    };

    (mixer, controller)
}

/// Combine all currently playing 3D sound sources into a single *B-format* stream.
///
/// The mixer implements `rodio::Source<Item = Bformat>`, which must be passed to a renderer before
/// playback in a `rodio::Sink`.
pub struct BstreamMixer {
    controller: Arc<BmixerComposer>,
    active_streams: Vec<Bstream>,
}

impl Source for BstreamMixer {
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        1 // actually 4, but they are packed into one struct
    }

    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.controller.sample_rate
    }

    #[inline(always)]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for BstreamMixer {
    type Item = Bformat;

    fn next(&mut self) -> Option<Self::Item> {
        if self.controller.has_pending.load(Ordering::SeqCst) {
            let mut pending = self
                .controller
                .pending_streams
                .lock()
                .expect("Cannot lock pending streams");
            self.active_streams.extend(pending.drain(..));
            self.controller.has_pending.store(false, Ordering::SeqCst);
        }

        let mut mix = Bformat::zero_value();

        let mut done = Vec::new();

        for (i, stream) in self.active_streams.iter_mut().enumerate() {
            match stream.next() {
                Some(x) => mix = mix.saturating_add(x),
                None => done.push(i),
            }
        }

        for i in done.into_iter().rev() {
            self.active_streams.remove(i);
        }

        Some(mix)
    }
}

/// Compose the 3D sound scene
pub struct BmixerComposer {
    has_pending: AtomicBool,
    pending_streams: Mutex<Vec<Bstream>>,
    sample_rate: u32,
}

impl BmixerComposer {
    /// Add a single-channel `Source` to the sound scene at a position relative to the listener
    ///
    /// Returns a controller object that can be used to control the source during playback.
    pub fn play<I>(&self, input: I, config: BstreamConfig) -> SoundController
    where
        I: Source<Item = f32> + Send + 'static,
    {
        let (bstream, sound_ctl) = if input.sample_rate() == self.sample_rate {
            bstream::bstream(input, config)
        } else {
            let input = UniformSourceIterator::new(input, 1, self.sample_rate);
            bstream::bstream(input, config)
        };

        self.pending_streams
            .lock()
            .expect("Cannot lock pending streams")
            .push(bstream);
        self.has_pending.store(true, Ordering::SeqCst);

        sound_ctl
    }
}
