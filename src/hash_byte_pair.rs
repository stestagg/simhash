use siphasher::sip::SipHasher;
use std::hash::{Hash, Hasher};
use lazy_static::lazy_static;

#[inline(always)]
fn sip_hash_byte_pair(value: u16) -> u64 {
    let mut s = SipHasher::default();
    value.hash(&mut s);
    s.finish()
}

#[inline(always)]
fn xxh3_hash_byte_pair(value: u16) -> u64 {
    xxhash_rust::xxh3::xxh3_64(&value.to_le_bytes())
}

macro_rules! define_sip_lookup {
    ($name:ident, $hash_fn:ident, $lookup_fn:ident) => {
        lazy_static! {
            static ref $name: [u64; 65535] = {
                let mut dest = [0; 65535];
                (0..65535).for_each(|value: u16| dest[value as usize] = $hash_fn(value));
                dest
            };
        }

        #[inline(always)]
        pub fn $lookup_fn(value: u16) -> u64 {
            $name[value as usize]
        }
    };
}

define_sip_lookup!(SIP_BYTE_PAIR_TABLE, sip_hash_byte_pair, sip_lookup_byte_pair);
define_sip_lookup!(XXH3_BYTE_PAIR_TABLE, xxh3_hash_byte_pair, xxh3_lookup_byte_pair);


pub fn hash_pair<F>(val: &str, f: F) -> u64
where
    F: Fn(u16) -> u64,
{
    let bytes = val.as_bytes();
    if bytes.is_empty() {
        return 0;
    }
    if bytes.len() == 1 {
        return f(u16::from_le_bytes([bytes[0], 0]));
    }

    let feature_adj: u32 = ((bytes.len() - 1) / 2) as u32;
    let mut buckets = [0u32; 64];

    for slice in val.as_bytes().windows(2).into_iter() {
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