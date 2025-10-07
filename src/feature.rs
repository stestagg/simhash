use pyo3::pyclass;
use unicode_segmentation::UnicodeSegmentation;


#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureType {
    Bytes, Graphemes, Chars, Words
}

pub trait Features {
    fn byte_features(&self) -> impl Iterator<Item = u8>;
    fn grapheme_features(&self) -> impl Iterator<Item = usize>;
    fn char_features(&self) -> impl Iterator<Item = usize>;
    fn word_features(&self) -> impl Iterator<Item = (usize, usize)>;
}

impl<T: AsRef<str>> Features for T {

    fn byte_features(&self) -> impl Iterator<Item = u8> {
        self.as_ref().as_bytes().iter().copied()
    }

    fn grapheme_features(&self) -> impl Iterator<Item = usize> {
        let grapheme_it = UnicodeSegmentation::grapheme_indices(self.as_ref(), true);
        grapheme_it.map(|(i, g)| i + g.len()) // return the end index of each grapheme
    }

    fn char_features(&self) -> impl Iterator<Item = usize> {
        self.as_ref().char_indices().map(|(i, c)| i + c.len_utf8())
    }

    fn word_features(&self) -> impl Iterator<Item = (usize, usize)> {
        self.as_ref().unicode_word_indices().map(|(i, w)| (i, i + w.len()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_features() {
        let s = "hello";
        let bytes: Vec<u8> = s.byte_features().collect();
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_grapheme_features() {
        let s = "a̐éö̲"; // a with combining
        let graphemes: Vec<usize> = s.grapheme_features().collect();
        assert_eq!(graphemes, vec![3, 6, 11]);
    }

    #[test]
    fn test_char_features() {
        let s = "hello";
        let chars: Vec<char> = s.char_features().map(|i| s[i - 1..i].chars().next().unwrap()).collect();
        assert_eq!(chars, vec!['h', 'e', 'l', 'l', 'o']);
    }

    #[test]
    fn test_word_features() {
        let s = "Hello, world! This is Rust.";
        let words: Vec<(usize, usize)> = s.word_features().collect();
        let extracted = words.iter().map(|&(start, end)| &s[start..end]).collect::<Vec<&str>>();
        assert_eq!(extracted, vec!["Hello", "world", "This", "is", "Rust"]);
    }
}