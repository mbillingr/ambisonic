use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{Sample, Source};

use bformat::{Bformat, Bweights};
use bstream::{self, Bstream, BstreamController};

pub fn bmixer(sample_rate: u32) -> (BstreamMixer, Arc<BmixerController>) {
    let controller = Arc::new(BmixerController {
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

pub struct BstreamMixer {
    controller: Arc<BmixerController>,
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
            let mut pending = self.controller
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

pub struct BmixerController {
    has_pending: AtomicBool,
    pending_streams: Mutex<Vec<Bstream>>,
    sample_rate: u32,
}

impl BmixerController {
    pub fn play<I>(&self, input: I, pos: [f32; 3]) -> Arc<BstreamController>
    where
        I: Source<Item = f32> + Send + 'static,
    {
        assert_eq!(input.channels(), 1);

        let (bstream, sound_ctl) = bstream::bstream(input, pos);

        self.pending_streams
            .lock()
            .expect("Cannot lock pending streams")
            .push(bstream);
        self.has_pending.store(true, Ordering::SeqCst);

        sound_ctl
    }
}
