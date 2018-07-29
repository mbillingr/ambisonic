use std::collections::VecDeque;
use std::iter::Sum;
use std::ops::{Add, Mul};

use rodio::Sample;

/// A simple delay
#[derive(Clone)]
pub struct Delay<X>
{
    buffer: VecDeque<X>,
    last_output: X,
}

impl<X: Sample> Delay<X>
{
    /// Construct new filter from weights
    pub fn new(n: usize) -> Self {
        Delay {
            buffer: vec![X::zero_value(); n].into(),
            last_output: X::zero_value(),
        }
    }
}

impl<X: Copy> Delay<X>
{
    /// Push new sample into the filter and return filtered output
    pub fn push(&mut self, x: X) -> X {
        self.last_output = self.buffer.pop_back().unwrap();
        self.buffer.push_front(x);
        self.last_output
    }

    /// Retrieve final samples if there is no more input
    pub fn push_empty(&mut self) -> Option<X> {
        self.buffer.pop_back()
    }

    pub fn last(&self) -> X {
        self.last_output
    }
}

/// An all-pass filter with delay element
#[derive(Clone)]
pub struct AllPass<W, X>
{
    buffer: VecDeque<X>,
    last_output: X,
    forward_coefficient: W,
    backward_coefficient: W,
}

impl<W, X: Sample> AllPass<W, X>
{
    /// Construct new filter from weights
    pub fn new(n: usize, f: W, b: W) -> Self {
        AllPass {
            buffer: vec![X::zero_value(); n].into(),
            last_output: X::zero_value(),
            forward_coefficient: f,
            backward_coefficient: b,
        }
    }
}

impl<W, X: Copy> AllPass<W, X>
where
    X: Copy + Mul<W, Output=X> + Add<X, Output=X>,
    W: Copy
{
    /// Push new sample into the filter and return filtered output
    pub fn push(&mut self, x: X) -> X {
        let d =  self.buffer.pop_back().unwrap();
        let c = x + d * self.backward_coefficient;
        self.last_output  = c * self.forward_coefficient + d;
        self.buffer.push_front(c);
        self.last_output
    }

    /// Retrieve final samples if there is no more input
    pub fn push_empty(&mut self) -> Option<X> {
        let d =  self.buffer.pop_back().unwrap();
        let c = d * self.backward_coefficient;
        self.last_output  = c * self.forward_coefficient + d;
        self.buffer.push_front(c);
        Some(self.last_output)
    }

    pub fn last(&self) -> X {
        self.last_output
    }
}

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
        FirFilter {
            convolution_buffer: vec![X::zero_value(); weights.len()].into(),
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
