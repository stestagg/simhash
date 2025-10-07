use std::collections::VecDeque;


pub trait PairIterExt<T: Iterator> {
    fn sliding_pairs(self) -> PairsWindowIter<T>;
}

impl<T: Iterator> PairIterExt<T> for T
    where
        T::Item: Clone
{
    fn sliding_pairs(self) -> PairsWindowIter<T> {
        PairsWindowIter::new(self)
    }
}

pub struct PairsWindowIter<T: Iterator> {
    inp: T,
    last: Option<T::Item>,
}
impl <T: Iterator> PairsWindowIter<T>
where
    T::Item: Clone
{
    pub fn new(inp: T) -> Self {
        Self { inp, last: None }
    }
}
impl<T: Iterator> Iterator for PairsWindowIter<T>
where
    T::Item: Clone
{
    type Item = (T::Item, T::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.inp.next() {
            None => None,
            Some(next) => {
                match self.last.take() {
                    Some(last) => {
                        self.last = Some(next.clone());
                        Some((last, next))
                    },
                    None => {
                        self.last = Some(next);
                        self.next()
                    }
                }
            }
        }
    }
}

pub trait SlidingWindowIterExt<T: Iterator> {
    fn sliding_window(self, window_size: usize) -> SlidingWindowIter<T>;
}

impl <T: Iterator> SlidingWindowIterExt<T> for T {
    fn sliding_window(self, window_size: usize) -> SlidingWindowIter<T> {
        SlidingWindowIter::new(self, window_size)
    }
}

pub struct SlidingWindowIter<T: Iterator> {
    inp: T,
    window: VecDeque<T::Item>,
    window_size: usize,
}
impl <T: Iterator> SlidingWindowIter<T> {
    pub fn new(inp: T, window_size: usize) -> Self {
        Self { inp, window: VecDeque::with_capacity(window_size), window_size }
    }
}
impl<T: Iterator> Iterator for SlidingWindowIter<T>
where
    T::Item: Clone
{
    type Item = Vec<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.window.len() < self.window_size {
            match self.inp.next() {
                Some(next) => self.window.push_back(next),
                None => break,
            }
        }
        if self.window.len() == self.window_size {
            let result = self.window.iter().cloned().collect();
            self.window.pop_front();
            Some(result)
        } else {
            None
        }
    }
}

pub trait SequentialSlidingWindowIterExt<T: Iterator<Item = usize>> {
    fn sliding_sequential_window(self, window_size: usize) -> SequentialSlidingWindowIter<T>;
}

pub struct SequentialSlidingWindowIter<T: Iterator<Item = usize>> {
    inp: T,
    window: VecDeque<usize>,
    window_size: usize,
}
impl <T: Iterator<Item = usize>> SequentialSlidingWindowIterExt<T> for T {
    fn sliding_sequential_window(self, window_size: usize) -> SequentialSlidingWindowIter<T> {
        SequentialSlidingWindowIter::new(self, window_size)
    }
}
impl <T: Iterator<Item = usize>> SequentialSlidingWindowIter<T> {
    pub fn new(inp: T, window_size: usize) -> Self {
        let mut window = VecDeque::with_capacity(window_size);
        window.push_back(0);
        Self { inp, window, window_size }
    }
}
impl<T: Iterator<Item = usize>> Iterator for SequentialSlidingWindowIter<T> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.window.len() <= self.window_size {
            match self.inp.next() {
                Some(next) => self.window.push_back(next),
                None => break,
            }
        }
        if self.window.len() > self.window_size {
            let first = self.window.pop_front().unwrap();
            let result = (first, *self.window.back().unwrap());
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::feature::{Features};
    use crate::util::SequentialToRange;

    use super::*;

    #[test]
    fn test_sliding_pair() {
        let data = b"hello";
        let pairs = data.iter().cloned().sliding_pairs().collect::<Vec<(u8, u8)>>();
        assert_eq!(pairs, vec![
            (b'h', b'e'),
            (b'e', b'l'),
            (b'l', b'l'),
            (b'l', b'o'),
        ]);
    }

    #[test]
    fn test_sliding_window() {
        let data = b"hello";
        let windows = data.iter().cloned().sliding_window(3).collect::<Vec<Vec<u8>>>();
        assert_eq!(windows, vec![
            vec![b'h', b'e', b'l'],
            vec![b'e', b'l', b'l'],
            vec![b'l', b'l', b'o'],
        ]);
    }

    #[test]
    fn test_sliding_window_4() {
        let data = b"hello";
        let windows = data.iter().cloned().sliding_window(4).collect::<Vec<Vec<u8>>>();
        assert_eq!(windows, vec![
            vec![b'h', b'e', b'l', b'l'],
            vec![b'e', b'l', b'l', b'o'],
        ]);
    }

    #[test]
    fn test_sliding_window_5() {
        let data = b"hello";
        let windows = data.iter().cloned().sliding_window(5).collect::<Vec<Vec<u8>>>();
        assert_eq!(windows, vec![
            vec![b'h', b'e', b'l', b'l', b'o'],
        ]);
    }

    #[test]
    fn test_sliding_window_6() {
        let data = b"hello";
        let windows = data.iter().cloned().sliding_window(6).collect::<Vec<Vec<u8>>>();
        assert_eq!(windows.is_empty(), true);
    }

    #[test]
    fn test_graphemes() {
        let s = "a̐éö̲"; // a with combining
        let graphemes: Vec<(usize, usize)> = s.grapheme_features().sliding_pairs().collect();
        assert_eq!(graphemes, vec![(3, 6), (6, 11)]);
        let graphemes = s.grapheme_features().sequential_to_range().sliding_pairs().collect::<Vec<((usize, usize), (usize, usize))>>();
        assert_eq!(graphemes, vec![
            ((0, 3), (3, 6)),
            ((3, 6), (6, 11)),
        ]);
    }

    #[test]
    fn test_graphemes_sequential() {
        let s = "a̐éö̲"; // a with combining
        let graphemes: Vec<(usize, usize)> = s.grapheme_features().sliding_sequential_window(2).collect();

        assert_eq!(graphemes, vec![(0, 6), (3, 11)]);
    }

    #[test]
    fn test_words() {
        let s = "Hello, world! This is Rust.";
        let word_pairs = s.word_features().sliding_pairs();
        let extracted = word_pairs.map(|((start1, end1), (start2, end2))| (&s[start1..end1], &s[start2..end2])).collect::<Vec<(&str, &str)>>();
        assert_eq!(extracted, vec![
            ("Hello", "world"),
            ("world", "This"),
            ("This", "is"),
            ("is", "Rust"),
        ]);
    }
    #[test]
    fn test_words_sequential() {
        let s = "Hello, world! This is Rust.";
        let word_pairs = s.word_features().sliding_pairs().map(|(_, b)| b);
        let mut results = Vec::new();
        let mut index = 0;
        for (w1, w2) in word_pairs {
            let span = &s[index..w2];
            index = w1;
            results.push(span.to_string());
        }
        assert_eq!(results, vec![
            "Hello, world",
            "world! This",
            "This is",
            "is Rust",
        ]);
    }
}