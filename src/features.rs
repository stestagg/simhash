use unicode_segmentation::{UnicodeSegmentation, UnicodeSentences};
use std::borrow::Cow;
use std::collections::VecDeque;

struct WindowIndexIter<'a, T: Iterator<Item = usize> + 'a> {
    value: &'a [u8],
    slice_iter: T,
    window_size: usize,
    buf: VecDeque<usize>,
    input_done: bool,
}

impl<'a, T: Iterator<Item = usize> + 'a> WindowIndexIter<'a, T> {
    fn new(value: &'a [u8], slice_iter: T, window_size: usize) -> Self {
        let mut initial_buf = VecDeque::with_capacity(window_size);
        initial_buf.push_back(0); // Always start with index 0
        Self {
            value,
            slice_iter,
            window_size,
            buf: initial_buf,
            input_done: false,
        }
    }
}

impl<'a, T: Iterator<Item = usize> + 'a> Iterator for WindowIndexIter<'a, T> {
    type Item = Cow<'a, [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        // Input iterator provides the indices of the last byte of each slice

        // If input is done, we should have already returned the last window slice
        if self.input_done { return None; }

        // Try to fill the buffer to the window size
        while self.buf.len() <= self.window_size {
            match self.slice_iter.next() {
                Some(idx) => self.buf.push_back(idx),
                None => {
                    // If we get here:
                    // 1. before the first window is full, then we will never return anything (that's ok)
                    // 2. after the first window is full, then we've already returned the last window slice last time, nothing to do
                    // 3. We can't get here if the buffer is exactly window_size, because that was the other match arm..
                    self.input_done = true;
                    return None;
                }
            }
        }
        let start_idx = self.buf.pop_front().unwrap_or(0);
        let end_idx = *self.buf.back().unwrap();
        Some(Cow::Borrowed(&self.value[start_idx..end_idx]))
    }
}

pub fn graphemes<'a>(n_chars: usize, value: &'a [u8]) -> impl Iterator<Item = Cow<'a, [u8]>> + 'a{
    let s = std::str::from_utf8(value).unwrap_or("");
    let grapheme_it = UnicodeSegmentation::grapheme_indices(s, true);
    WindowIndexIter::new(value, grapheme_it.filter_map(|(idx, v)| match v.len() { 0 => None, _ => Some(idx + v.len()) }), n_chars)
}

pub fn chars<'a>(n_chars: usize, value: &'a [u8]) -> impl Iterator<Item = Cow<'a, [u8]>> + 'a {
    let s = std::str::from_utf8(value).unwrap_or("");
    let char_it = s.char_indices();
    WindowIndexIter::new(value, char_it.map(|(idx, v)| idx + v.len_utf8()), n_chars)
}