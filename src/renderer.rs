//! Render *B-format* audio streams to streams suitable for playback on audio equipment.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

use rodio::{Sample, Source};

use bformat::{Bformat, Bweights};

/// Render a *B-format* stream to a stereo representation.
///
/// Suitable for playback over two speakers arranged in front of the user.
/// The default setting assumes a symmetric arrangement of +/- 45º.
pub struct BstreamStereoRenderer<I> {
    input: I,
    buffered_sample: Option<f32>,
    left_mic: Bweights,
    right_mic: Bweights,
}

impl<I> BstreamStereoRenderer<I> {
    /// Construct a new stereo renderer with default settings
    pub fn new(input: I) -> Self {
        BstreamStereoRenderer {
            input,
            buffered_sample: None,
            left_mic: Bweights::virtual_microphone([-1.0, 1.0, 0.0], 0.5),
            right_mic: Bweights::virtual_microphone([1.0, 1.0, 0.0], 0.5),
        }
    }
}

impl<I> Source for BstreamStereoRenderer<I>
where
    I: Source<Item = Bformat>,
{
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        2 // well, it's stereo...
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

impl<I> Iterator for BstreamStereoRenderer<I>
where
    I: Source<Item = Bformat>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffered_sample.take() {
            Some(s) => Some(s),
            None => {
                let sample = self.input.next()?;

                let left = self.left_mic.dot(sample);
                let right = self.right_mic.dot(sample);

                // emit left channel now, and right channel next time
                self.buffered_sample = Some(right);
                Some(left)
            }
        }
    }
}

/// Render a *B-format* stream for headphones using head related transfer functions.
pub struct BstreamHrtfRenderer<I> {
    input: I,
    buffered_output: Option<f32>,
    convolution_buffers: Vec<VecDeque<f32>>,
    virtual_speakers: Vec<VirtualSpeaker>,
}

impl<I> BstreamHrtfRenderer<I>
where
    I: Source<Item = Bformat>,
{
    /// Construct a new HRTF renderer with default settings
    pub fn new(input: I, hrir_file: &str) -> Self {
        let (fs, virtual_speakers) = load_hrir(hrir_file);
        assert_eq!(fs as u32, input.sample_rate());

        let convolution_buffers = virtual_speakers
            .iter()
            .map(|speaker| {
                let n = speaker.left_hrir.len().max(speaker.right_hrir.len());
                VecDeque::from(vec![0.0; n])
            })
            .collect();

        BstreamHrtfRenderer {
            input,
            buffered_output: None,
            convolution_buffers,
            virtual_speakers,
        }
    }
}

impl<I> Source for BstreamHrtfRenderer<I>
where
    I: Source<Item = Bformat>,
{
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
        2 // well, it's stereo...
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

impl<I> Iterator for BstreamHrtfRenderer<I>
where
    I: Source<Item = Bformat>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffered_output.take() {
            Some(s) => Some(s),
            None => {
                let sample = self.input.next()?;

                let (mut left, mut right) = (0.0, 0.0);

                for (speaker, buffer) in self.virtual_speakers
                    .iter()
                    .zip(self.convolution_buffers.iter_mut())
                {
                    let signal = speaker.bweights.dot(sample);
                    buffer.push_front(signal);

                    left += buffer
                        .iter()
                        .zip(&speaker.left_hrir)
                        .map(|(s, h)| s * h)
                        .sum::<f32>();

                    right += buffer
                        .iter()
                        .zip(&speaker.right_hrir)
                        .map(|(s, h)| s * h)
                        .sum::<f32>();
                }

                // emit left channel now, and right channel next time
                self.buffered_output = Some(right);
                Some(left)
            }
        }
    }
}

struct VirtualSpeaker {
    bweights: Bweights,
    left_hrir: Vec<f32>,
    right_hrir: Vec<f32>,
}

fn load_hrir(filename: &str) -> (f32, Vec<VirtualSpeaker>) {
    // todo: proper error handling
    let file = File::open(filename).unwrap();
    let mut bufr = BufReader::new(file);
    let mut data = String::new();
    bufr.read_to_string(&mut data).unwrap();

    let mut lines = data.split('\n');
    let fs: f32 = lines.next().unwrap().parse().unwrap();
    assert_eq!(lines.next(), Some(""));

    let mut virtual_speakers = Vec::new();

    loop {
        let bweights: Bweights = match lines.next() {
            None | Some("") => break,
            Some(l) => l.split(", ").map(|s| s.parse().unwrap()).collect(),
        };

        let left_hrir: Vec<f32> = match lines.next() {
            None | Some("") => panic!("expected hrir"),
            Some(l) => l.split(", ").map(|s| s.parse().unwrap()).collect(),
        };

        let right_hrir: Vec<f32> = match lines.next() {
            None | Some("") => panic!("expected hrir"),
            Some(l) => l.split(", ").map(|s| s.parse().unwrap()).collect(),
        };

        assert_eq!(lines.next(), Some(""));

        virtual_speakers.push(VirtualSpeaker {
            bweights,
            left_hrir,
            right_hrir,
        });
    }

    assert!(lines.next().is_none());

    (fs, virtual_speakers)
}
