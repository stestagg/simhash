
pub trait IntoU64 {
    fn into_u64(self) -> u64;
}

impl IntoU64 for u64 {
    fn into_u64(self) -> u64 {
        self
    }
}

pub fn hamming_distance<T: IntoU64, U: IntoU64>(a: T, b: U) -> u32 {
    (a.into_u64() ^ b.into_u64()).count_ones()
}
