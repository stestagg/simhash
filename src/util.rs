
pub trait SequentialToRange {
    fn sequential_to_range(self) -> impl Iterator<Item = (usize, usize)>;
}
struct SequentialToRangeIter<T> {
    iter: T,
    last: usize,
}
impl<T: Iterator<Item = usize>> SequentialToRange for T {
    fn sequential_to_range(self) -> impl Iterator<Item = (usize, usize)> {
        SequentialToRangeIter { iter: self, last: 0 }
    }
}
impl <T: Iterator<Item = usize>> Iterator for SequentialToRangeIter<T> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter.next() {
            let start = self.last;
            self.last = next;
            Some((start, next))
        } else {
            None
        }
    }
}

pub trait PairToU16Ext {
    fn pair_to_u16(self) -> impl Iterator<Item = u16>;
}
impl<T: Iterator<Item = (u8, u8)>> PairToU16Ext for T {
    fn pair_to_u16(self) -> impl Iterator<Item = u16> {
        self.map(|(a, b)| u16::from(b) << 8 | u16::from(a))
    }
}

pub fn window_range(len: usize, window_size: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..=len.saturating_sub(window_size)).map(move |i| (i, i + window_size))
}