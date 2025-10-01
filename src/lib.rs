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

mod hash_byte_pair;
mod tree;


#[pyclass]
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
        hamming_distance(self.value, other.value)
    }

    fn difference(&self, other: &SimHash) -> u32 {
        self.hamming_distance(other)
    }

}

#[pyclass]
struct SimDict {
    tree: tree::HashTree<PyObject>,
    identity_map: std::collections::HashMap<u64, PyObject>,
}

#[pymethods]
impl SimDict {
    #[new]
    fn new() -> Self {
        Self {
            tree: tree::HashTree::new(),
            identity_map: std::collections::HashMap::new(),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
mod simhash {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::SimHash;

    #[pymodule_export]
    use super::SimDict;

    #[pyfunction]
    fn hash_sip_2byte(val: &str) -> SimHash {
        SimHash { value: super::hash_byte_pair::hash_pair(val, super::hash_byte_pair::sip_lookup_byte_pair) }
    }

    #[pyfunction]
    fn hash_xxh3_2byte(val: &str) -> SimHash {
        SimHash { value: super::hash_byte_pair::hash_pair(val, super::hash_byte_pair::xxh3_lookup_byte_pair) }
    }
    
}