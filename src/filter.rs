use std::collections::VecDeque;
use std::iter::Sum;
use std::ops::Mul;

use rodio::Sample;

/// Finite impulse response filter
pub struct FirFilter<W, X>
{
    convolution_buffer: VecDeque<X>,
    weights: Vec<W>,
}

impl<W, X> FirFilter<W, X>
where
    X: Sample + Mul<W> + Copy,
    X::Output: Sum,
    W: Copy,
{
    /// Construct new filter from weights
    pub fn new(weights: Vec<W>) -> Self {
        let convolution_buffer = vec![X::zero_value(); weights.len()].into();
        FirFilter {
            convolution_buffer,
            weights,
        }
    }
}

impl<W, X> FirFilter<W, X>
    where
        X: Mul<W> + Copy,
        X::Output: Sum,
        W: Copy,
{
    /// Push new sample into the filter and return filtered output
    pub fn push(&mut self, x: X) -> X::Output {
        self.convolution_buffer.pop_back();
        self.convolution_buffer.push_front(x);

        self.convolution_buffer.iter()
            .rev()
            .zip(self.weights.iter().rev())
            .map(|(x, w)| *x * *w)
            .sum()
    }

    /// Retrieve final samples if there is no more input
    pub fn push_empty(&mut self) -> Option<X::Output> {
        self.convolution_buffer.pop_back()?;

        let y = self.convolution_buffer.iter()
            .rev()
            .zip(self.weights.iter().rev())
            .map(|(x, w)| *x * *w)
            .sum();

        Some(y)
    }
}

pub struct FirFilterIterator<I, W>
    where
        I: Iterator,
{
    input: I,
    filter: FirFilter<W, I::Item>
}

impl<I, W> FirFilterIterator<I, W>
    where
        I: Iterator,
        I::Item: Sample + Mul<W>,
        <I::Item as Mul<W>>::Output: Sum,
        W: Copy,
{
    pub fn new(input: I, weights: Vec<W>) -> Self {
        FirFilterIterator {
            input,
            filter: FirFilter::new(weights),
        }
    }
}


impl<I, W> Iterator for FirFilterIterator<I, W>
    where
        I: Iterator,
        I::Item: Mul<W> + Copy,
        <I::Item as Mul<W>>::Output: Sum,
        W: Copy,
{
    type Item = <I::Item as Mul<W>>::Output;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self.input.next() {
            Some(x) => Some(self.filter.push(x)),
            None => self.filter.push_empty(),
        }
    }
}
