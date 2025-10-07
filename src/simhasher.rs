use std::fmt::Display;

use pyo3::{IntoPyObject, Py, PyAny, PyErr, PyObject, PyResult};

use crate::{
    feature::{FeatureType, Features},
    hash::{HashMethod, ShHash},
    hash_dispatch,
    util::{PairToU16Ext, SequentialToRange, window_range},
    window::{PairIterExt, SlidingWindowIterExt},
};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct InvalidWindowSize(&'static str);

impl Display for InvalidWindowSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid window size: {}", self.0)
    }
}


#[derive(Debug)]
pub enum Err {
    InvalidWindowSize(InvalidWindowSize)
}

impl Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Err::InvalidWindowSize(e) => write!(f, "{}", e),
        }
    }
}

impl From<InvalidWindowSize> for Err {
    fn from(e: InvalidWindowSize) -> Self {
        Err::InvalidWindowSize(e)
    }
}

fn make_simhasher(
    features: FeatureType,
    window_size: usize,
    hash_method: HashMethod,
) -> Result<Box<dyn Fn(&str) -> u64 + Send + Sync>, Err> {
    if window_size == 0 {
        return Err(InvalidWindowSize("Window size must be greater than 0").into());
    }
    hash_dispatch!(hash_method, {
        match features {
            FeatureType::Bytes => match window_size {
                1 => {
                    return Ok(Box::new(move |s: &str| {
                        let hashes = <hasher_type!()>::hashing_items_u8(s.byte_features());
                        simhash_impl(hashes)
                    }));
                }
                2 => {
                    return Ok(Box::new(move |s: &str| {
                        let vals = s.byte_features().sliding_pairs().pair_to_u16();
                        let hashes = <hasher_type!()>::hashing_items_u16(vals);
                        simhash_impl(hashes)
                    }));
                }
                n => {
                    return Ok(Box::new(move |s: &str| {
                        let hashes = <hasher_type!()>::hashing_items_range(window_range(s.len(), n), s);
                        simhash_impl(hashes)
                    }));
                }
            },
            FeatureType::Chars => match window_size {
                1 => {
                    return Ok(Box::new(move |s: &str| {
                        let char_indices = s.char_features().sequential_to_range();
                        let hashes = <hasher_type!()>::hashing_items_range(char_indices, s);
                        simhash_impl(hashes)
                    }));
                }
                n => {
                    return Ok(Box::new(move |s: &str| {
                        let windows = s.char_features().sequential_to_range().sliding_window(n);
                        let hashes = <hasher_type!()>::hashing_windows(windows, s);
                        simhash_impl(hashes)
                    }));
                }
            },
            FeatureType::Graphemes => match window_size {
                1 => {
                    return Ok(Box::new(move |s: &str| {
                        let grapheme_indices = s.grapheme_features().sequential_to_range();
                        let hashes = <hasher_type!()>::hashing_items_range(grapheme_indices, s);
                        simhash_impl(hashes)
                    }));
                }
                n => {
                    return Ok(Box::new(move |s: &str| {
                        let windows = s.grapheme_features().sequential_to_range().sliding_window(n);
                        let hashes = <hasher_type!()>::hashing_windows(windows, s);
                        simhash_impl(hashes)
                    }));
                }
            },
            FeatureType::Words => match window_size {
                1 => {
                    return Ok(Box::new(move |s: &str| {
                        let word_indices = s.word_features();
                        let hashes = <hasher_type!()>::hashing_items_range(word_indices, s);
                        simhash_impl(hashes)
                    }));
                }
                n => {
                    return Ok(Box::new(move |s: &str| {
                        let windows = s.word_features().sliding_window(n);
                        let hashes = <hasher_type!()>::hashing_windows(windows, s);
                        simhash_impl(hashes)
                    }));
                }
            },

        }
    })
}

pub trait AnyFeature{
    fn clone_into_py(&self, py: pyo3::Python) -> PyResult<Py<PyAny>>;
}

impl AnyFeature for u8 {

    fn clone_into_py(&self, py: pyo3::Python) -> PyResult<Py<PyAny>> {
        let val = vec![*self];
        Ok(val.into_pyobject(py).map(Py::from)?)
    }

}

impl AnyFeature for String {

    fn clone_into_py(&self, py: pyo3::Python) -> PyResult<Py<PyAny>> {
        Ok(self.to_owned().into_pyobject(py).map(Py::from)?)
    }

}

// impl<T> AnyFeature for T
// where
//     T: ToOwned + for<'a> pyo3::IntoPyObject<'a>,
//     T::Owned: for<'a> IntoPyObject<'a>,
//     Py<PyAny>: for<'a> From<<T::Owned as IntoPyObject<'a>>::Output>, 
//     PyErr: for<'a> From<<T::Owned as IntoPyObject<'a>>::Error>
// {

