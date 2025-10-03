use pyo3::prelude::*;

pub trait IntoU64 {
    fn into_u64(self) -> u64;
}

impl IntoU64 for u64 {
    fn into_u64(self) -> u64 {
        self
    }
}

fn hamming_distance<T: IntoU64, U: IntoU64>(a: T, b: U) -> u32 {
    (a.into_u64() ^ b.into_u64()).count_ones()
}

mod hash;
mod tree;
mod features;

use std::borrow::Cow;
type AnyFeatureIter<'a> = dyn Iterator<Item = Cow<'a, [u8]>> + 'a;

pub struct SimHasher {
    hash_fn: Box<dyn Fn(&[u8]) -> u64 + Send + Sync>,
    feature_fn: Box<dyn for<'a> Fn(&'a [u8]) -> Box<AnyFeatureIter<'a>> + Send + Sync>,
}

impl SimHasher {
    pub fn new<H, F>(hash_fn: H, feature_fn: F) -> Self
    where
        H: Fn(&[u8]) -> u64 + Send + Sync + 'static,
        F: for<'a> Fn(&'a [u8]) -> Box<AnyFeatureIter<'a>> + Send + Sync + 'static,
    {
        Self {
            hash_fn: Box::new(hash_fn),
            feature_fn: Box::new(feature_fn),
        }
    }

    pub fn hash<T: SimHashable>(&self, value: T) -> u64 {
        value.simhash(self)
    }

    pub fn hash_bytes(&self, bytes: &[u8]) -> u64 {
        let hash_fn = &self.hash_fn;
        let feature_fn = &self.feature_fn;
        crate::hash::hash_features(
            bytes,
            |feature| (hash_fn)(feature),
            Box::new(move |input| (feature_fn)(input)),
        )
    }
}

pub trait SimHashable: Sized {
    fn simhash(self, hasher: &SimHasher) -> u64;
}

