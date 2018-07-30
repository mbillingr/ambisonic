
use std::time::Duration;

use rodio::{Sample, Source};

use bformat::{Bformat, Bweights};
use filter::{AllPass, Delay, FirFilter};


pub struct Reverb<I>
where
    I: Iterator
{
    input: I,
    delay: Delay<I::Item>,
    lowpass: FirFilter<Bweights, Bformat>,
    aps: Vec<AllPass<Bweights, Bformat>>,
}

impl<I> Reverb<I>
where
    I: Source,
    I::Item: Sample,
{
    /// Construct a new stereo renderer with default settings
    pub fn new(input: I) -> Self {
        Reverb {
            input,
            delay: Delay::new(5000),
            lowpass: FirFilter::new(vec![Bweights::new(0.1, 0.1, 0.1, 0.1); 10]),
            aps: vec![AllPass::new(10, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(12, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(110, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(130, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(140, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(500, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(700, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(800, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(1500, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(1000, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5)),
                      AllPass::new(30000, Bweights::new(-0.5, -0.5, -0.5, -0.5), Bweights::new(0.5, 0.5, 0.5, 0.5))],
        }
    }
}

impl<I> Source for Reverb<I>
    where
        I: Source<Item = Bformat>,
{
    #[inline(always)]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline(always)]
    fn channels(&self) -> u16 {
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

impl<I> Iterator for Reverb<I>
    where
        I: Source<Item = Bformat>,
{
    type Item = Bformat;

    fn next(&mut self) -> Option<Self::Item> {
        match self.input.next() {
            Some(mut x) => {
                /*let a = self.delay.last().amplify(0.8);
                let b = self.lowpass.push(a);
                x = x.saturating_add(b);
                self.delay.push(x);*/

                let mut a = self.aps.last().unwrap().last();
                a = a + x;
                for ap in &mut self.aps {
                    a = ap.push(a).amplify(0.9);
                }

                Some(a + x)
            },
            None => self.delay.push_empty(),
        }
    }
}
