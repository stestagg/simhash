use siphasher::sip::SipHasher;
use std::hash::Hasher;
use pyo3::pyclass;


#[pyclass(eq, eq_int)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum HashMethod {
    SipHash,
    XXHash,
}

#[macro_export]
macro_rules! hash_dispatch {
    ($method:expr, $body:tt) => {
        match $method {
            HashMethod::SipHash => {
                macro_rules! hasher_type { () => { crate::hash::sip_::Hasher }; }
                $body
            }
            HashMethod::XXHash => {
                macro_rules! hasher_type { () => { crate::hash::xxh3_::Hasher }; }
                $body
            }
        }
    };
}

pub trait ShHash {
    fn hash_u8(value: u8) -> u64;
    fn hash_u16(value: u16) -> u64;
    fn hash_bytes(bytes: &[u8]) -> u64;
    fn hash_multi<'a>(source: &'a[u8], slices: Vec<(usize, usize)>) -> u64;

    fn hashing_items_u8<'a>(values: impl Iterator<Item=u8> + 'a) -> impl Iterator<Item=u64> + 'a {
        values.map(|v| { Self::hash_u8(v) })
    }
    fn hashing_items_u16(values: impl Iterator<Item=u16>) -> impl Iterator<Item=u64> {
        values.map(|v| Self::hash_u16(v))
    }
    fn hashing_items_range<'a>(ranges: impl Iterator<Item=(usize, usize)> + 'a, source: &'a str) -> impl Iterator<Item=u64> + 'a {
        let bytes = source.as_bytes();
        ranges.map(move |(start, end)| {
            Self::hash_bytes(&bytes[start..end])
        })
    }
    fn hashing_windows<'a>(ranges: impl Iterator<Item=Vec<(usize, usize)>> + 'a, source: &'a str) -> impl Iterator<Item=u64> + 'a {
        let bytes = source.as_bytes();
        ranges.map(move |positions| {
            Self::hash_multi(bytes, positions)
        })
    }
}

pub fn sip_hash_fn<'a, U: AsRef<[u8]> + 'a + ?Sized, T: Iterator<Item=&'a U>>(vals: T) -> u64 {
    let mut hasher = SipHasher::new();
    for val in vals {
        hasher.write(val.as_ref());
    }
    hasher.finish()
}

pub fn xxh3_hash_fn<'a, U: AsRef<[u8]> + 'a + ?Sized, T: Iterator<Item=&'a U>>(vals: T) -> u64 {
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
    for val in vals {
        hasher.update(val.as_ref());
    }
    hasher.digest()
}


macro_rules! hash_impl {
    ($name:ident, $hash_fn:path) => {
        pub mod $name {
            use lazy_static::lazy_static;
            use super::ShHash;

            lazy_static! {
                static ref U16_TABLE: [u64; 65535] = {
                    let mut dest = [0; 65535];
                    (0..65535).for_each(|value: u16| {
                        let bytes = value.to_le_bytes();
                        dest[value as usize] = $hash_fn(std::iter::once(&bytes));
                    });
                    dest
                };
                static ref U8_TABLE: [u64; 255] = {
                    let mut dest = [0; 255];
                    (0..255).for_each(|value: u8| {
                        dest[value as usize] = $hash_fn(std::iter::once(&[value]));
                    });
                    dest
                };
            }

            pub struct Hasher;
            impl ShHash for Hasher {
                fn hash_u8(value: u8) -> u64 {
                    U8_TABLE[value as usize]
                }
                fn hash_u16(value: u16) -> u64 {
                    U16_TABLE[value as usize]
                }
                fn hash_bytes(bytes: &[u8]) -> u64 {
                    $hash_fn(std::iter::once(&bytes))
                }
                fn hash_multi(source: &[u8], slices: Vec<(usize, usize)>) -> u64 {
                    let vals = slices.iter().map(|(start, end)| &source[*start..*end]);
                    $hash_fn(vals)
                }
            }

        }
    }
}

hash_impl!(sip_, super::sip_hash_fn);
hash_impl!(xxh3_, super::xxh3_hash_fn);


#[cfg(test)]
mod tests {
    use crate::{util::PairToU16Ext, window::{PairIterExt, SlidingWindowIterExt}};

    use super::*;
    #[test]
    fn test_sip_hash() {
        let v1 = sip_::Hasher::hash_bytes(b"hello");
        let v2 = sip_::Hasher::hash_bytes(b"world");
        assert_ne!(v1, v2);
        assert_eq!(v1, sip_::Hasher::hash_bytes(b"hello"));
    }

    #[test]
    fn test_xxh3_hash() {
        let v1 = xxh3_::Hasher::hash_bytes(b"hello");
        let v2 = xxh3_::Hasher::hash_bytes(b"world");
        assert_ne!(v1, v2);
        assert_eq!(v1, xxh3_::Hasher::hash_bytes(b"hello"));
    }

    #[test]
    fn test_hashing_u8() {
        let data = b"bob";
        let v1 = data.iter().map(|&b| sip_::Hasher::hash_u8(b)).collect::<Vec<_>>();
        let v2 = sip_::Hasher::hashing_items_u8(data.iter().cloned()).collect::<Vec<_>>();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_pairs() {
        use crate::feature::Features;
        let data = "abce";
        let v1_it = data.byte_features().sliding_pairs().pair_to_u16();
        let v1 = sip_::Hasher::hashing_items_u16(v1_it).collect::<Vec<_>>();

        let v2 = data.byte_features().sliding_window(2).map(|w| sip_::Hasher::hash_bytes(w.as_ref())).collect::<Vec<_>>();
        assert_eq!(v1, v2);
    }

}