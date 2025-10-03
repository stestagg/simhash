use siphasher::sip::SipHasher;
use std::hash::{Hash, Hasher};
use lazy_static::lazy_static;

#[inline(always)]
pub fn sip_hash_fn(value: &[u8]) -> u64 {
    SipHasher::default().hash(value)
}

macro_rules! hash_impl {
    ($name:ident, $hash_fn:path) => {
        pub mod $name {
            use lazy_static::lazy_static;
            pub use $hash_fn as hash_fn;

            lazy_static! {
                static ref U16_TABLE: [u64; 65535] = {
                    let mut dest = [0; 65535];
                    (0..65535).for_each(|value: u16| dest[value as usize] = $hash_fn(value.to_le_bytes().as_ref()));
                    dest
                };
                static ref U8_TABLE: [u64; 255] = {
                    let mut dest = [0; 255];
                    (0..255).for_each(|value: u8| dest[value as usize] = $hash_fn([value, 0].as_ref()));
                    dest
                };
            }     

            #[inline(always)]
            pub fn lookup_u16(value: u16) -> u64 {
                U16_TABLE[value as usize]
            }

            #[inline(always)]
            pub fn lookup_u8(value: u8) -> u64 {
                U8_TABLE[value as usize]
            }
        }
    }
}

hash_impl!(sip_, super::sip_hash_fn);
hash_impl!(xxh3_, xxhash_rust::xxh3::xxh3_64);


pub fn hash_byte_pair<F>(bytes: &[u8], f: F) -> u64
where
    F: Fn(u16) -> u64,
{
    if bytes.is_empty() {
        return 0;
    }
    if bytes.len() == 1 {
        return f(u16::from_le_bytes([bytes[0], 0]));
    }

    let feature_adj: u32 = ((bytes.len() - 1) / 2) as u32;
    let mut buckets = [0u32; 64];

    for slice in bytes.windows(2).into_iter() {
        let feature = u16::from_le_bytes([slice[0], slice[1]]);
        let hashval = f(feature);
        for i in 0..64 {
            buckets[i] = buckets[i].saturating_add((hashval >> i & 1) as u32);
        }
    }

    let val = buckets.iter().enumerate().fold(0, |acc, (i, &b)| {
        let bitval = (if b > feature_adj { 1 } else { 0 }) << i;
        acc | bitval
    });
    val
}

pub fn hash_features<'a, 'b, H>(bytes: &'b [u8], hash: H, feature_extractor: Box<dyn FnOnce(&'b [u8]) -> Box<crate::AnyFeatureIter<'b>> + 'a>) -> u64
where
    H: Fn(&[u8]) -> u64,
{
    if bytes.is_empty() {
        return 0;
    }

    let features = feature_extractor(bytes);
    let mut feature_adj: u32 = 0;
    let mut buckets = [0u32; 64];

    for feature in features {
        feature_adj += 1;
        let hashval = hash(feature.as_ref());
        for i in 0..64 {
            buckets[i] = buckets[i].saturating_add((hashval >> i & 1) as u32);
        }
    }

    feature_adj /= 2;

    let val = buckets.iter().enumerate().fold(0, |acc, (i, &b)| {
        let bitval = (if b > feature_adj { 1 } else { 0 }) << i;
        acc | bitval
    });
    val
}



// impl<T: IntoU64> Shr<u64> for T{
//     type Output = (SimHash, u64);

//     #[inline]
//     fn shr(self, rhs: usize) -> Self::Output {
//         let mask = (1 << rhs) - 1;
//         (SimHash(self.0 >> rhs), self.0 & mask)
//     }
// }

// struct DistanceMeasure<T: Copy + Debug + Default> {
//     tree: HashTree<T>,
//     identity_map: HashMap<u64, T>
// }

// impl <T: Copy + Debug + Default> DistanceMeasure<T> {
//     pub fn new() -> Self {
//         Self {
//             tree: HashTree::new(),
//             identity_map: HashMap::new()
//         }
//     }

