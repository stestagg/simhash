use std::fmt::Debug;

const BRANCH_BITS: usize = 4;
const BRANCH_FACTOR: u8 = 1 << BRANCH_BITS;
const TREE_DEPTH: usize = 64 / BRANCH_BITS;

#[derive(Debug)]
enum HashTreeEntry<T: Debug> {
    None,
    Node(Box<HashTree<T>>),
}

impl<T: Debug + Default> Default for HashTreeEntry<T> {
    fn default() -> Self {
        HashTreeEntry::None
    }
}

#[inline(always)]
fn pop_bits(value: u64, bits: usize) -> (u64, u64) {
    let mask = (1 << bits) - 1;
    (value >> bits, value & mask)
}


#[derive(Debug)]
pub struct HashTree<T: Debug> {
    branches: [HashTreeEntry<T>; BRANCH_FACTOR as usize],
    value: Option<T>,
}

impl<T: Debug> HashTree<T> {

    pub fn new() -> Self {
        HashTree {
            branches: [const { HashTreeEntry::None }; BRANCH_FACTOR as usize],
            value: None,
        }
    }

    fn leaf(value: T) -> Self {
        HashTree {
            branches: [const { HashTreeEntry::None }; BRANCH_FACTOR as usize],
            value: Some(value),
        }
    }

    pub fn contains(&self, hash: u64, max_diff: usize) -> Option<&T> {
        self._contains(hash, max_diff, 0)
    }

    fn _contains(&self, hash: u64, max_diff: usize, level: usize) -> Option<&T> {
        let remaining_levels = TREE_DEPTH - level as usize;
        if remaining_levels == 0 {
            return self.value.as_ref();
        }

        let (rest, level_bits) = pop_bits(hash, BRANCH_BITS);
        for i in 0..BRANCH_FACTOR {
            let diff = (level_bits as u8 ^ i).count_ones() as usize;
            if diff < max_diff {

                match &self.branches[i as usize] {
                    HashTreeEntry::None => {},
                    HashTreeEntry::Node(node) => {
                        if let Some(val) = node._contains(rest, max_diff - diff, level + 1) {
                            return Some(val);
                        }
                    }
                }

            }
        }
        None
    }

    fn _add(&mut self, hash: u64, value: T, level: usize) {
        let (rest, level_bits) = pop_bits(hash, BRANCH_BITS);

        // If We ran out of bits, then add a leaf node (if needed!)
        if level == (TREE_DEPTH - 1) as usize {
            self.branches[level_bits as usize] = HashTreeEntry::Node(HashTree::leaf(value).into());
            return;
        }

        // We have some more bits, so drill down
        match self.branches[level_bits as usize] {
            HashTreeEntry::None => {
                // Hash tree has nothing below here so we need to add a node
                let mut node = HashTree::new();
                node._add(rest, value, level + 1);
                self.branches[level_bits as usize] = HashTreeEntry::Node(node.into());
            }
            HashTreeEntry::Node(ref mut node) => {
                node._add(rest, value, level + 1);
            }
        }
        
    }

    fn add(&mut self, hash: u64, value: T) {
        self._add(hash, value, 0);
    }
}