//     fn clone_into_py(&self, py: pyo3::Python) -> PyResult<Py<PyAny>> {
//         Ok(self.to_owned().into_pyobject(py).map(Py::from)?)
//     }
// }

fn valid_utf8_ranges_to_feature_vec(ranges: impl Iterator<Item = (usize, usize)>, s: &str) -> Vec<Box<dyn AnyFeature>> {
    let bytes = s.as_bytes();
    ranges.map(|(start, end)| {
        let slice = &bytes[start..end];
        let feature = std::str::from_utf8(slice).unwrap().to_string();
        Box::new(feature) as Box<dyn AnyFeature>
    }).collect()
}

pub fn make_feature_extractor<'a>(features: FeatureType) -> Box<dyn for<'b> Fn(&'b str) -> Vec<Box<dyn AnyFeature>> + Send + Sync> {
    match features {
        FeatureType::Bytes => Box::new(|s: &str| s.byte_features().map(|f| Box::new(f) as Box<dyn AnyFeature>).collect::<Vec<_>>()),
        FeatureType::Chars => Box::new(|s: &str| valid_utf8_ranges_to_feature_vec(s.char_features().sequential_to_range(), s)),
        FeatureType::Graphemes => Box::new(|s: &str| valid_utf8_ranges_to_feature_vec(s.grapheme_features().sequential_to_range(), s)),
        FeatureType::Words => Box::new(|s: &str| valid_utf8_ranges_to_feature_vec(s.word_features(), s)),
    }
}

pub fn simhash_impl(hashes: impl Iterator<Item = u64>) -> u64 {
    let mut feature_adj: u32 = 0;
    let mut buckets = [0u32; 64];

    for hash in hashes {
        feature_adj += 1;
        for i in 0..64 {
            buckets[i] = buckets[i].saturating_add((hash >> i & 1) as u32);
        }
    }

    feature_adj /= 2;

    let val = buckets.iter().enumerate().fold(0, |acc, (i, &b)| {
        let bitval = (if b > feature_adj { 1 } else { 0 }) << i;
        acc | bitval
    });
    val
}

pub struct SimHasher {
    hash_method: HashMethod,
    feature_type: FeatureType,
    window_size: usize,
    maker: Box<dyn Fn(&str) -> u64 + Send + Sync>,
    pub feature_extractor: Box<dyn Fn(&str) -> Vec<Box<dyn AnyFeature>> + Send + Sync>,
}

impl SimHasher {
    pub fn new(
        hash_method: HashMethod,
        features: FeatureType,
        window_size: usize,
    ) -> Result<Self, Err> {
        let maker = make_simhasher(features, window_size, hash_method)?;
        let feature_extractor = make_feature_extractor(features);
        Ok(Self {
            hash_method,
            feature_type: features,
            window_size,
            maker,
            feature_extractor,
        })
    }

    pub fn hash<T: AsRef<str>>(&self, text: T) -> u64 {
        (self.maker)(text.as_ref())
    }
}

impl Clone for SimHasher {
    fn clone(&self) -> Self {
        SimHasher::new(self.hash_method, self.feature_type, self.window_size).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    fn test_simhasher() {
        let sh = SimHasher::new(HashMethod::SipHash, FeatureType::Bytes, 1).unwrap();
        let v1 = sh.hash("hello");
        let v2 = sh.hash("world");
        assert_ne!(v1, v2);
        assert_eq!(v1, sh.hash("hello"));
        assert_eq!(v1, 3880401949562285464);
    }

    #[test]
    fn compare_approaches() {
        let sh = SimHasher::new(HashMethod::SipHash, FeatureType::Bytes, 1).unwrap();
        let val = "h";
        let v1 = sh.hash(val);

        let h2 = crate::hash::sip_::Hasher::hashing_items_range(window_range(val.len(), 1), val);
        let v2 = simhash_impl(h2);
        assert_eq!(v1, v2);
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    fn compare_bytes_chars(#[case] n: usize) {
        let val = "Hello world!";
        let sh_bytes = SimHasher::new(HashMethod::SipHash, FeatureType::Bytes, n).unwrap();
        let sh_chars = SimHasher::new(HashMethod::SipHash, FeatureType::Chars, n).unwrap();
        let sh_graph = SimHasher::new(HashMethod::SipHash, FeatureType::Graphemes, n).unwrap();

        let v_bytes = sh_bytes.hash(val);
        let v_chars = sh_chars.hash(val);
        let v_graph = sh_graph.hash(val);
        assert_eq!(v_bytes, v_chars);
        assert_eq!(v_bytes, v_graph);
    }
}