//     pub fn add(&mut self, hash: SimHash, value: T) {
//         self.tree.add(hash, value);
//         self.identity_map.insert(hash.0, value);
//     }

//     pub fn add_identity(&mut self, hash: SimHash, value: T) {
//         self.identity_map.insert(hash.0, value);
//     }

//     pub fn contains(&self, hash: SimHash, max_diff: usize) -> Option<T> {
//         match self.identity_map.get(&hash.0) {
//             Some(val) => return Some(*val),
//             None => {}
//         }
//         self.tree.contains(hash, max_diff)
//     }

// }


// pub fn deduplicate_texts<T: SimHashable>(texts: &[T], max_diff: usize) -> Vec<Option<usize>> {
//     let mut measurer = DistanceMeasure::<usize>::new();
//     let mut results = Vec::with_capacity(texts.len());

//     for (idx, text) in texts.iter().enumerate() {
//         let hash = text.simhash();
//         match measurer.contains(hash, max_diff) {
//             Some(val) => {
//                 results.push(Some(val));
//                 measurer.add_identity(hash, val);
//             },
//             None => {
//                 results.push(None);
//                 measurer.add(hash, idx);
//             }
//         }
//     }
//     results
// }


// pub fn group_texts<T: SimHashable>(texts: &[T], max_diff: usize) -> HashMap<usize, Vec<usize>> {
//     let mut measurer = DistanceMeasure::<usize>::new();
//     let mut groups = HashMap::<usize, Vec<usize>>::with_capacity(texts.len());

//     for (idx, text) in texts.iter().enumerate() {
//         let hash = text.simhash();
//         match measurer.contains(hash, max_diff) {
//             Some(val) => {
//                 groups.get_mut(&val).unwrap().push(idx);
//             }
//             None => {
//                 groups.insert(idx, vec![idx]);
//                 measurer.add(hash, idx);
//             }
//         }
//     }
//     groups
// }




// pub trait SimHashDedupeExt: Iterator
// {
//     fn deduplicate<F, Q>(self, max_diff: usize, mapper: F) -> SimHashDedupeFilter<Self, F, Q>
//     where 
//         F: Fn(&Self::Item) -> Q,
//         Q: SimHashable,
//         Self: Sized
//     {
//         SimHashDedupeFilter::<Self, F, Q>::new(self, max_diff, mapper)
//     }
// }


// impl<I> SimHashDedupeExt for I where I: Iterator {}

// pub struct SimHashDedupeFilter<I, F, U>
// where 
//     I: Iterator,
//     F: Fn(&I::Item) -> U,
//     U: SimHashable
// {
//     iter: I,
//     max_diff: usize,
//     measurer: DistanceMeasure<usize>,
//     mapper: F
// }

// impl<I, F, U> SimHashDedupeFilter<I, F, U>
// where 
//     I: Iterator,
//     F: Fn(&I::Item) -> U,
//     U: SimHashable
// {
//     pub fn new(iter: I, max_diff: usize, mapper: F) -> Self {
//         Self {
//             iter,
//             max_diff,
//             measurer: DistanceMeasure::new(),
//             mapper
//         }
//     }
// }


// impl<I, F, U> Iterator for SimHashDedupeFilter<I, F, U>
// where
//     I: Iterator,
//     I::Item: Debug,
//     F: Fn(&I::Item) -> U,
//     U: SimHashable,
// {
//     type Item = I::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             match self.iter.next() {
//                 Some(item) => {
//                     let hash = (self.mapper)(&item).simhash();
//                     match self.measurer.contains(hash, self.max_diff) {
//                         Some(_) => {}
//                         None => {
//                             self.measurer.add(hash, 0);
//                             return Some(item);
//                         }
//                     }
//                 }
//                 None => return None,
//             }
//         }
        
//     }
// }