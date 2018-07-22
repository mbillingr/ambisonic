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
    input_buffer: VecDeque<Bformat>,
    left_coefs: Vec<Bweights>,
    right_coefs: Vec<Bweights>,
}

impl<I> BstreamHrtfRenderer<I>
where
    I: Source<Item = Bformat>,
{
    /// Construct a new HRTF renderer with default settings
    pub fn new(input: I, hrir_file: &str) -> Self {
        let (fs, left_coefs, right_coefs) = load_hrir(hrir_file);
        assert_eq!(fs as u32, input.sample_rate());
        assert_eq!(left_coefs.len(), right_coefs.len());

        let n = left_coefs.len();
        let input_buffer = VecDeque::from(vec![Bformat::zero_value(); n]);

        BstreamHrtfRenderer {
            input,
            buffered_output: None,
            input_buffer,
            left_coefs,
            right_coefs,
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

                self.input_buffer.pop_back();
                self.input_buffer.push_front(sample);

                let left: f32 = self.input_buffer.iter()
                    .zip(self.left_coefs.iter())
                    .map(|(sample, weight)| weight.dot(*sample))
                    .sum();

                let right: f32 = self.input_buffer.iter()
                    .zip(self.right_coefs.iter())
                    .map(|(sample, weight)| weight.dot(*sample))
                    .sum();

                // emit left channel now, and right channel next time
                self.buffered_output = Some(right);
                Some(left)
            }
        }
    }
}


fn load_hrir(filename: &str) -> (f32, Vec<Bweights>, Vec<Bweights>) {
    // todo: proper error handling
    let file = File::open(filename).unwrap();
    let mut bufr = BufReader::new(file);
    let mut data = String::new();
    bufr.read_to_string(&mut data).unwrap();

    let mut lines = data.split('\n');
    let fs: f32 = lines.next().unwrap().parse().unwrap();
    assert_eq!(lines.next(), Some(""));

    let mut lr = [Vec::new(), Vec::new()];

    for side in &mut lr {
        loop {
            let weights: Vec<_> = match lines.next() {
                None => panic!("unexpected end of file"),
                Some("") => break,
                Some(l) => l.split(", ").map(|s| s.parse().unwrap()).collect()
            };

            assert_eq!(weights.len(), 4);

            let (w, x, y, z) = (weights[0], weights[1], weights[2], weights[3]);
            let bw = Bweights::new(w, x, y, z);

            side.push(bw);
        }
    }

    (fs, lr[0].clone(), lr[1].clone())
}
