use pyo3::prelude::*;

mod hamming;
mod util;
mod hash;
mod feature;
mod window;
mod simhasher;
mod tree;
mod map;

pub use simhasher::SimHasher;
pub use feature::FeatureType;
pub use hash::HashMethod;


#[pymodule]
mod simhash {
    use std::borrow::Cow;

    use pyo3::prelude::*;
    use pyo3::types::{PyList, PyString};

    #[pymodule_export]
    use crate::feature::FeatureType;

    #[pymodule_export]
    use crate::hash::HashMethod;

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
            super::hamming::hamming_distance(self.value, other.value)
        }

        fn difference(&self, other: &SimHash) -> u32 {
            super::hamming::hamming_distance(self.value, other.value)
        }

    }


    #[pyclass]
    struct SimHasher {
        hasher: crate::simhasher::SimHasher
    }
    #[pymethods]
    impl SimHasher {
        #[new]
        #[pyo3(signature = (hash_method=HashMethod::XXHash, features=FeatureType::Bytes, n=2 ))]
        fn new(hash_method: HashMethod, features: FeatureType, n: usize) -> PyResult<Self> {
            let hasher = crate::simhasher::SimHasher::new(hash_method, features, n) 
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;


            Ok(SimHasher {
                hasher
            })
        }

        fn hash(&self, input: &str) -> SimHash {
            let hash_value = self.hasher.hash(input);
            SimHash { value: hash_value }
        }

        fn features(&self, py: Python, input: &str) -> PyResult<Vec<Py<PyAny>>> {
            let features = (self.hasher.feature_extractor)(input);
            features.into_iter().map(|f| f.clone_into_py(py)).collect::<Result<Vec<_>, _>>()
        }

        fn group_texts(&self, py: Python, texts: Bound<PyList>, max_diff: usize) -> PyResult<Vec<Vec<Py<PyAny>>>> {
            let mut dict = crate::map::SimMap::new(
                self.hasher.clone(),
                max_diff as u8
            );
            let mut groups: std::collections::HashMap<usize, Vec<Py<PyAny>>> = std::collections::HashMap::new();

            for text in texts.iter() {
                let text_val = text.extract::<String>()?;

                let group_val = dict.maybe_insert_close_or(text_val, || groups.len());
                groups.entry(*group_val).or_default().push(text.into());
            }

            Ok(groups.into_values().collect())
        }
    }

    #[pyfunction]
    #[pyo3(signature = (value, method=HashMethod::XXHash, features=FeatureType::Bytes, n=2 ))]
    fn hash(value: &str, method: HashMethod, features: FeatureType, n: usize) -> PyResult<SimHash> {
        let hasher = crate::simhasher::SimHasher::new(method, features, n) 
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let hash_value = hasher.hash(value);
        Ok(SimHash { value: hash_value })
    }

    #[pyfunction]
    #[pyo3(signature = (value, features=FeatureType::Bytes))]
    fn features(py: Python, value: &str, features: FeatureType) -> PyResult<Vec<Py<PyAny>>> {
        let hasher = crate::simhasher::SimHasher::new(HashMethod::SipHash, features, 1)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let features = (hasher.feature_extractor)(value);
        features.into_iter().map(|f| f.clone_into_py(py)).collect::<Result<Vec<_>, _>>()
    }

    #[pyfunction]
    #[pyo3(signature = (texts, max_diff=3, method=HashMethod::XXHash, features=FeatureType::Bytes, n=2 ))]
    fn group_texts(py: Python, texts: Bound<PyList>, max_diff: usize, method: HashMethod, features: FeatureType, n: usize) -> PyResult<Vec<Vec<Py<PyAny>>>> {
        let hasher = SimHasher::new(method, features, n)?;
        hasher.group_texts(py, texts, max_diff)
    }


}