impl<T> SimHashable for T
where
    T: AsRef<[u8]> + Sized,
{
    fn simhash(self, hasher: &SimHasher) -> u64 {
        hasher.hash_bytes(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::{SimHashable, SimHasher};

    #[test]
    fn simhasher_hashes_bytes() {
        let hasher = SimHasher::new(crate::hash::xxh3_::hash_fn, |bytes: &[u8]| {
            Box::new(bytes.windows(3).map(|window| window.into()))
        });

        let left = hasher.hash("hello world");
        let right = hasher.hash("hello world");

        assert_ne!(left, 0);
        assert_eq!(left, right);
    }

    #[test]
    fn simhashable_impl_for_as_ref() {
        let hasher = SimHasher::new(crate::hash::sip_::hash_fn, |bytes: &[u8]| {
            Box::new(bytes.iter().map(|byte| std::slice::from_ref(byte).into()))
        });

        let text = String::from("example");
        let hash_from_hasher = hasher.hash(text.as_bytes());
        let hash_from_trait = text.simhash(&hasher);

        assert_eq!(hash_from_hasher, hash_from_trait);
    }
}

#[pymodule]
mod simhash {
    use std::collections::HashMap;

    use pyo3::prelude::*;
    use crate::tree;
    use crate::hash::{sip_, xxh3_};
    use crate::AnyFeatureIter;

    fn to_bytes(obj: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
        if let Ok(s) = obj.extract::<&str>() {
            Ok(s.as_bytes().to_vec())
        } else if let Ok(b) = obj.extract::<&[u8]>() {
            Ok(b.to_vec())
        } else if let Ok(b) = obj.downcast::<pyo3::types::PyByteArray>() {
            Ok(unsafe { b.as_bytes().to_vec() })
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "Input must be str or bytes, got {}",
                    obj.get_type()
                        .name()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "a value with unknown type".to_string())
                )
            ))
        }
    }

    #[pyclass]
    #[derive(Clone)]
    struct SimHash{
        #[pyo3(get, set)]
        value: u64
    }

    #[pymethods]
    impl SimHash {

        #[staticmethod]
        fn from_int(val: u64) -> Self {
            SimHash { value: val }
        }

        fn __str__(&self) -> String {
            format!("0x{:016x}", self.value)
        }
        fn __repr__(&self) -> String {
            format!("<SimHash 0x{:016x}>", self.value)
        }
        fn __hash__(&self) -> u64 {
            self.value
        }

        fn __eq__(&self, other: &SimHash) -> bool {
            self.value == other.value
        }
        fn __ne__(&self, other: &SimHash) -> bool {
            self.value != other.value
        }

        // These added just to allow containers to work nicely
        fn __lt__(&self, other: &SimHash) -> bool {
            self.value < other.value
        }
        fn __le__(&self, other: &SimHash) -> bool {
            self.value <= other.value
        }
        fn __gt__(&self, other: &SimHash) -> bool {
            self.value > other.value
        }
        fn __ge__(&self, other: &SimHash) -> bool {
            self.value >= other.value
        }

        fn __int__(&self) -> u64 {
            self.value
        }

        fn hamming_distance(&self, other: &SimHash) -> u32 {
            super::hamming_distance(self.value, other.value)
        }

        fn difference(&self, other: &SimHash) -> u32 {
            super::hamming_distance(self.value, other.value)
        }

    }

    #[pyclass(eq, eq_int)]
    #[derive(PartialEq, Copy, Clone, Debug)]
    enum HashMethod {
        SipHash,
        XXHash,
    }

    #[pyclass]
    #[derive(PartialEq, Copy, Clone, Debug)]
    enum Features {
        Bytes(usize),
        Chars(usize),
        Graphemes(usize),
        // Words(usize),
    }

    impl Features {

        fn feature_fn<'a>(&self) -> Result<Box<dyn FnOnce(&'a [u8]) -> Box<AnyFeatureIter<'a>> + '_>, &'static str> {
            match self {
                Features::Bytes(1) => return Ok(Box::new(|v: &'a [u8]| Box::new(v.iter().map(|b| std::slice::from_ref(b).into())))),
                Features::Bytes(n) => {
                    if n == &0 {
                        return Err("Feature size must be greater than 0");
                    }
                    return Ok(Box::new(move |v: &'a [u8]| Box::new(v.windows(*n as usize).map(|w| w.into()))));
                },
                Features::Chars(n) => {
                    if n == &0 {
                        return Err("Feature size must be greater than 0");
                    }
                    return Ok(Box::new(move |v: &'a [u8]| Box::new(crate::features::chars(*n, v))));
                },
                Features::Graphemes(n) => {
                    if n == &0 {
                        return Err("Feature size must be greater than 0");
                    }
                    return Ok(Box::new(move |v: &'a [u8]| Box::new(crate::features::graphemes(*n, v))));
                },
                // Features::Words(n) => {
                //     if n == &0 {
                //         return Err("Feature size must be greater than 0");
                //     }
                //     return Ok(Box::new(move |v: &'a [u8]| Box::new(crate::features::words(*n, v))));
                // },
            }
        }

    }

    #[pyclass]
    struct SimDict {
        tree: tree::HashTree<PyObject>,
        identity_map: std::collections::HashMap<u64, PyObject>,
        hash_method: HashMethod,
        features: Features,
        max_diff: usize,
    }

    #[pymethods]
    impl SimDict {
        // It's like a dict, keys are hashes of the key string/bytes
        // Identity map stores all values
        // Tree used for approximate matching, but items not added to tree if already approximately present

        #[new]
        #[pyo3(signature = (method=HashMethod::SipHash, features=Features::Bytes(2), max_diff=3))]
        fn new(method: HashMethod, features: Features, max_diff: usize) -> Self {
            Self {
                tree: tree::HashTree::new(),
                identity_map: std::collections::HashMap::new(),
                hash_method: method,
                features,
                max_diff,
            }
        }

        fn __len__(&self) -> usize {
            self.identity_map.len()
        }

        fn __contains__(&self, key: Bound<'_, PyAny>) -> PyResult<bool> {
            let hash = maybe_hash(key.as_ref(), self.hash_method, self.features)?;
            if self.identity_map.contains_key(&hash) {
                return Ok(true);
            }
            if self.tree.contains(hash, self.max_diff).is_some() {
                return Ok(true);
            }
            return Ok(false);
        }

        fn __setitem__(&mut self, key: Bound<'_, PyAny>, value: PyObject) -> PyResult<()> {
            self.set(key, value).map(|_| ())
        }

        fn set(&mut self, key: Bound<'_, PyAny>, value: PyObject) -> PyResult<PyObject> {
            let hash = maybe_hash(key.as_ref(), self.hash_method, self.features)?;
            let py = key.py();

            let entry = self.identity_map.entry(hash);

            match entry {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    return Ok(o.get().clone_ref(py))
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    let tree_val = self.tree.contains(hash, self.max_diff);
                    match tree_val {
                        Some(store_v) => {
                            v.insert(store_v.clone_ref(py));
                            return Ok(store_v.clone_ref(py));
                        },
                        None => {
                            v.insert(value.clone_ref(py));
                            self.tree.add(hash, value.clone_ref(py));
                            return Ok(value.clone_ref(py));
                        }
                    }
                }
            };
        }

        fn __getitem__(&self, key: Bound<'_, PyAny>) -> PyResult<PyObject> {
            let hash = maybe_hash(key.as_ref(), self.hash_method, self.features)?;
            let py = key.py();
            if let Some(v) = self.identity_map.get(&hash) {
                return Ok(v.clone_ref(py));
            }
            if let Some(v) = self.tree.contains(hash, self.max_diff) {
                return Ok(v.clone_ref(py));
            }
            Err(pyo3::exceptions::PyKeyError::new_err("Key not found"))
        }

        fn get(&self, key: Bound<'_, PyAny>) -> PyResult<Option<PyObject>> {
            let hash = maybe_hash(key.as_ref(), self.hash_method, self.features)?;
            let py = key.py();
            if let Some(v) = self.identity_map.get(&hash) {
                return Ok(Some(v.clone_ref(py)));
            }
            if let Some(v) = self.tree.contains(hash, self.max_diff) {
                return Ok(Some(v.clone_ref(py)));
            }
            Ok(None)
        }

        fn items(&self, py: Python) -> Vec<(u64, PyObject)> {
            self.identity_map.iter().map(|(k, v)| (*k, v.clone_ref(py))).collect()
        }
        
    }

    fn maybe_hash(value: &Bound<'_, PyAny>, method: HashMethod, features: Features) -> PyResult<u64> {
        if let Ok(simhash) = (&value).extract::<SimHash>() {
            return Ok(simhash.value);
        }
        let bytes = to_bytes(value)?;
        hash(bytes, method.clone(), features.clone()).map(|s| s.value)
    }

    #[pyfunction]
    #[pyo3(signature = (val, method=HashMethod::SipHash, features=Features::Bytes(2) ))]
    fn hash(#[pyo3(from_py_with=to_bytes)] val: Vec<u8>, method: HashMethod, features: Features) -> PyResult<SimHash> {

        let hash_value = match (method, features) {
            (HashMethod::SipHash, Features::Bytes(2)) => {
                super::hash::hash_byte_pair(&val, sip_::lookup_u16)
            }
            (HashMethod::XXHash, Features::Bytes(2)) => {
                super::hash::hash_byte_pair(&val, xxh3_::lookup_u16)
            }
            (HashMethod::SipHash, f) => {
                let feature_fn = f.feature_fn().map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
                super::hash::hash_features(&val, sip_::hash_fn, feature_fn)
            }
            (HashMethod::XXHash, f) => {
                let feature_fn = f.feature_fn().map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
                super::hash::hash_features(&val, xxh3_::hash_fn, feature_fn)
            }
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Unsupported combination of hash method and feature extraction",
                ))
            }
        };

        Ok(SimHash { value: hash_value })
    }

    #[pyfunction]
    fn features(#[pyo3(from_py_with=to_bytes)] val: Vec<u8>, features: Features) -> PyResult<Vec<Vec<u8>>> {
        let feature_fn = features.feature_fn().map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
        Ok(feature_fn(&val).map(|c| c.into_owned()).collect())
    }

    // #[pyfunction]
    // fn group_texts(py: Python, texts: Vec<PyObject>, method: HashMethod, features: Features, max_diff: usize) -> PyResult<Vec<Vec<PyObject>>> {
    //     let dict = SimDict::new(method, features, max_diff);
    //     let mut groups: HashMap<usize, Vec<PyObject>> = HashMap::new();

    //     for text in texts.iter() {
    //         let group_val = dict.set(
    //         let group_key = group_val.extract::<usize>(py)?;
    //         groups.entry(group_key).or_insert_with(Vec::new).push(text.clone_ref(py));
    //     }

    //     Ok(groups.into_values().collect())
    // }
    
}